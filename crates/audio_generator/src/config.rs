//! TTS エンジンの設定（環境変数）。
//!
//! ```text
//! TTS_ENGINES=voicevox,irodori                 # 使うエンジンの名前（カンマ区切り）
//! TTS_DEFAULT_ENGINE=voicevox                  # 旧形式の speaker に使うエンジン（既定: TTS_ENGINES の先頭）
//! TTS_ENGINE_<NAME>_KIND=voicevox|openai       # 必須
//! TTS_ENGINE_<NAME>_ENDPOINT=http://...        # 必須
//! TTS_ENGINE_<NAME>_TOKEN=...                  # 任意。bearer トークン（OpenAI 互換）
//! TTS_ENGINE_<NAME>_MODEL=...                  # 任意。OpenAI 互換の model（既定 irodori-tts）
//! TTS_ENGINE_<NAME>_MAX_CONCURRENCY=1          # 任意。同時に投げる数
//! TTS_ENGINE_<NAME>_TIMEOUT_SEC=120            # 任意
//! TTS_CACHE_DIR=...                            # 任意。合成結果のキャッシュの置き場（既定: ワークディレクトリ）
//! ```
//!
//! `<NAME>` はエンジン名を大文字にし `-` を `_` にしたもの。`TTS_ENGINES` が無く `VOICEVOX_ENDPOINT` が
//! あれば、`voicevox` だけのエンジンとして扱う（互換）。空文字の環境変数は未設定とみなす。

use anyhow::{bail, Context, Result};
use std::{path::PathBuf, time::Duration};

pub const DEFAULT_MAX_CONCURRENCY: usize = 1;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
/// `VOICEVOX_ENDPOINT` の互換で作るエンジンの名前
pub const LEGACY_VOICEVOX_ENGINE: &str = "voicevox";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineKind {
    /// `/audio_query` → `/synthesis`（VOICEVOX・AivisSpeech など）
    Voicevox,
    /// `POST /audio/speech`（Irodori-TTS-Server・OpenAI など）
    OpenAi,
}

impl std::str::FromStr for EngineKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "voicevox" => Ok(Self::Voicevox),
            "openai" => Ok(Self::OpenAi),
            other => bail!("unknown engine kind {other:?} (voicevox | openai)"),
        }
    }
}

/// ログやトレースに出さない値。
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(***)")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EngineConfig {
    pub name: String,
    pub kind: EngineKind,
    /// 末尾の `/` は除く
    pub endpoint: String,
    pub token: Option<Secret>,
    pub model: Option<String>,
    pub max_concurrency: usize,
    pub timeout: Duration,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TtsConfig {
    pub engines: Vec<EngineConfig>,
    /// エンジンが 1 つも無いときは `None`
    pub default_engine: Option<String>,
    pub cache_dir: Option<PathBuf>,
}

fn env_key(name: &str) -> String {
    name.to_ascii_uppercase().replace('-', "_")
}

