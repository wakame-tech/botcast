//! アプリの定期実行の設定（[`ScheduleSource`]）を kafru のスケジュールへ同期する。
//!
//! スケジュールは名前の接頭辞（例: botcast の `podcast:`）でアプリ・種類ごとに分け、
//! 同期はその接頭辞のスケジュールだけを作成・作り直し・削除する。
//! 比べるのは cron だけなので、ジョブの引数には ID だけを入れ、設定の本体は実行時に読み直す。

use anyhow::bail;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use kafru::cron_schedule::CronSchedule;
use kafru::database::Db;
use kafru::schedule::{Schedule, ScheduleData, ScheduleListConditions, ScheduleStatus};
use kafru::task::RecordId;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

/// Quartz 形式 7 フィールド (`秒 分 時 日 月 曜日 年`) の cron を `CronSchedule` にする
pub fn parse_cron(expr: &str) -> anyhow::Result<CronSchedule> {
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

/// アプリの設定から求められるスケジュール
#[derive(Debug, Clone, PartialEq)]
pub struct DesiredSchedule {
    /// [`ScheduleSource::prefix`] で始まる名前
    pub name: String,
    /// 正規化した cron (`CronSchedule::get_expression`)
    pub cron: String,
    /// 発火したジョブの引数（`QueueData.parameters`）
    pub parameters: HashMap<String, Value>,
}

/// kafru に登録済みのスケジュール
#[derive(Debug, Clone, PartialEq)]
pub struct CurrentSchedule {
    pub id: String,
    pub name: String,
    pub cron: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleChange {
    Create(DesiredSchedule),
    Recreate { id: String, desired: DesiredSchedule },
    Remove { id: String },
}

/// 求められるスケジュールと登録済みのスケジュールの差分
pub fn diff_schedules(
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

/// 定期実行の設定の読み先（botcast は CMS の番組）
#[async_trait]
pub trait ScheduleSource: Send + Sync {
    /// このソースが持つスケジュールの名前の接頭辞。同期はこの接頭辞のスケジュールだけを扱う
    fn prefix(&self) -> &str;

    /// 今あるべきスケジュール。名前は [`Self::prefix`] で始め、cron は [`parse_cron`] で正規化しておく
    async fn desired(&self) -> anyhow::Result<Vec<DesiredSchedule>>;
}

/// 発火したスケジュールの積み先
#[derive(Debug, Clone)]
pub struct ScheduleTarget {
    pub queue: String,
    pub handler: String,
}

/// [`ScheduleSource`] を kafru のスケジュールへ同期する
pub struct ScheduleSync<S> {
    source: S,
    target: ScheduleTarget,
    interval: std::time::Duration,
    schedule: Schedule<'static>,
}

impl<S: ScheduleSource> ScheduleSync<S> {
    pub async fn new(
        source: S,
        db: Arc<Db>,
        target: ScheduleTarget,
        interval: std::time::Duration,
    ) -> Self {
        Self {
            source,
            target,
            interval,
            schedule: Schedule::new(Some(db)).await,
        }
    }

    async fn current(&self) -> anyhow::Result<Vec<CurrentSchedule>> {
        let list = self
            .schedule
            .list(ScheduleListConditions {
                handler: Some(vec![self.target.handler.clone()]),
                upcoming: None,
                ..Default::default()
            })
            .await
            .map_err(anyhow::Error::msg)?;
        let prefix = self.source.prefix();
        Ok(list
            .into_iter()
            .filter_map(|s| {
                let name = s.name?;
                if !name.starts_with(prefix) {
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
        self.schedule
            .create(ScheduleData {
                name: Some(d.name.clone()),
                queue: Some(self.target.queue.clone()),
                cron_expression: Some(parse_cron(&d.cron)?),
                handler: Some(self.target.handler.clone()),
                parameters: Some(d.parameters.clone()),
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
    pub async fn sync_once(&self) -> anyhow::Result<()> {
        let desired = self.source.desired().await?;
        let changes = diff_schedules(&desired, &self.current().await?);
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

    /// 起動時と、その後 `interval` ごとに同期する
    pub async fn run(self) {
        let mut interval = tokio::time::interval(self.interval);
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

    #[test]
    fn parse_cron_accepts_seven_fields() {
        let cron = parse_cron("0 0 12 * * Fri *").unwrap();
        assert_eq!(cron.get_expression(), "0 0 12 * * Fri *");
    }

    #[test]
    fn parse_cron_rejects_wrong_field_count() {
        assert!(parse_cron("0 12 * * Fri").is_err());
    }

    fn desired(id: &str, cron: &str) -> DesiredSchedule {
        DesiredSchedule {
            name: format!("test:{id}"),
            cron: cron.to_string(),
            parameters: HashMap::new(),
        }
    }

    fn current(id: &str, cron: &str) -> CurrentSchedule {
        CurrentSchedule {
            id: format!("kafru_schedules:{id}"),
            name: format!("test:{id}"),
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
