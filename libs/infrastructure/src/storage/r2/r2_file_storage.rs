// pub struct R2Config {
//     pub public_bucket: String,
//     pub private_bucket: String,
//     pub public_cdn_base_url: String, // "https://cdn.yourapp.com"
// }

// pub struct R2Storage {
//     client: Client, // 1 client dùng chung, credentials cùng account
//     config: R2Config,
// }

// impl R2Storage {
//     fn bucket_for(&self, kind: AssetKind) -> &str {
//         match kind.access_policy() {
//             AccessPolicy::Public => &self.config.public_bucket,
//             AccessPolicy::Private => &self.config.private_bucket,
//         }
//     }
// }

// impl ObjectStorage for R2Storage {
//     async fn create_multipart(
//         &self,
//         kind: AssetKind,
//         key: &str,
//         content_type: &str,
//     ) -> Result<String, AppError> {
//         let out = self
//             .client
//             .create_multipart_upload()
//             .bucket(self.bucket_for(kind))
//             .key(key)
//             .content_type(content_type)
//             .send()
//             .await
//             .map_err(map_s3_error)?;
//         out.upload_id
//             .ok_or(AppError::Internal("no upload_id".into()))
//     }

//     async fn presign_upload_part(
//         &self,
//         kind: AssetKind,
//         key: &str,
//         upload_id: &str,
//         part_number: i32,
//         expires_in: Duration,
//     ) -> Result<String, AppError> {
//         let cfg = PresigningConfig::expires_in(expires_in)
//             .map_err(|e| AppError::Internal(e.to_string()))?;
//         let req = self
//             .client
//             .upload_part()
//             .bucket(self.bucket_for(kind))
//             .key(key)
//             .upload_id(upload_id)
//             .part_number(part_number)
//             .presigned(cfg)
//             .await
//             .map_err(map_s3_error)?;
//         Ok(req.uri().to_string())
//     }

//     async fn public_url(
//         &self,
//         kind: AssetKind,
//         key: &str,
//         ttl: Option<Duration>,
//     ) -> Result<String, AppError> {
//         match kind.access_policy() {
//             AccessPolicy::Public => {
//                 // Serve qua CDN — không presign
//                 Ok(format!("{}/{}", self.config.public_cdn_base_url, key))
//             }
//             AccessPolicy::Private => {
//                 let ttl = ttl.unwrap_or(Duration::from_secs(900)); // default 15p
//                 let cfg = PresigningConfig::expires_in(ttl)
//                     .map_err(|e| AppError::Internal(e.to_string()))?;
//                 let req = self
//                     .client
//                     .get_object()
//                     .bucket(self.bucket_for(kind))
//                     .key(key)
//                     .presigned(cfg)
//                     .await
//                     .map_err(map_s3_error)?;
//                 Ok(req.uri().to_string())
//             }
//         }
//     }
// }

use aws_sdk_s3::{
    Client,
    error::SdkError,
    presigning::PresigningConfig,
    types::{CompletedMultipartUpload, CompletedPart as S3CompletedPart},
};
use std::time::Duration;

use domain::upload::{
    AccessPolicy, AssetKind, CompletedPart, MultipartUpload, ObjectStorage, StorageError,
};

#[derive(Debug, Clone)]
pub struct R2Config {
    pub public_bucket: String,
    pub private_bucket: String,
    pub public_cdn_base_url: String,
}

pub struct R2Storage {
    client: Client,
    config: R2Config,
}

impl R2Storage {
    pub fn new(client: Client, config: R2Config) -> Self {
        Self { client, config }
    }

    fn bucket_for(&self, kind: AssetKind) -> &str {
        match kind.access_policy() {
            AccessPolicy::Public => &self.config.public_bucket,
            AccessPolicy::Private => &self.config.private_bucket,
        }
    }

    fn presign_config(expires_in: Duration) -> Result<PresigningConfig, StorageError> {
        PresigningConfig::expires_in(expires_in).map_err(|e| StorageError::Internal(e.to_string()))
    }
}

