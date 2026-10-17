use super::cms_client::{CmsClient, ListQuery};
use super::task_service::{Args, QUEUE_NAME};
use anyhow::bail;
use chrono::{Duration, Utc};
use kafru::cron_schedule::CronSchedule;
use kafru::database::Db;
use kafru::schedule::{Schedule, ScheduleData, ScheduleListConditions, ScheduleStatus};
use kafru::task::RecordId;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

const HANDLER: &str = "execute_task";
const SYNC_INTERVAL_SECS: u64 = 60;

/// 番組ごとの kafru スケジュール名
pub(crate) fn schedule_name(podcast_id: &str) -> String {
    format!("podcast:{podcast_id}")
}

/// Quartz 形式 7 フィールド (`秒 分 時 日 月 曜日 年`) の cron を `CronSchedule` にする
pub(crate) fn parse_cron(expr: &str) -> anyhow::Result<CronSchedule> {
    let f: Vec<&str> = expr.split_whitespace().collect();
    if f.len() != 7 {
        bail!("cron must have 7 fields (sec min hour day month weekday year): {expr}");
    }
    let mut cron = CronSchedule::new();
    cron.set_second(f[0].to_string());
    cron.set_minute(f[1].to_string());
    cron.set_hour(f[2].to_string());
    cron.set_day_of_month(f[3].to_string());
    cron.set_month(f[4].to_string());
    cron.set_day_of_week(f[5].to_string());
    cron.set_year(f[6].to_string());
    // 次回時刻を計算できない式 (不正なフィールド値) は弾く
    cron.clone()
        .get_upcoming(None)
        .map_err(|e| anyhow::anyhow!("invalid cron {expr}: {e}"))?;
    Ok(cron)
}

/// CMS の番組設定から求められるスケジュール
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DesiredSchedule {
    pub name: String,
    pub podcast_id: String,
    /// 正規化した cron (`CronSchedule::get_expression`)
    pub cron: String,
}

