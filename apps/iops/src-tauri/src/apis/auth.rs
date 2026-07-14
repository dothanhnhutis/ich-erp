use application::dto::{
    auth_dto::{LoginRequest, LoginResponse},
    user_dto::UserResponse,
};
use domain::entities::session::Session;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

use crate::{
    error::{map_response_error, ApiError},
    helper::{
        delete_tokens_file, get_access_token, load_tokens_from_disk, save_tokens_to_disk,
        show_login, show_main, API_BASE_URL,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthContext {
    session: Session,
    user: UserResponse,
    permission_codes: Vec<String>,
}

#[derive(Default)]
pub struct AuthState(pub tokio::sync::Mutex<Option<LoginResponse>>);

#[tauri::command]
pub async fn login(
    app: tauri::AppHandle,
    state: State<'_, AuthState>,
    client: State<'_, reqwest::Client>,
    payload: LoginRequest,
) -> Result<AuthContext, ApiError> {
    let res = client
        .post(format!("{}/auth/login", API_BASE_URL))
        .json(&serde_json::json!(payload))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(map_response_error(res).await);
    }

    let token: LoginResponse = res.json().await?;

    let profile_res = client
        .get(format!("{}/users/me", API_BASE_URL))
        .bearer_auth(&token.session)
        .send()
        .await?;

    if !profile_res.status().is_success() {
        return Err(map_response_error(profile_res).await);
    }

    let auth_context: AuthContext = profile_res.json().await?;

    save_tokens_to_disk(&app, &token);
    *state.0.lock().await = Some(token);

    show_main(&app)?;

    Ok(auth_context)
}

/// Khôi phục session từ disk + validate với backend. Nạp `AuthState`, KHÔNG đụng window.
/// Dùng chung cho `setup` (chọn window) và command `hydrate` (đổ vào React).
/// Trả `Some(profile)` nếu session còn sống, `None` nếu phải đăng nhập lại.
pub async fn restore_auth(app: &tauri::AppHandle) -> Result<Option<AuthContext>, ApiError> {
    let state = app.state::<AuthState>();

    // 1. Nạp tokens từ disk nếu state đang rỗng
    {
        let mut guard = state.0.lock().await;
        if guard.is_none() {
            *guard = load_tokens_from_disk(app);
        }
        if guard.is_none() {
            return Ok(None);
        }
    }

    // 2. Validate qua /users/me
    let client = app.state::<reqwest::Client>();
    let session = get_access_token(&state).await?;
    let res = client
        .get(format!("{}/users/me", API_BASE_URL))
        .bearer_auth(&session)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(Some(res.json().await?))
    } else {
        *state.0.lock().await = None;
        delete_tokens_file(app);
        Ok(None)
    }
}

/// Command frontend gọi lúc mount để đổ auth vào React context (window đã do
/// `setup`/`login`/`logout` lo).
#[tauri::command]
pub async fn hydrate(app: tauri::AppHandle) -> Result<Option<AuthContext>, ApiError> {
    restore_auth(&app).await
}

#[tauri::command]
pub async fn logout(
    app: tauri::AppHandle,
    state: State<'_, AuthState>,
    client: State<'_, reqwest::Client>,
) -> Result<(), ApiError> {
    let tokens = state.0.lock().await.take();
    delete_tokens_file(&app);

    if let Some(t) = tokens {
        let _ = client
            .post(format!("{}/auth/logout", API_BASE_URL))
            .bearer_auth(&t.session)
            .send()
            .await;
    }

    show_login(&app)?;

    Ok(())
}
