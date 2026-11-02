//! CMS の番組の `data.schedule` を kafru のスケジュールへ同期する（[`agent_kit::schedule`]）

use super::cms_client::{CmsClient, ListQuery};
use super::episode_generation::ScheduleConfig;
use super::task_service::{Args, QUEUE_NAME};
use agent_kit::schedule::{parse_cron, DesiredSchedule, ScheduleSource, ScheduleSync, ScheduleTarget};
use async_trait::async_trait;
use kafru::database::Db;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};

const HANDLER: &str = "execute_task";
const SYNC_INTERVAL_SECS: u64 = 60;
const PREFIX: &str = "podcast:";

/// 番組ごとの kafru スケジュール名
pub(crate) fn schedule_name(podcast_id: &str) -> String {
    format!("{PREFIX}{podcast_id}")
}

/// `schedule.enabled` が true で `cron` が正しく、生成に必要な値（script なら `script_id`、
/// llm なら `prompt`）がある番組のスケジュールを返す。発火すると `GenerateEpisode` が動く
pub(crate) fn desired_schedules(podcasts: &[Value]) -> Vec<DesiredSchedule> {
    podcasts
        .iter()
        .filter_map(|p| {
            let id = p["id"].as_str()?;
            let schedule = &p["data"]["schedule"];
            if schedule["enabled"].as_bool() != Some(true) {
                return None;
            }
            match ScheduleConfig::from_podcast(p) {
                Ok(config) if config.is_runnable() => {}
                Ok(_) => return None,
                Err(e) => {
                    tracing::warn!("skip schedule of podcast {id}: {e:#}");
                    return None;
                }
            }
            let cron = schedule["cron"].as_str()?;
            let cron = match parse_cron(cron) {
                Ok(c) => c.get_expression(),
                Err(e) => {
                    tracing::warn!("skip schedule of podcast {id}: {e}");
                    return None;
                }
            };
            let args = serde_json::to_value(Args::GenerateEpisode {
                podcast_id: id.to_string(),
            })
            .ok()?;
            Some(DesiredSchedule {
                name: schedule_name(id),
                cron,
                parameters: HashMap::from([("args".to_string(), args)]),
            })
        })
        .collect()
}

/// CMS の `podcasts` を読む [`ScheduleSource`]
pub(crate) struct PodcastSchedules {
    cms: Arc<CmsClient>,
}

#[async_trait]
impl ScheduleSource for PodcastSchedules {
    fn prefix(&self) -> &str {
        PREFIX
    }

    async fn desired(&self) -> anyhow::Result<Vec<DesiredSchedule>> {
        let podcasts = self
            .cms
            .list_records("podcasts", &ListQuery::default())
            .await?;
        Ok(desired_schedules(&podcasts))
    }
}

/// 番組のスケジュールを 60 秒ごとに同期する [`ScheduleSync`]
pub(crate) async fn podcast_schedule_sync(
    cms: Arc<CmsClient>,
    db: Arc<Db>,
) -> ScheduleSync<PodcastSchedules> {
    ScheduleSync::new(
        PodcastSchedules { cms },
        db,
        ScheduleTarget {
            queue: QUEUE_NAME.to_string(),
            handler: HANDLER.to_string(),
        },
        Duration::from_secs(SYNC_INTERVAL_SECS),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn podcast(id: &str, schedule: serde_json::Value) -> serde_json::Value {
        json!({ "id": id, "data": { "title": "t", "schedule": schedule } })
    }

    #[test]
    fn desired_schedules_picks_enabled_valid_ones() {
        let podcasts = vec![
            podcast("a", json!({ "cron": "0 0 12 * * Fri *", "script_id": "s", "enabled": true })),
            podcast("b", json!({ "cron": "0 0 12 * * Fri *", "script_id": "s", "enabled": false })),
            podcast("c", json!({ "cron": "bad", "script_id": "s", "enabled": true })),
            podcast("d", json!({ "cron": "0 0 12 * * Fri *", "enabled": true })),
            json!({ "id": "e", "data": { "title": "t" } }),
        ];
        assert_eq!(
            desired_schedules(&podcasts),
            vec![DesiredSchedule {
                name: "podcast:a".to_string(),
                cron: "0 0 12 * * Fri *".to_string(),
                parameters: HashMap::from([(
                    "args".to_string(),
                    serde_json::to_value(Args::GenerateEpisode { podcast_id: "a".to_string() }).unwrap(),
                )]),
            }]
        );
    }

    #[test]
    fn desired_schedules_accepts_llm_mode_without_script() {
        let podcasts = vec![
            podcast("llm", json!({ "cron": "0 0 22 * * Sun *", "enabled": true, "generator": "llm", "prompt": "今週の記事を紹介して" })),
            // llm で prompt が無い・空なら動かせない
            podcast("noprompt", json!({ "cron": "0 0 22 * * Sun *", "enabled": true, "generator": "llm" })),
            podcast("blank", json!({ "cron": "0 0 22 * * Sun *", "enabled": true, "generator": "llm", "prompt": "  " })),
            // 明示した script は今までどおり script_id が要る
            podcast("script", json!({ "cron": "0 0 22 * * Sun *", "enabled": true, "generator": "script" })),
            podcast("unknown", json!({ "cron": "0 0 22 * * Sun *", "enabled": true, "generator": "x", "script_id": "s" })),
        ];
        let names: Vec<String> = desired_schedules(&podcasts).into_iter().map(|d| d.name).collect();
        assert_eq!(names, vec!["podcast:llm".to_string()]);
    }
}
