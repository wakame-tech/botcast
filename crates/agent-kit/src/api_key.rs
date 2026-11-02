//! LLM の API キー。値はログ・span・エラーに出さない

use std::fmt;

/// LLM の API キー。`Debug` / `Display` は値を出さない
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// リクエストのヘッダーに入れるときだけ使う
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(<redacted>)")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}