/// kafru に登録済みのスケジュール
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CurrentSchedule {
    pub id: String,
    pub name: String,
    pub cron: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ScheduleChange {
    Create(DesiredSchedule),
    Recreate { id: String, desired: DesiredSchedule },
    Remove { id: String },
}

/// `schedule.enabled` が true で `cron`・`script_id` が正しい番組のスケジュールを返す
pub(crate) fn desired_schedules(podcasts: &[Value]) -> Vec<DesiredSchedule> {
    podcasts
        .iter()
        .filter_map(|p| {
            let id = p["id"].as_str()?;
            let schedule = &p["data"]["schedule"];
            if schedule["enabled"].as_bool() != Some(true) || schedule["script_id"].as_str().is_none() {
                return None;
            }
            let cron = schedule["cron"].as_str()?;
            match parse_cron(cron) {
                Ok(c) => Some(DesiredSchedule {
                    name: schedule_name(id),
                    podcast_id: id.to_string(),
                    cron: c.get_expression(),
                }),
                Err(e) => {
                    tracing::warn!("skip schedule of podcast {id}: {e}");
                    None
                }
            }
        })
        .collect()
}

/// 求められるスケジュールと登録済みのスケジュールの差分
pub(crate) fn diff_schedules(
    desired: &[DesiredSchedule],
    current: &[CurrentSchedule],
) -> Vec<ScheduleChange> {
    let mut changes = vec![];
    for d in desired {
        match current.iter().find(|c| c.name == d.name) {
            None => changes.push(ScheduleChange::Create(d.clone())),
            Some(c) if c.cron != d.cron => changes.push(ScheduleChange::Recreate {
                id: c.id.clone(),
                desired: d.clone(),
            }),
            Some(_) => {}
        }
    }
    for c in current {
        if !desired.iter().any(|d| d.name == c.name) {
            changes.push(ScheduleChange::Remove { id: c.id.clone() });
        }
    }
    changes
}

/// CMS の `podcasts.data.schedule` を kafru のスケジュールへ同期する
pub(crate) struct ScheduleSync {
    cms: Arc<CmsClient>,
    schedule: Schedule<'static>,
}

impl ScheduleSync {
    pub(crate) async fn new(cms: Arc<CmsClient>, db: Arc<Db>) -> Self {
        Self {
            cms,
            schedule: Schedule::new(Some(db)).await,
        }
    }

    async fn current(&self) -> anyhow::Result<Vec<CurrentSchedule>> {
        let list = self
            .schedule
            .list(ScheduleListConditions {
                handler: Some(vec![HANDLER.to_string()]),
                upcoming: None,
                ..Default::default()
            })
            .await
            .map_err(anyhow::Error::msg)?;
        Ok(list
            .into_iter()
            .filter_map(|s| {
                let name = s.name?;
                if !name.starts_with("podcast:") {
                    return None;
                }
                Some(CurrentSchedule {
                    id: s.id?.to_string(),
                    name,
                    cron: s.cron_expression?.get_expression(),
                })
            })
            .collect())
    }

    async fn create(&self, d: &DesiredSchedule) -> anyhow::Result<()> {
        let args = serde_json::to_value(Args::GenerateEpisode {
            podcast_id: d.podcast_id.clone(),
        })?;
        self.schedule
            .create(ScheduleData {
                name: Some(d.name.clone()),
                queue: Some(QUEUE_NAME.to_string()),
                cron_expression: Some(parse_cron(&d.cron)?),
                handler: Some(HANDLER.to_string()),
                parameters: Some(HashMap::from([("args".to_string(), args)])),
                status: Some(ScheduleStatus::Enabled),
                // kafru は until_schedule >= 現在 のスケジュールだけを発火させる
                until_schedule: Some(Utc::now() + Duration::days(365 * 100)),
                ..Default::default()
            })
            .await
            .map_err(anyhow::Error::msg)?;
        Ok(())
    }

    async fn remove(&self, id: &str) -> anyhow::Result<()> {
        let id: RecordId = id
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid schedule id {id}: {e}"))?;
        self.schedule.remove(id).await.map_err(anyhow::Error::msg)?;
        Ok(())
    }

    /// 1 回分の同期
    pub(crate) async fn sync_once(&self) -> anyhow::Result<()> {
        let podcasts = self
            .cms
            .list_records("podcasts", &ListQuery::default())
            .await?;
        let changes = diff_schedules(&desired_schedules(&podcasts), &self.current().await?);
        for change in changes {
            tracing::info!("schedule change: {change:?}");
            match change {
                ScheduleChange::Create(d) => self.create(&d).await?,
                ScheduleChange::Recreate { id, desired } => {
                    self.remove(&id).await?;
                    self.create(&desired).await?;
                }
                ScheduleChange::Remove { id } => self.remove(&id).await?,
            }
        }
        Ok(())
    }

    /// 起動時と、その後 60 秒ごとに同期する
    pub(crate) async fn run(self) {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(SYNC_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = self.sync_once().await {
                tracing::warn!("schedule sync failed: {e:#}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_cron_accepts_seven_fields() {
        let cron = parse_cron("0 0 12 * * Fri *").unwrap();
        assert_eq!(cron.get_expression(), "0 0 12 * * Fri *");
    }

    #[test]
    fn parse_cron_rejects_wrong_field_count() {
        assert!(parse_cron("0 12 * * Fri").is_err());
    }

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
                podcast_id: "a".to_string(),
                cron: "0 0 12 * * Fri *".to_string(),
            }]
        );
    }

    fn desired(id: &str, cron: &str) -> DesiredSchedule {
        DesiredSchedule {
            name: schedule_name(id),
            podcast_id: id.to_string(),
            cron: cron.to_string(),
        }
    }

    fn current(id: &str, cron: &str) -> CurrentSchedule {
        CurrentSchedule {
            id: format!("kafru_schedules:{id}"),
            name: schedule_name(id),
            cron: cron.to_string(),
        }
    }

    #[test]
    fn diff_schedules_creates_recreates_and_removes() {
        let changes = diff_schedules(
            &[
                desired("new", "0 0 12 * * Fri *"),
                desired("same", "0 0 12 * * Fri *"),
                desired("changed", "0 0 13 * * Fri *"),
            ],
            &[
                current("same", "0 0 12 * * Fri *"),
                current("changed", "0 0 12 * * Fri *"),
                current("gone", "0 0 12 * * Fri *"),
            ],
        );
        assert_eq!(
            changes,
            vec![
                ScheduleChange::Create(desired("new", "0 0 12 * * Fri *")),
                ScheduleChange::Recreate {
                    id: "kafru_schedules:changed".to_string(),
                    desired: desired("changed", "0 0 13 * * Fri *"),
                },
                ScheduleChange::Remove { id: "kafru_schedules:gone".to_string() },
            ]
        );
    }
}
