use super::Storage;
use async_trait::async_trait;
use s3::{creds::Credentials, Bucket, Region};
use std::{fmt::Debug, time::Duration};

#[derive(Debug, Clone)]
pub struct R2Storage {
    bucket: Box<Bucket>,
}

impl R2Storage {
    const BUCKET_NAME: &'static str = "botcast";

    /// `S3_ENDPOINT` が設定されていれば S3 互換ストレージ (MinIO 等) を path-style で使い、
    /// なければ `CLOUDFLARE_ACCOUNT_ID` の R2 を使う
    pub fn new() -> anyhow::Result<Self> {
        let credentials = Credentials::from_env()?;
        let bucket = match std::env::var("S3_ENDPOINT") {
            Ok(endpoint) => Bucket::new(
                Self::BUCKET_NAME,
                Region::Custom {
                    region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
                    endpoint,
                },
                credentials,
            )?
            .with_path_style(),
            Err(_) => Bucket::new(
                Self::BUCKET_NAME,
                Region::R2 {
                    account_id: std::env::var("CLOUDFLARE_ACCOUNT_ID")?,
                },
                credentials,
            )?,
        };
        Ok(Self { bucket })
    }
}

#[async_trait]
impl Storage for R2Storage {
    async fn get_presign_url(&self, path: &str, expiration: Duration) -> anyhow::Result<String> {
        let url = self
            .bucket
            .presign_get(path, expiration.as_secs() as u32, None)
            .await?
            .to_string();
        Ok(url)
    }

    async fn upload(&self, path: &str, data: &[u8], content_type: &str) -> anyhow::Result<()> {
        self.bucket
            .put_object_with_content_type(path, data, content_type)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    pub(crate) struct DummyStorage;

    #[async_trait]
    impl Storage for DummyStorage {
        async fn get_presign_url(
            &self,
            _path: &str,
            _expiration: Duration,
        ) -> anyhow::Result<String> {
            Ok("http://example.com".to_string())
        }

        async fn upload(
            &self,
            _path: &str,
            _data: &[u8],
            _content_type: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }
}
