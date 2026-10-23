//! 台本生成などで使う LLM のモデルを決める。
//!
//! モデルの一覧は CMS の `llm_models`、ユーザーの選択は `user_settings.llm_model`（`llm_models` のレコード ID）。
//! 決める順番: ジョブの持ち主（番組の `owner_id`）の設定 → `llm_models` の既定 → どちらも無ければエラー。
//! 設定のモデルが無効（`enabled: false`）や一覧に無ければ、既定にフォールバックしてログと span に出す。

use super::cms_client::{CmsClient, ListQuery};
use anyhow::Context;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use tracing::field::Empty;

pub(crate) const LLM_MODELS: &str = "llm_models";
pub(crate) const USER_SETTINGS: &str = "user_settings";

/// 一覧を 1 度に取得する数（モデルはこれより少ない前提）
const LLM_MODELS_LIMIT: u32 = 100;

/// `llm_models` のレコード
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LlmModel {
    /// レコード ID（`user_settings.llm_model` が参照する）
    #[serde(skip)]
    pub id: String,
    pub model_id: String,
    #[serde(default)]
    pub default: bool,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub order: i64,
}

impl LlmModel {
    fn from_record(record: &Value) -> anyhow::Result<Self> {
        let mut model: Self =
            serde_json::from_value(record["data"].clone()).context("invalid llm_models record")?;
        model.id = record["id"]
            .as_str()
            .context("llm_models record has no id")?
            .to_string();
        Ok(model)
    }
}

/// モデルをどこから決めたか（span・ログに出す）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelSource {
    /// ユーザーの設定
    UserSetting,
    /// 設定が無いので既定
    Default,
    /// 設定のモデルが使えないので既定
    Fallback,
}

