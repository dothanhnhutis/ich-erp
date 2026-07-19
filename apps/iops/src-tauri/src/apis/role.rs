use application::dto::role_dto::RoleResponse;
use tauri::State;

use crate::{
    apis::{auth::AuthState, user::Paginated},
    error::ApiError,
    helper::{get_access_token, parse_json, API_BASE_URL},
};

#[tauri::command]
pub async fn list_roles(
    state: State<'_, AuthState>,
    client: State<'_, reqwest::Client>,
    page: Option<u32>,
    page_size: Option<u32>,
    q: Option<String>,
) -> Result<Paginated<RoleResponse>, ApiError> {
    let token = get_access_token(&state).await?;
    let mut req = client
        .get(format!("{}/roles", API_BASE_URL))
        .bearer_auth(&token);
    if let Some(p) = page {
        req = req.query(&[("page", p.to_string())]);
    }
    if let Some(ps) = page_size {
        req = req.query(&[("page_size", ps.to_string())]);
    }
    if let Some(q) = q.filter(|s| !s.is_empty()) {
        req = req.query(&[("q", q)]);
    }
    let res = req.send().await?;
    parse_json(res).await
}
