use application::dto::auth_dto::LoginResponse;
use std::path::PathBuf;
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::{
    apis::auth::AuthState,
    error::{map_response_error, ApiError},
};

pub const API_BASE_URL: &str = "http://localhost:4000/api/v1";

// ===== Token persistence helpers =====

pub fn token_file(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .app_local_data_dir()
        .ok()
        .map(|p| p.join("auth.json"))
}

pub fn load_tokens_from_disk(app: &tauri::AppHandle) -> Option<LoginResponse> {
    let path = token_file(app)?;
    let json = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}

pub fn save_tokens_to_disk(app: &tauri::AppHandle, tokens: &LoginResponse) {
    let Some(path) = token_file(app) else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(tokens) {
        let _ = std::fs::write(path, json);
    }
}

pub fn delete_tokens_file(app: &tauri::AppHandle) {
    if let Some(path) = token_file(app) {
        let _ = std::fs::remove_file(path);
    }
}

pub async fn parse_json<T: serde::de::DeserializeOwned>(
    res: reqwest::Response,
) -> Result<T, ApiError> {
    if !res.status().is_success() {
        return Err(map_response_error(res).await);
    }
    res.json::<T>().await.map_err(Into::into)
}

pub async fn get_access_token(state: &State<'_, AuthState>) -> Result<String, ApiError> {
    let guard = state.0.lock().await;
    guard
        .as_ref()
        .map(|t| t.session.clone())
        .ok_or(ApiError::NotAuthenticated)
}

// ===== Window control (create-on-demand) =====
// Tạo mới window đích + đóng window cũ. Window đích luôn mount mới nên frontend
// hydrate lại đúng trạng thái — không lo state cũ, không dùng show/hide.

/// Mở cửa sổ chính (app đã đăng nhập), đóng cửa sổ login nếu còn.
pub fn show_main(app: &tauri::AppHandle) -> Result<(), ApiError> {
    if let Some(login_win) = app.get_webview_window("login") {
        let _ = login_win.close();
    }
    if app.get_webview_window("main").is_none() {
        WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
            .title("iops")
            .inner_size(1920.0, 1080.0)
            .build()
            .map_err(|e| ApiError::WindowError {
                message: e.to_string(),
            })?;
    }
    Ok(())
}

/// Mở cửa sổ login (chưa đăng nhập / vừa đăng xuất), đóng cửa sổ main nếu còn.
pub fn show_login(app: &tauri::AppHandle) -> Result<(), ApiError> {
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.close();
    }
    if app.get_webview_window("login").is_none() {
        WebviewWindowBuilder::new(app, "login", WebviewUrl::App("index.html".into()))
            .title("Login")
            .inner_size(480.0, 600.0)
            .center()
            .resizable(false)
            .build()
            .map_err(|e| ApiError::WindowError {
                message: e.to_string(),
            })?;
    }
    Ok(())
}
