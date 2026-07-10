use serde::Serialize;
use tauri::{Manager, State};

#[tauri::command]
async fn login(app: tauri::AppHandle) -> Result<(), String> {
    // Tạo main window
    // let main_window = WebviewWindowBuilder::new(
    //     &app,
    //     "main",
    //     WebviewUrl::App("/#/dashboard".into()),
    // )
    // .build()
    // .map_err(|e| e.to_string())?;

    // main_window.show().map_err(|e| e.to_string())?;

    if let Some(main_win) = app.get_webview_window("main") {
        main_win.show().map_err(|e| e.to_string())?;
    }

    // Đóng login window
    if let Some(login_win) = app.get_webview_window("login") {
        login_win.close().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[derive(thiserror::Error, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum ApiError {
    #[error("{message}")]
    Network { message: String },

    #[error("{message}")]
    Unauthorized { message: String },

    #[error("Server error ({status}): {message}")]
    Server { status: u16, message: String },

    #[error("Chưa đăng nhập")]
    NotAuthenticated,
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Network {
            message: e.to_string(),
        }
    }
}

// #[derive(Default)]
// pub struct AuthState(tokio::sync::Mutex<Option<Tokens>>);

// #[tauri::command]
// pub async fn api_login1(
//     app: tauri::AppHandle,
//     state: State<'_, AuthState>,
//     client: State<'_, reqwest::Client>,
//     email: String,
//     password: String,
// ) -> Result<LoginOutcome, ApiError> {
//     let res = client
//         .post(format!("{}/auth/login", API_BASE_URL))
//         .json(&serde_json::json!({ "email": email, "password": password }))
//         .send()
//         .await?;

//     if !res.status().is_success() {
//         return Err(map_response_error(res).await);
//     }

//     let login: LoginResponse = res.json().await?;
//     let access_token = login.access_token.clone();

//     let profile_res = client
//         .get(format!("{}/users/me", API_BASE_URL))
//         .bearer_auth(&access_token)
//         .send()
//         .await?;

//     if !profile_res.status().is_success() {
//         return Err(map_response_error(profile_res).await);
//     }

//     let profile: ProfileResponse = profile_res.json().await?;

//     let tokens = Tokens {
//         access_token,
//         refresh_token: login.refresh_token,
//     };
//     save_tokens_to_disk(&app, &tokens);
//     *state.0.lock().await = Some(tokens);

//     Ok(LoginOutcome {
//         user_id: login.user_id,
//         profile,
//     })
// }

// async fn parse_json<T: serde::de::DeserializeOwned>(res: reqwest::Response) -> Result<T, ApiError> {
//     if !res.status().is_success() {
//         return Err(map_response_error(res).await);
//     }
//     res.json::<T>().await.map_err(Into::into)
// }

// async fn ensure_success(res: reqwest::Response) -> Result<(), ApiError> {
//     if !res.status().is_success() {
//         return Err(map_response_error(res).await);
//     }
//     Ok(())
// }

// #[tauri::command]
// pub async fn api_logout(
//     app: tauri::AppHandle,
//     state: State<'_, AuthState>,
//     client: State<'_, reqwest::Client>,
// ) -> Result<(), ApiError> {
//     let tokens = state.0.lock().await.take();
//     delete_tokens_file(&app);

//     if let Some(t) = tokens {
//         let _ = client
//             .post(format!("{}/auth/logout", API_BASE_URL))
//             .bearer_auth(&t.access_token)
//             .json(&serde_json::json!({ "refresh_token": t.refresh_token }))
//             .send()
//             .await;
//     }

//     Ok(())
// }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![login])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
