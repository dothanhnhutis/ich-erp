mod apis;
mod error;
mod helper;
use apis::auth;
use apis::role;
use apis::user;
use helper::{show_login, show_main};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(reqwest::Client::new())
        .manage(auth::AuthState::default())
        .setup(|app| {
            // Config không khai báo window nào — tự tạo window đầu tiên theo trạng thái
            // đăng nhập (khôi phục từ disk). Đã đăng nhập → main; chưa → login.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let opened = match auth::restore_auth(&handle).await {
                    Ok(Some(_)) => show_main(&handle),
                    _ => show_login(&handle),
                };
                if let Err(e) = opened {
                    eprintln!("setup: không mở được window: {e}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::login,
            auth::hydrate,
            auth::logout,
            role::list_roles,
            user::me,
            user::list_users,
            user::get_user_by_id
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
