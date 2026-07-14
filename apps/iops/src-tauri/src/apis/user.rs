use crate::{
    apis::auth::{AuthContext, AuthState},
    error::ApiError,
    helper::{get_access_token, parse_json, API_BASE_URL},
};
use tauri::State;

#[tauri::command]
pub async fn me(
    state: State<'_, AuthState>,
    client: State<'_, reqwest::Client>,
) -> Result<AuthContext, ApiError> {
    let session = get_access_token(&state).await?;
    let res = client
        .get(format!("{}/users/me", API_BASE_URL))
        .bearer_auth(&session)
        .send()
        .await?;
    parse_json(res).await
}
