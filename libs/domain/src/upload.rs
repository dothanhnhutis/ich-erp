// use chrono::Duration;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum AssetKind {
//     Avatar,
//     Document,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum AccessPolicy {
//     /// Ai có URL đều xem được, serve qua CDN
//     Public,
//     /// Chỉ owner (hoặc người được authorize) xem được, cần presigned URL
//     Private,
// }

// impl AssetKind {
//     pub fn access_policy(&self) -> AccessPolicy {
//         match self {
//             AssetKind::Avatar => AccessPolicy::Public,
//             AssetKind::Document => AccessPolicy::Private,
//         }
//     }

//     pub fn max_size(&self) -> u64 {
//         match self {
//             AssetKind::Avatar => 5 * 1024 * 1024,     // 5MB
//             AssetKind::Document => 500 * 1024 * 1024, // 500MB
//         }
//     }
//     pub fn allowed_content_types(&self) -> &[&str] {
//         match self {
//             AssetKind::Avatar => &["image/png", "image/jpeg", "image/webp"],
//             AssetKind::Document => &["application/pdf", "application/msword"],
//         }
//     }
// }

// pub trait ObjectStorage: Send + Sync {
//     async fn create_multipart(
//         &self,
//         kind: AssetKind,
//         key: &str,
//         content_type: &str,
//     ) -> impl Future<Output = Result<String, StorageError>> + Send;

//     async fn presign_upload_part(
//         &self,
//         kind: AssetKind,
//     ) -> impl Future<Output = Result<String, StorageError>> + Send;

//     /// Trả về URL để CLIENT truy cập object.
//     /// - Public: CDN URL trực tiếp (không cần presign)
//     /// - Private: presigned GET URL với TTL
//     async fn public_url(
//         &self,
//         kind: AssetKind,
//         key: &str,
//         ttl: Option<Duration>,
//     ) -> impl Future<Output = Result<String, StorageError>> + Send;
// }
use std::future::Future;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage object not found: {key}")]
    NotFound { key: String },

    #[error("storage unauthorized")]
    Unauthorized,

    #[error("invalid storage request: {0}")]
    InvalidRequest(String),

    #[error("storage unavailable: {0}")]
    Unavailable(String),

    #[error("storage internal error: {0}")]
    Internal(String),
}

impl StorageError {
    pub fn not_found(key: impl Into<String>) -> Self {
        Self::NotFound { key: key.into() }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::Unavailable(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Avatar,
    Document,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPolicy {
    Public,
    Private,
}

impl AssetKind {
    pub fn access_policy(self) -> AccessPolicy {
        match self {
            Self::Avatar => AccessPolicy::Public,
            Self::Document => AccessPolicy::Private,
        }
    }

    pub fn max_size(self) -> u64 {
        match self {
            Self::Avatar => 5 * 1024 * 1024,
            Self::Document => 500 * 1024 * 1024,
        }
    }

    pub fn allowed_content_types(self) -> &'static [&'static str] {
        match self {
            Self::Avatar => &["image/png", "image/jpeg", "image/jpg", "image/webp"],
            Self::Document => &[
                "application/pdf",
                "application/msword",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct PresignedUpload {
    pub url: String,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct MultipartUpload {
    pub upload_id: String,
    pub key: String,
}

pub trait ObjectStorage: Send + Sync {
    /// Avatar / small file:
    /// tạo presigned PUT URL.
    fn presign_put(
        &self,
        kind: AssetKind,
        key: &str,
        content_type: &str,
        expires_in: Duration,
    ) -> impl Future<Output = Result<String, StorageError>> + Send;

    /// Document / file lớn:
    /// tạo multipart upload.
    fn create_multipart(
        &self,
        kind: AssetKind,
        key: &str,
        content_type: &str,
    ) -> impl Future<Output = Result<MultipartUpload, StorageError>> + Send;

    /// Tạo URL cho từng part.
    fn presign_upload_part(
        &self,
        kind: AssetKind,
        key: &str,
        upload_id: &str,
        part_number: i32,
        expires_in: Duration,
    ) -> impl Future<Output = Result<String, StorageError>> + Send;

    /// Hoàn tất multipart upload.
    fn complete_multipart(
        &self,
        kind: AssetKind,
        key: &str,
        upload_id: &str,
        parts: Vec<CompletedPart>,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    /// Hủy multipart upload khi client fail/cancel.
    fn abort_multipart(
        &self,
        kind: AssetKind,
        key: &str,
        upload_id: &str,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    /// Xóa object.
    fn delete(
        &self,
        kind: AssetKind,
        key: &str,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    /// URL client dùng để đọc file.
    ///
    /// Public:
    ///     https://cdn.example.com/...
    ///
    /// Private:
    ///     presigned GET URL
    fn object_url(
        &self,
        kind: AssetKind,
        key: &str,
        ttl: Option<Duration>,
    ) -> impl Future<Output = Result<String, StorageError>> + Send;
}

#[derive(Debug, Clone)]
pub struct CompletedPart {
    pub part_number: i32,
    pub etag: String,
}