impl TtsConfig {
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// `lookup` で環境変数を読んで設定を作り、検証する。
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let get = |key: &str| {
            lookup(key)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        let cache_dir = get("TTS_CACHE_DIR").map(PathBuf::from);

        let (names, legacy_endpoint) = match get("TTS_ENGINES") {
            Some(list) => (
                list.split(',')
                    .map(|n| n.trim().to_string())
                    .filter(|n| !n.is_empty())
                    .collect::<Vec<_>>(),
                None,
            ),
            None => match get("VOICEVOX_ENDPOINT") {
                Some(endpoint) => (vec![LEGACY_VOICEVOX_ENGINE.to_string()], Some(endpoint)),
                None => (vec![], None),
            },
        };

        let mut engines: Vec<EngineConfig> = vec![];
        for name in names {
            if !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                bail!("TTS_ENGINES: invalid engine name {name:?} (use ASCII letters, digits, '-' or '_')");
            }
            let key = env_key(&name);
            if engines.iter().any(|e| env_key(&e.name) == key) {
                bail!("TTS_ENGINES: duplicated engine name {name:?}");
            }
            let var = |suffix: &str| format!("TTS_ENGINE_{key}_{suffix}");

            let (kind, endpoint) = match &legacy_endpoint {
                Some(endpoint) => (EngineKind::Voicevox, endpoint.clone()),
                None => {
                    let kind = get(&var("KIND"))
                        .with_context(|| format!("{} is required", var("KIND")))?
                        .parse::<EngineKind>()
                        .with_context(|| format!("invalid {}", var("KIND")))?;
                    let endpoint = get(&var("ENDPOINT"))
                        .with_context(|| format!("{} is required", var("ENDPOINT")))?;
                    (kind, endpoint)
                }
            };
            let endpoint_var = if legacy_endpoint.is_some() {
                "VOICEVOX_ENDPOINT".to_string()
            } else {
                var("ENDPOINT")
            };
            // 値に認証情報が入っていることがあるので、エラーに値は出さない
            let url = reqwest::Url::parse(&endpoint)
                .with_context(|| format!("invalid {endpoint_var} (not a URL)"))?;
            if !matches!(url.scheme(), "http" | "https") {
                bail!("invalid {endpoint_var} (http or https only)");
            }
            // URL の認証情報はエラーやログに出やすいので受け付けない（トークンは TOKEN に書く）
            if !url.username().is_empty() || url.password().is_some() {
                bail!(
                    "invalid {endpoint_var} (credentials in URL are not allowed; use {} for a bearer token)",
                    var("TOKEN")
                );
            }

            let max_concurrency = match get(&var("MAX_CONCURRENCY")) {
                Some(v) => match v.parse::<usize>() {
                    Ok(n) if n >= 1 => n,
                    _ => bail!(
                        "{} must be a positive integer: {v:?}",
                        var("MAX_CONCURRENCY")
                    ),
                },
                None => DEFAULT_MAX_CONCURRENCY,
            };
            let timeout = match get(&var("TIMEOUT_SEC")) {
                Some(v) => match v.parse::<u64>() {
                    Ok(n) if n >= 1 => Duration::from_secs(n),
                    _ => bail!("{} must be a positive integer: {v:?}", var("TIMEOUT_SEC")),
                },
                None => DEFAULT_TIMEOUT,
            };

            engines.push(EngineConfig {
                name,
                kind,
                endpoint: endpoint.trim_end_matches('/').to_string(),
                token: get(&var("TOKEN")).map(Secret::new),
                model: get(&var("MODEL")),
                max_concurrency,
                timeout,
            });
        }

        let default_engine = match get("TTS_DEFAULT_ENGINE") {
            Some(name) => {
                if !engines.iter().any(|e| e.name == name) {
                    bail!("TTS_DEFAULT_ENGINE {name:?} is not in TTS_ENGINES");
                }
                Some(name)
            }
            None => engines.first().map(|e| e.name.clone()),
        };

