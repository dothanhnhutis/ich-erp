use chrono::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Avatar,
    Document,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPolicy {
    /// Ai có URL đều xem được, serve qua CDN
    Public,
    /// Chỉ owner (hoặc người được authorize) xem được, cần presigned URL
    Private,
}

impl AssetKind {
    pub fn access_policy(&self) -> AccessPolicy {
        match self {
            AssetKind::Avatar => AccessPolicy::Public,
            AssetKind::Document => AccessPolicy::Private,
        }
    }

    pub fn max_size(&self) -> u64 {
        match self {
            AssetKind::Avatar => 5 * 1024 * 1024,     // 5MB
            AssetKind::Document => 500 * 1024 * 1024, // 500MB
        }
    }
    pub fn allowed_content_types(&self) -> &[&str] {
        match self {
            AssetKind::Avatar => &["image/png", "image/jpeg", "image/webp"],
            AssetKind::Document => &["application/pdf", "application/msword"],
        }
    }
}

pub trait ObjectStorage: Send + Sync {
    async fn create_multipart(
        &self,
        kind: AssetKind,
        key: &str,
        content_type: &str,
    ) -> impl Future<Output = Result<String, StorageError>> + Send;

    async fn presign_upload_part(
        &self,
        kind: AssetKind,
    ) -> impl Future<Output = Result<String, StorageError>> + Send;

    /// Trả về URL để CLIENT truy cập object.
    /// - Public: CDN URL trực tiếp (không cần presign)
    /// - Private: presigned GET URL với TTL
    async fn public_url(
        &self,
        kind: AssetKind,
        key: &str,
        ttl: Option<Duration>,
    ) -> impl Future<Output = Result<String, StorageError>> + Send;
}