pub fn map_s3_error<E>(error: SdkError<E>, key: &str) -> StorageError
where
    E: std::fmt::Debug,
{
    match error {
        SdkError::TimeoutError(_) => StorageError::Unavailable("storage request timeout".into()),

        SdkError::DispatchFailure(err) => {
            StorageError::Unavailable(format!("storage network error: {err:?}"))
        }

        SdkError::ConstructionFailure(err) => {
            StorageError::Internal(format!("storage request construction failed: {err:?}"))
        }

        SdkError::ResponseError(err) => {
            StorageError::Unavailable(format!("storage response error: {err:?}"))
        }

        // SdkError::ServiceError(err) => {
        //     // let status = err.raw_response().status().as_u16();
        //     let status = err.raw().status().as_u16();

        //     match status {
        //         401 | 403 => StorageError::Unauthorized,

        //         404 => StorageError::NotFound {
        //             key: key.to_owned(),
        //         },

        //         400 => StorageError::InvalidRequest(format!("{:?}", err.err())),

        //         429 | 500..=599 => {
        //             StorageError::Unavailable(format!("storage returned HTTP {status}"))
        //         }

        //         _ => StorageError::Internal(format!(
        //             "storage returned HTTP {status}: {:?}",
        //             err.err()
        //         )),
        //     }
        // }
        SdkError::ServiceError(err) => {
            let status = err.raw().status().as_u16();

            match status {
                401 | 403 => StorageError::Unauthorized,

                404 => StorageError::NotFound {
                    key: key.to_owned(),
                },

                400 => StorageError::InvalidRequest(format!("{:?}", err.err())),

                429 | 500..=599 => {
                    StorageError::Unavailable(format!("storage returned HTTP {status}"))
                }

                _ => StorageError::Internal(format!(
                    "storage returned HTTP {status}: {:?}",
                    err.err()
                )),
            }
        }
        other => StorageError::Internal(format!("unexpected storage error: {other:?}")),
        // other => StorageError::Internal(format!("unexpected storage error: {other:?}")),
    }
}

impl ObjectStorage for R2Storage {
    async fn presign_put(
        &self,
        kind: AssetKind,
        key: &str,
        content_type: &str,
        expires_in: Duration,
    ) -> Result<String, StorageError> {
        let cfg = Self::presign_config(expires_in)?;

        let req = self
            .client
            .put_object()
            .bucket(self.bucket_for(kind))
            .key(key)
            .content_type(content_type)
            .presigned(cfg)
            .await
            .map_err(|error| map_s3_error(error, key))?;

        Ok(req.uri().to_string())
    }

    async fn create_multipart(
        &self,
        kind: AssetKind,
        key: &str,
        content_type: &str,
    ) -> Result<MultipartUpload, StorageError> {
        let out = self
            .client
            .create_multipart_upload()
            .bucket(self.bucket_for(kind))
            .key(key)
            .content_type(content_type)
            .send()
            .await
            .map_err(|error| map_s3_error(error, key))?;

        let upload_id = out
            .upload_id()
            .ok_or_else(|| StorageError::Internal("missing upload_id".into()))?
            .to_owned();

        Ok(MultipartUpload {
            upload_id,
            key: key.to_owned(),
        })
    }

    async fn presign_upload_part(
        &self,
        kind: AssetKind,
        key: &str,
        upload_id: &str,
        part_number: i32,
        expires_in: Duration,
    ) -> Result<String, StorageError> {
        let cfg = Self::presign_config(expires_in)?;

        let req = self
            .client
            .upload_part()
            .bucket(self.bucket_for(kind))
            .key(key)
            .upload_id(upload_id)
            .part_number(part_number)
            .presigned(cfg)
            .await
            .map_err(|error| map_s3_error(error, key))?;

        Ok(req.uri().to_string())
    }

    async fn complete_multipart(
        &self,
        kind: AssetKind,
        key: &str,
        upload_id: &str,
        parts: Vec<CompletedPart>,
    ) -> Result<(), StorageError> {
        let completed_parts = parts
            .into_iter()
            .map(|part| {
                S3CompletedPart::builder()
                    .part_number(part.part_number)
                    .e_tag(part.etag)
                    .build()
            })
            .collect::<Vec<_>>();

        let upload = CompletedMultipartUpload::builder()
            .set_parts(Some(completed_parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(self.bucket_for(kind))
            .key(key)
            .upload_id(upload_id)
            .multipart_upload(upload)
            .send()
            .await
            .map_err(|error| map_s3_error(error, key))?;

        Ok(())
    }

    async fn abort_multipart(
        &self,
        kind: AssetKind,
        key: &str,
        upload_id: &str,
    ) -> Result<(), StorageError> {
        self.client
            .abort_multipart_upload()
            .bucket(self.bucket_for(kind))
            .key(key)
            .upload_id(upload_id)
            .send()
            .await
            .map_err(|error| map_s3_error(error, key))?;

        Ok(())
    }

    async fn delete(&self, kind: AssetKind, key: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(self.bucket_for(kind))
            .key(key)
            .send()
            .await
            .map_err(|error| map_s3_error(error, key))?;

        Ok(())
    }

    async fn object_url(
        &self,
        kind: AssetKind,
        key: &str,
        ttl: Option<Duration>,
    ) -> Result<String, StorageError> {
        match kind.access_policy() {
            AccessPolicy::Public => Ok(format!(
                "{}/{}",
                self.config.public_cdn_base_url.trim_end_matches('/'),
                key.trim_start_matches('/')
            )),

            AccessPolicy::Private => {
                let ttl = ttl.unwrap_or(Duration::from_secs(15 * 60));

                let cfg = Self::presign_config(ttl)?;

                let req = self
                    .client
                    .get_object()
                    .bucket(self.bucket_for(kind))
                    .key(key)
                    .presigned(cfg)
                    .await
                    .map_err(|error| map_s3_error(error, key))?;

                Ok(req.uri().to_string())
            }
        }
    }
}