        Ok(Self {
            engines,
            default_engine,
            cache_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn load(vars: &[(&str, &str)]) -> Result<TtsConfig> {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        TtsConfig::from_lookup(|k| vars.get(k).cloned())
    }

    #[test]
    fn legacy_voicevox_endpoint() {
        let config = load(&[("VOICEVOX_ENDPOINT", "http://localhost:50021/")]).unwrap();
        assert_eq!(
            config,
            TtsConfig {
                engines: vec![EngineConfig {
                    name: "voicevox".to_string(),
                    kind: EngineKind::Voicevox,
                    endpoint: "http://localhost:50021".to_string(),
                    token: None,
                    model: None,
                    max_concurrency: 1,
                    timeout: Duration::from_secs(120),
                }],
                default_engine: Some("voicevox".to_string()),
                cache_dir: None,
            }
        );
    }

    #[test]
    fn no_engines_is_allowed() {
        let config = load(&[("VOICEVOX_ENDPOINT", "")]).unwrap();
        assert!(config.engines.is_empty());
        assert_eq!(config.default_engine, None);
    }

    #[test]
    fn multiple_engines() {
        let config = load(&[
            ("TTS_ENGINES", "voicevox, irodori-v3"),
            ("TTS_DEFAULT_ENGINE", "irodori-v3"),
            ("TTS_ENGINE_VOICEVOX_KIND", "voicevox"),
            ("TTS_ENGINE_VOICEVOX_ENDPOINT", "http://vv:50021"),
            ("TTS_ENGINE_IRODORI_V3_KIND", "openai"),
            ("TTS_ENGINE_IRODORI_V3_ENDPOINT", "http://gpu:8088/v1"),
            ("TTS_ENGINE_IRODORI_V3_TOKEN", "secret-token"),
            ("TTS_ENGINE_IRODORI_V3_MODEL", "irodori-tts"),
            ("TTS_ENGINE_IRODORI_V3_MAX_CONCURRENCY", "2"),
            ("TTS_ENGINE_IRODORI_V3_TIMEOUT_SEC", "300"),
            ("TTS_CACHE_DIR", "/var/cache/tts"),
            // TTS_ENGINES があるときは VOICEVOX_ENDPOINT を読まない
            ("VOICEVOX_ENDPOINT", "http://ignored:50021"),
        ])
        .unwrap();
        assert_eq!(config.engines.len(), 2);
        assert_eq!(config.engines[0].endpoint, "http://vv:50021");
        let irodori = &config.engines[1];
        assert_eq!(irodori.name, "irodori-v3");
        assert_eq!(irodori.kind, EngineKind::OpenAi);
        assert_eq!(irodori.token, Some(Secret::new("secret-token")));
        assert_eq!(irodori.model.as_deref(), Some("irodori-tts"));
        assert_eq!(irodori.max_concurrency, 2);
        assert_eq!(irodori.timeout, Duration::from_secs(300));
        assert_eq!(config.default_engine.as_deref(), Some("irodori-v3"));
        assert_eq!(config.cache_dir, Some(PathBuf::from("/var/cache/tts")));
    }

    #[test]
    fn token_is_not_in_debug() {
        let config = load(&[
            ("TTS_ENGINES", "irodori"),
            ("TTS_ENGINE_IRODORI_KIND", "openai"),
            ("TTS_ENGINE_IRODORI_ENDPOINT", "http://gpu:8088/v1"),
            ("TTS_ENGINE_IRODORI_TOKEN", "secret-token"),
        ])
        .unwrap();
        let debug = format!("{config:?}");
        assert!(!debug.contains("secret-token"), "{debug}");
        assert_eq!(config.default_engine.as_deref(), Some("irodori"));
    }

    #[test]
    fn invalid_configs() {
        let base = [
            ("TTS_ENGINES", "vv"),
            ("TTS_ENGINE_VV_KIND", "voicevox"),
            ("TTS_ENGINE_VV_ENDPOINT", "http://vv:50021"),
        ];
        let with = |extra: &[(&'static str, &'static str)]| {
            let mut vars = base.to_vec();
            for (k, v) in extra {
                vars.retain(|(key, _)| key != k);
                vars.push((k, v));
            }
            load(&vars).unwrap_err().to_string()
        };
        assert!(with(&[("TTS_ENGINE_VV_KIND", "")]).contains("TTS_ENGINE_VV_KIND is required"));
        assert!(with(&[("TTS_ENGINE_VV_KIND", "foo")]).contains("invalid TTS_ENGINE_VV_KIND"));
        assert!(
            with(&[("TTS_ENGINE_VV_ENDPOINT", "")]).contains("TTS_ENGINE_VV_ENDPOINT is required")
        );
        assert!(with(&[("TTS_ENGINE_VV_ENDPOINT", "vv:50021")]).contains("TTS_ENGINE_VV_ENDPOINT"));
        let err = with(&[("TTS_ENGINE_VV_ENDPOINT", "http://user:pass@vv:50021")]);
        assert!(err.contains("credentials in URL are not allowed"), "{err}");
        assert!(!err.contains("pass"), "{err}");
        assert!(with(&[("TTS_ENGINE_VV_ENDPOINT", "http://user@vv:50021")]).contains("credentials"));
        assert!(with(&[("TTS_ENGINE_VV_MAX_CONCURRENCY", "0")]).contains("MAX_CONCURRENCY"));
        assert!(with(&[("TTS_ENGINE_VV_TIMEOUT_SEC", "abc")]).contains("TIMEOUT_SEC"));
        assert!(with(&[("TTS_DEFAULT_ENGINE", "other")]).contains("TTS_DEFAULT_ENGINE"));
        assert!(with(&[("TTS_ENGINES", "vv,vv")]).contains("duplicated"));
        assert!(with(&[("TTS_ENGINES", "v v")]).contains("invalid engine name"));
    }
}
