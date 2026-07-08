pub struct R2Config {
    pub public_bucket: String,
    pub private_bucket: String,
    pub public_cdn_base_url: String, // "https://cdn.yourapp.com"
}

pub struct R2Storage {
    client: Client, // 1 client dùng chung, credentials cùng account
    config: R2Config,
}

impl R2Storage {
    fn bucket_for(&self, kind: AssetKind) -> &str {
        match kind.access_policy() {
            AccessPolicy::Public => &self.config.public_bucket,
            AccessPolicy::Private => &self.config.private_bucket,
        }
    }
}

impl ObjectStorage for R2Storage {
    async fn create_multipart(
        &self,
        kind: AssetKind,
        key: &str,
        content_type: &str,
    ) -> Result<String, AppError> {
        let out = self
            .client
            .create_multipart_upload()
            .bucket(self.bucket_for(kind))
            .key(key)
            .content_type(content_type)
            .send()
            .await
            .map_err(map_s3_error)?;
        out.upload_id
            .ok_or(AppError::Internal("no upload_id".into()))
    }

    async fn presign_upload_part(
        &self,
        kind: AssetKind,
        key: &str,
        upload_id: &str,
        part_number: i32,
        expires_in: Duration,
    ) -> Result<String, AppError> {
        let cfg = PresigningConfig::expires_in(expires_in)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let req = self
            .client
            .upload_part()
            .bucket(self.bucket_for(kind))
            .key(key)
            .upload_id(upload_id)
            .part_number(part_number)
            .presigned(cfg)
            .await
            .map_err(map_s3_error)?;
        Ok(req.uri().to_string())
    }

    async fn public_url(
        &self,
        kind: AssetKind,
        key: &str,
        ttl: Option<Duration>,
    ) -> Result<String, AppError> {
        match kind.access_policy() {
            AccessPolicy::Public => {
                // Serve qua CDN — không presign
                Ok(format!("{}/{}", self.config.public_cdn_base_url, key))
            }
            AccessPolicy::Private => {
                let ttl = ttl.unwrap_or(Duration::from_secs(900)); // default 15p
                let cfg = PresigningConfig::expires_in(ttl)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                let req = self
                    .client
                    .get_object()
                    .bucket(self.bucket_for(kind))
                    .key(key)
                    .presigned(cfg)
                    .await
                    .map_err(map_s3_error)?;
                Ok(req.uri().to_string())
            }
        }
    }
}
