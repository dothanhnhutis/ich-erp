use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::email::EmailError;

/// Endpoint đổi refresh token lấy access token của Google.
const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
/// Refresh sớm 60s trước khi token thực sự hết hạn (tránh dùng token vừa hết hạn).
const EXPIRY_SKEW: Duration = Duration::from_secs(60);

struct Cached {
    token: String,
    /// Mốc (monotonic) coi như token hết hạn — đã trừ `EXPIRY_SKEW`.
    expires_at: Instant,
}

/// Giữ credential OAuth2 của Google + cache access token, tự refresh khi hết hạn.
pub struct GmailOAuth {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    http: reqwest::Client,
    cache: Mutex<Option<Cached>>,
}

#[derive(serde::Deserialize)]
struct TokenResp {
    access_token: String,
    expires_in: u64,
}

impl GmailOAuth {
    pub fn new(client_id: String, client_secret: String, refresh_token: String) -> Self {
        Self {
            client_id,
            client_secret,
            refresh_token,
            http: reqwest::Client::new(),
            cache: Mutex::new(None),
        }
    }

    /// Trả access token còn hạn; nếu thiếu/sắp hết hạn thì POST refresh_token grant.
    /// Mọi lỗi mạng/HTTP xếp vào `Transport` (tạm thời → consumer sẽ requeue).
    pub async fn access_token(&self) -> Result<String, EmailError> {
        let mut guard = self.cache.lock().await;
        if let Some(c) = guard.as_ref()
            && c.expires_at > Instant::now()
        {
            return Ok(c.token.clone());
        }

        let resp = self
            .http
            .post(TOKEN_ENDPOINT)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", self.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| EmailError::Transport(format!("gọi token endpoint lỗi: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(EmailError::Transport(format!(
                "refresh access token thất bại ({status}): {body}"
            )));
        }

        let token: TokenResp = resp
            .json()
            .await
            .map_err(|e| EmailError::Transport(format!("parse token response lỗi: {e}")))?;

        let expires_at =
            Instant::now() + Duration::from_secs(token.expires_in).saturating_sub(EXPIRY_SKEW);
        let access = token.access_token.clone();
        *guard = Some(Cached {
            token: token.access_token,
            expires_at,
        });
        Ok(access)
    }
}
