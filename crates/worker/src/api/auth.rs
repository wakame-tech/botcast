use anyhow::Context;
use axum::http::{header::AUTHORIZATION, HeaderMap};
use reqwest::StatusCode;

/// `Authorization: Bearer <token>` からトークンを取り出す。
///
/// スキーム名は大文字小文字を区別しない。ヘッダーが無い・Bearer でない・トークンが空なら `None`。
pub(crate) fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.trim().split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = token.trim();
    (!token.is_empty()).then_some(token)
}

/// 検証結果
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenStatus {
    Valid,
    Invalid,
}

/// CMS の応答ステータスをトークンの有効性に読み替える。
/// 401/403 は無効、2xx は有効、それ以外は CMS 側の異常としてエラーにする。
pub(crate) fn token_status(status: StatusCode) -> anyhow::Result<TokenStatus> {
    if status.is_success() {
        Ok(TokenStatus::Valid)
    } else if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        Ok(TokenStatus::Invalid)
    } else {
        anyhow::bail!("unexpected CMS status while verifying token: {}", status)
    }
}

/// botcast-cms の JWT を CMS に問い合わせて検証する。
///
/// worker は JWT の秘密鍵を持たないため、そのトークンで CMS の `GET /collections` を叩き、
/// 通れば有効とみなす（リクエストごとに問い合わせ、キャッシュはしない）。
#[derive(Debug, Clone)]
pub(crate) struct CmsTokenVerifier {
    http: reqwest::Client,
    base_url: String,
}

impl CmsTokenVerifier {
    pub(crate) fn from_env() -> Self {
        let base_url =
            std::env::var("CMS_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub(crate) async fn verify(&self, token: &str) -> anyhow::Result<TokenStatus> {
        let res = self
            .http
            .get(format!("{}/collections", self.base_url))
            .bearer_auth(token)
            .send()
            .await
            .context("CMS request failed while verifying token")?;
        token_status(res.status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(AUTHORIZATION, HeaderValue::from_str(value).unwrap());
        h
    }

    #[test]
    fn bearer_token_extracts_token() {
        assert_eq!(bearer_token(&headers("Bearer abc.def.ghi")), Some("abc.def.ghi"));
    }

    #[test]
    fn bearer_token_scheme_is_case_insensitive() {
        assert_eq!(bearer_token(&headers("bearer tok")), Some("tok"));
    }

    #[test]
    fn bearer_token_missing_header_is_none() {
        assert_eq!(bearer_token(&HeaderMap::new()), None);
    }

    #[test]
    fn bearer_token_other_scheme_is_none() {
        assert_eq!(bearer_token(&headers("Basic dXNlcjpwYXNz")), None);
    }

    #[test]
    fn bearer_token_empty_token_is_none() {
        assert_eq!(bearer_token(&headers("Bearer ")), None);
        assert_eq!(bearer_token(&headers("Bearer")), None);
    }

    #[test]
    fn token_status_maps_cms_status() {
        assert_eq!(token_status(StatusCode::OK).unwrap(), TokenStatus::Valid);
        assert_eq!(
            token_status(StatusCode::UNAUTHORIZED).unwrap(),
            TokenStatus::Invalid
        );
        assert_eq!(
            token_status(StatusCode::FORBIDDEN).unwrap(),
            TokenStatus::Invalid
        );
        assert!(token_status(StatusCode::INTERNAL_SERVER_ERROR).is_err());
    }
}