impl ModelSource {
    fn as_str(self) -> &'static str {
        match self {
            ModelSource::UserSetting => "user_setting",
            ModelSource::Default => "default",
            ModelSource::Fallback => "fallback",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelChoice {
    pub model_id: String,
    pub source: ModelSource,
}

/// 一覧と設定（`llm_models` のレコード ID）からモデルを決める
pub(crate) fn choose_model(
    models: &[LlmModel],
    setting: Option<&str>,
) -> anyhow::Result<ModelChoice> {
    if let Some(setting) = setting {
        if let Some(m) = models.iter().find(|m| m.id == setting && m.enabled) {
            return Ok(ModelChoice {
                model_id: m.model_id.clone(),
                source: ModelSource::UserSetting,
            });
        }
    }
    let default = models
        .iter()
        .filter(|m| m.default && m.enabled)
        .min_by_key(|m| m.order)
        .context("no default LLM model: add an enabled model with default: true to the CMS collection 'llm_models'")?;
    Ok(ModelChoice {
        model_id: default.model_id.clone(),
        source: if setting.is_some() {
            ModelSource::Fallback
        } else {
            ModelSource::Default
        },
    })
}

#[async_trait]
pub(crate) trait LlmModelStore: Send + Sync {
    async fn list_llm_models(&self) -> anyhow::Result<Vec<LlmModel>>;
    /// ユーザーの設定のモデル（`llm_models` のレコード ID）。設定が無ければ `None`
    async fn user_llm_model(&self, owner: &str) -> anyhow::Result<Option<String>>;
}

#[async_trait]
impl LlmModelStore for CmsClient {
    async fn list_llm_models(&self) -> anyhow::Result<Vec<LlmModel>> {
        let records = self
            .list_records(
                LLM_MODELS,
                &ListQuery {
                    limit: Some(LLM_MODELS_LIMIT),
                    ..Default::default()
                },
            )
            .await?;
        records.iter().map(LlmModel::from_record).collect()
    }

    async fn user_llm_model(&self, owner: &str) -> anyhow::Result<Option<String>> {
        // user_settings は read: owner なので、持ち主として読むとその人の設定だけになる
        let records = self
            .list_records_as(
                USER_SETTINGS,
                &ListQuery {
                    sort: Some("created_at".to_string()),
                    order_desc: true,
                    limit: Some(1),
                    ..Default::default()
                },
                Some(owner),
            )
            .await?;
        Ok(records
            .first()
            .and_then(|r| r["data"]["llm_model"].as_str())
            .map(str::to_string))
    }
}

/// ジョブの持ち主（番組の `owner_id`。`user:` 以外は設定を持たない）のモデルを決める
#[tracing::instrument(
    name = "llm.resolve_model",
    skip_all,
    fields(
        gen_ai.request.model = Empty,
        llm.model_source = Empty,
        llm.requested_model = Empty,
    ),
    err
)]
pub(crate) async fn resolve_model(
    store: &dyn LlmModelStore,
    owner: Option<&str>,
) -> anyhow::Result<String> {
    let models = store
        .list_llm_models()
        .await
        .context("Failed to list LLM models")?;
    let setting = match owner.filter(|o| o.starts_with("user:")) {
        Some(owner) => store
            .user_llm_model(owner)
            .await
            .context("Failed to fetch user settings")?,
        None => None,
    };
    let choice = choose_model(&models, setting.as_deref())?;
    let span = tracing::Span::current();
    span.record("gen_ai.request.model", choice.model_id.as_str());
    span.record("llm.model_source", choice.source.as_str());
    if choice.source == ModelSource::Fallback {
        let requested = setting.as_deref().unwrap_or_default();
        span.record("llm.requested_model", requested);
        tracing::warn!(
            "LLM model {requested:?} in user settings is disabled or unknown; using default {}",
            choice.model_id
        );
    }
    Ok(choice.model_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Mutex;

    fn model(id: &str, model_id: &str, default: bool, enabled: bool, order: i64) -> LlmModel {
        LlmModel::from_record(&json!({ "id": id, "data": {
            "model_id": model_id, "name": model_id, "provider": "anthropic",
            "default": default, "enabled": enabled, "order": order
        }}))
        .unwrap()
    }

    fn models() -> Vec<LlmModel> {
        vec![
            model("m1", "claude-opus-5-5", true, true, 0),
            model("m2", "claude-sonnet-5-5", false, true, 1),
            model("m3", "claude-haiku-5-5", false, false, 2),
        ]
    }

    #[test]
    fn uses_the_user_setting() {
        assert_eq!(
            choose_model(&models(), Some("m2")).unwrap(),
            ModelChoice {
                model_id: "claude-sonnet-5-5".into(),
                source: ModelSource::UserSetting
            }
        );
    }

    #[test]
    fn uses_the_default_without_setting() {
        assert_eq!(
            choose_model(&models(), None).unwrap(),
            ModelChoice {
                model_id: "claude-opus-5-5".into(),
                source: ModelSource::Default
            }
        );
    }

    #[test]
    fn falls_back_to_the_default_for_disabled_or_unknown_models() {
        for setting in ["m3", "gone"] {
            assert_eq!(
                choose_model(&models(), Some(setting)).unwrap(),
                ModelChoice {
                    model_id: "claude-opus-5-5".into(),
                    source: ModelSource::Fallback
                }
            );
        }
    }

    #[test]
    fn fails_without_a_usable_default() {
        let err = choose_model(&[], None).unwrap_err().to_string();
        assert!(err.contains("llm_models"), "{err}");
        // 無効な既定は使わない
        let disabled = vec![model("m1", "claude-opus-5-5", true, false, 0)];
        assert!(choose_model(&disabled, Some("m1")).is_err());
    }

    #[test]
    fn picks_the_first_default_by_order() {
        let models = vec![
            model("a", "late", true, true, 5),
            model("b", "early", true, true, 1),
        ];
        assert_eq!(choose_model(&models, None).unwrap().model_id, "early");
    }

    struct FakeStore {
        models: Vec<LlmModel>,
        setting: Option<String>,
        asked: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl LlmModelStore for FakeStore {
        async fn list_llm_models(&self) -> anyhow::Result<Vec<LlmModel>> {
            Ok(self.models.clone())
        }
        async fn user_llm_model(&self, owner: &str) -> anyhow::Result<Option<String>> {
            self.asked.lock().unwrap().push(owner.to_string());
            Ok(self.setting.clone())
        }
    }

    #[tokio::test]
    async fn resolves_from_the_owner_settings() {
        let store = FakeStore {
            models: models(),
            setting: Some("m2".into()),
            asked: Default::default(),
        };
        assert_eq!(
            resolve_model(&store, Some("user:a")).await.unwrap(),
            "claude-sonnet-5-5"
        );
        assert_eq!(*store.asked.lock().unwrap(), vec!["user:a".to_string()]);
        // サービス所有・持ち主不明の番組は設定を読まずに既定
        let store = FakeStore {
            models: models(),
            setting: Some("m2".into()),
            asked: Default::default(),
        };
        assert_eq!(
            resolve_model(&store, Some("service")).await.unwrap(),
            "claude-opus-5-5"
        );
        assert_eq!(
            resolve_model(&store, None).await.unwrap(),
            "claude-opus-5-5"
        );
        assert!(store.asked.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn fails_when_the_list_is_empty() {
        let store = FakeStore {
            models: vec![],
            setting: None,
            asked: Default::default(),
        };
        assert!(resolve_model(&store, Some("user:a")).await.is_err());
    }
}
