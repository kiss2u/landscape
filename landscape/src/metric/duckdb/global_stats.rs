use chrono::{Duration as ChronoDuration, Local, LocalResult, NaiveTime, TimeZone};
use landscape_common::concurrency::{spawn_task, task_label};
use landscape_common::error::{LdError, LdResult};
use landscape_common::metric::connect::ConnectGlobalStats;

use super::connect::query as connect_query;
use super::store::DuckMetricStore;

const GLOBAL_STATS_REFRESH_HOUR: u32 = 0;
const GLOBAL_STATS_REFRESH_MINUTE: u32 = 5;

fn next_global_stats_refresh_at(now: chrono::DateTime<Local>) -> chrono::DateTime<Local> {
    let refresh_time =
        NaiveTime::from_hms_opt(GLOBAL_STATS_REFRESH_HOUR, GLOBAL_STATS_REFRESH_MINUTE, 0)
            .expect("valid global stats refresh time");
    let mut target_date = now.date_naive();
    if now.time() >= refresh_time {
        target_date = target_date.succ_opt().unwrap_or(target_date);
    }

    let target_naive = target_date.and_time(refresh_time);
    match Local.from_local_datetime(&target_naive) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(first, second) => first.min(second),
        LocalResult::None => now + ChronoDuration::hours(24),
    }
}

pub(crate) fn spawn_global_stats_refresh_task(store: DuckMetricStore) {
    spawn_task(task_label::task::METRIC_GLOBAL_STATS_REFRESH, async move {
        loop {
            let now = Local::now();
            let next_refresh_at = next_global_stats_refresh_at(now);
            let sleep_duration = (next_refresh_at - now)
                .to_std()
                .unwrap_or_else(|_| std::time::Duration::from_secs(60));
            tracing::info!(
                "next connect global stats refresh scheduled at {}",
                next_refresh_at.to_rfc3339()
            );
            tokio::time::sleep(sleep_duration).await;
            match store.refresh_global_stats_cache().await {
                Ok(stats) => tracing::info!(
                    "refreshed connect global stats cache at {} total_connect_count={}",
                    stats.last_calculate_time,
                    stats.total_connect_count
                ),
                Err(error) => {
                    tracing::error!("failed to refresh connect global stats cache: {}", error)
                }
            }
        }
    });
}

impl DuckMetricStore {
    fn cached_global_stats(&self) -> ConnectGlobalStats {
        self.global_stats_cache.read().expect("metric global stats cache poisoned").clone()
    }

    pub(crate) async fn refresh_global_stats_cache(&self) -> LdResult<ConnectGlobalStats> {
        let refresh_requested_at =
            landscape_common::utils::time::get_current_time_ms().unwrap_or_default();
        let _refresh_guard = self.global_stats_refresh_lock.lock().await;
        let cached = self.cached_global_stats();
        if cached.last_calculate_time >= refresh_requested_at && cached.last_calculate_time != 0 {
            return Ok(cached);
        }

        let stats = self
            .run_query_result(task_label::op::METRIC_GLOBAL_STATS, move |store| {
                let conn = store.get_disk_conn();
                connect_query::query_global_stats(&conn).map_err(|error| {
                    LdError::DbMsg(format!("failed to refresh connect global stats: {error}"))
                })
            })
            .await?;

        let mut cache =
            self.global_stats_cache.write().expect("metric global stats cache poisoned");
        *cache = stats.clone();
        Ok(stats)
    }

    pub async fn get_global_stats(&self, force_refresh: bool) -> LdResult<ConnectGlobalStats> {
        if force_refresh {
            self.refresh_global_stats_cache().await
        } else {
            Ok(self.cached_global_stats())
        }
    }
}
