use crate::{
    apis::auth::{AuthContext, AuthState},
    error::ApiError,
    helper::{get_access_token, parse_json, API_BASE_URL},
};
use application::dto::user_dto::UserResponse;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

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

fn push_pagination(
    mut req: reqwest::RequestBuilder,
    page: Option<u32>,
    page_size: Option<u32>,
    q: Option<String>,
) -> reqwest::RequestBuilder {
    if let Some(p) = page {
        req = req.query(&[("page", p.to_string())]);
    }
    if let Some(ps) = page_size {
        req = req.query(&[("page_size", ps.to_string())]);
    }
    if let Some(q) = q.filter(|s| !s.is_empty()) {
        req = req.query(&[("q", q)]);
    }
    req
}

#[tauri::command]
pub async fn list_users(
    state: State<'_, AuthState>,
    client: State<'_, reqwest::Client>,
    page: Option<u32>,
    page_size: Option<u32>,
    q: Option<String>,
) -> Result<Paginated<UserResponse>, ApiError> {
    let token = get_access_token(&state).await?;
    let mut req = client
        .get(format!("{}/users", API_BASE_URL))
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

#[tauri::command]
pub async fn get_user_by_id(
    state: State<'_, AuthState>,
    client: State<'_, reqwest::Client>,
    id: String,
) -> Result<UserResponse, ApiError> {
    let token = get_access_token(&state).await?;
    let res = client
        .get(format!("{}/users/{}", API_BASE_URL, id))
        .bearer_auth(&token)
        .send()
        .await?;
    parse_json(res).await
}
