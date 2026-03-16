use std::{
    mem,
    sync::{Arc, RwLock},
    thread,
    time::{Duration as StdDuration, Instant as StdInstant, SystemTime, UNIX_EPOCH},
};

use crate::{
    config::TimeRuntimeConfig, sys_config, DEFAULT_TIME_ENABLE, DEFAULT_TIME_FALLBACK_SERVER,
};
use libc::{clock_gettime, timespec, CLOCK_BOOTTIME, CLOCK_MONOTONIC};
use once_cell::sync::{Lazy, OnceCell};
use serde::Serialize;
use tokio::time::{Duration, Instant};

const NTP_UNIX_OFFSET_SECS: u64 = ((70_u64 * 365) + 17) * 24 * 60 * 60;

#[derive(Clone, Copy)]
struct SyncedTimeState {
    base_time: SystemTime,
    base_instant: StdInstant,
}

impl SyncedTimeState {
    fn now() -> Self {
        Self {
            base_time: SystemTime::now(),
            base_instant: StdInstant::now(),
        }
    }

    fn from_system_time(base_time: SystemTime) -> Self {
        Self { base_time, base_instant: StdInstant::now() }
    }

    fn current_time(&self) -> SystemTime {
        self.base_time + self.base_instant.elapsed()
    }
}

static SHARED_TIME: Lazy<Arc<RwLock<SyncedTimeState>>> =
    Lazy::new(|| Arc::new(RwLock::new(SyncedTimeState::now())));
static TIME_SYNC_THREAD: OnceCell<()> = OnceCell::new();
static TIME_SYNC_CONFIG: Lazy<Arc<RwLock<TimeRuntimeConfig>>> =
    Lazy::new(|| Arc::new(RwLock::new(TimeRuntimeConfig::default())));
static TIME_SYNC_STATUS: Lazy<Arc<RwLock<TimeSyncStatus>>> = Lazy::new(|| {
    Arc::new(RwLock::new(TimeSyncStatus {
        enabled: DEFAULT_TIME_ENABLE,
        running: false,
        current_source: "system".to_string(),
        sync_stage: "startup".to_string(),
        last_action: "startup".to_string(),
        ..Default::default()
    }))
});

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TimeSyncStatus {
    pub enabled: bool,
    pub running: bool,
    pub current_source: String,
    pub sync_stage: String,
    pub last_action: String,
    pub last_attempt_at: Option<f64>,
    pub last_success_at: Option<f64>,
    pub last_system_clock_update_at: Option<f64>,
    pub last_server: Option<String>,
    pub last_offset_ms: Option<f64>,
    pub last_delay_ms: Option<f64>,
    pub selected_sample_count: Option<u8>,
    pub last_error: Option<String>,
    pub system_clock_synced: bool,
}

#[derive(Clone, Debug)]
struct NtpQueryResult {
    synced_time: SystemTime,
    server: String,
    offset_ms: f64,
    delay_ms: f64,
    sample_count: u8,
}

pub struct LdCountdown {
    start: Instant,
    duration: Duration,
}

fn shared_time() -> Arc<RwLock<SyncedTimeState>> {
    Arc::clone(&SHARED_TIME)
}

fn shared_time_sync_status() -> Arc<RwLock<TimeSyncStatus>> {
    Arc::clone(&TIME_SYNC_STATUS)
}

fn shared_time_sync_config() -> Arc<RwLock<TimeRuntimeConfig>> {
    Arc::clone(&TIME_SYNC_CONFIG)
}

pub fn get_time_sync_status() -> TimeSyncStatus {
    shared_time_sync_status().read().map(|status| status.clone()).unwrap_or_default()
}

fn normalized_time_config(mut config: TimeRuntimeConfig) -> TimeRuntimeConfig {
    config.sync_interval_secs = config.sync_interval_secs.max(1);
    config.timeout_secs = config.timeout_secs.max(1);
    config.samples_per_server = config.samples_per_server.max(1);
    config
}

fn update_shared_time(time: SystemTime) {
    if let Ok(mut current_time) = shared_time().write() {
        *current_time = SyncedTimeState::from_system_time(time);
    }
}

fn mirror_system_time(
    config: &TimeRuntimeConfig,
    status_ref: &Arc<RwLock<TimeSyncStatus>>,
    now_ms: f64,
    action: &str,
    stage: &str,
    last_error: Option<String>,
) {
    let system_time = SystemTime::now();
    update_shared_time(system_time);

    if let Ok(mut status) = status_ref.write() {
        status.enabled = config.enabled;
        status.running = true;
        status.current_source = "system".to_string();
        status.sync_stage = stage.to_string();
        status.last_action = action.to_string();
        status.last_attempt_at = Some(now_ms);
        status.last_error = last_error;
        status.system_clock_synced = true;

        if !config.enabled {
            status.last_server = None;
            status.last_offset_ms = None;
            status.last_delay_ms = None;
            status.selected_sample_count = None;
        }
    }
}

fn start_time_sync_thread(config: TimeRuntimeConfig) {
    update_time_sync_config(config);
    if let Ok(mut status) = shared_time_sync_status().write() {
        status.running = true;
        status.last_action = "startup".to_string();
        status.last_error = None;
    }

    TIME_SYNC_THREAD.get_or_init(|| {
        let status_ref = shared_time_sync_status();
        let config_ref = shared_time_sync_config();
        thread::Builder::new()
            .name("landscape-time-sync".to_string())
            .spawn(move || loop {
                let config = config_ref
                    .read()
                    .map(|config| normalized_time_config(config.clone()))
                    .unwrap_or_else(|_| normalized_time_config(TimeRuntimeConfig::default()));
                let now_ms = current_unix_ms();

                if !config.enabled {
                    mirror_system_time(
                        &config,
                        &status_ref,
                        now_ms,
                        "mirror-system",
                        "disabled",
                        None,
                    );
                    thread::sleep(StdDuration::from_secs(config.sync_interval_secs));
                    continue;
                }

                match query_ntp_time_with_sampling(
                    &config.servers,
                    StdDuration::from_secs(config.timeout_secs),
                    config.samples_per_server,
                ) {
                    Ok(result) => {
                        let (is_initial_sync, should_step) = status_ref
                            .read()
                            .map(|status| {
                                let is_initial_sync = status.last_success_at.is_none();
                                let should_step = is_initial_sync
                                    || result.offset_ms.abs() > config.step_threshold_ms as f64;
                                (is_initial_sync, should_step)
                            })
                            .unwrap_or((true, true));

                        let action = if is_initial_sync {
                            "initial-step"
                        } else if should_step {
                            "periodic-step"
                        } else {
                            "periodic-refresh"
                        };

                        match sys_config::set_system_time(result.synced_time) {
                            Ok(()) => {
                                tracing::info!(
                                    server = %result.server,
                                    action,
                                    offset_ms = result.offset_ms,
                                    delay_ms = result.delay_ms,
                                    "System time updated from NTP sync"
                                );
                                update_shared_time(result.synced_time);
                                if let Ok(mut status) = status_ref.write() {
                                    status.enabled = true;
                                    status.running = true;
                                    status.current_source = "ntp".to_string();
                                    status.sync_stage = if is_initial_sync {
                                        "initial".to_string()
                                    } else {
                                        "steady".to_string()
                                    };
                                    status.last_action = action.to_string();
                                    status.last_attempt_at = Some(now_ms);
                                    status.last_success_at = Some(now_ms);
                                    status.last_system_clock_update_at = Some(now_ms);
                                    status.last_server = Some(result.server.clone());
                                    status.last_offset_ms = Some(result.offset_ms);
                                    status.last_delay_ms = Some(result.delay_ms);
                                    status.selected_sample_count = Some(result.sample_count);
                                    status.last_error = None;
                                    status.system_clock_synced = true;
                                }
                            }
                            Err(err) => {
                                tracing::warn!(
                                    server = %result.server,
                                    action,
                                    offset_ms = result.offset_ms,
                                    delay_ms = result.delay_ms,
                                    error = %err,
                                    "NTP sync succeeded but failed to update system clock"
                                );
                                mirror_system_time(
                                    &config,
                                    &status_ref,
                                    now_ms,
                                    "ntp-set-failed",
                                    "error",
                                    Some(err.to_string()),
                                );
                                if let Ok(mut status) = status_ref.write() {
                                    status.last_success_at = Some(now_ms);
                                    status.last_server = Some(result.server.clone());
                                    status.last_offset_ms = Some(result.offset_ms);
                                    status.last_delay_ms = Some(result.delay_ms);
                                    status.selected_sample_count = Some(result.sample_count);
                                    status.system_clock_synced = false;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "NTP sync failed, falling back to system clock");
                        mirror_system_time(
                            &config,
                            &status_ref,
                            now_ms,
                            "fallback-system",
                            "fallback",
                            Some(err.to_string()),
                        );
                    }
                }

                thread::sleep(StdDuration::from_secs(config.sync_interval_secs));
            })
            .expect("failed to start time sync thread");
    });
}

fn get_synced_system_time() -> SystemTime {
    shared_time().read().map(|time| time.current_time()).unwrap_or_else(|_| SystemTime::now())
}

pub fn update_time_sync_config(config: TimeRuntimeConfig) {
    let config = normalized_time_config(config);
    if let Ok(mut shared_config) = shared_time_sync_config().write() {
        *shared_config = config.clone();
    }
    if let Ok(mut status) = shared_time_sync_status().write() {
        status.enabled = config.enabled;
    }
}

pub fn start_time_sync_service(config: TimeRuntimeConfig) {
    start_time_sync_thread(config);
}

pub fn start_ntp_sync_thread(config: TimeRuntimeConfig) {
    start_time_sync_service(config);
}

fn query_ntp_time_with_sampling(
    servers: &[String],
    timeout: StdDuration,
    samples_per_server: u8,
) -> std::io::Result<NtpQueryResult> {
    let mut last_error = None;
    let mut best_result: Option<NtpQueryResult> = None;

    for server in servers {
        for _ in 0..samples_per_server {
            match query_ntp_time_from_server(server, timeout) {
                Ok(result) => {
                    let replace = best_result
                        .as_ref()
                        .map(|best| {
                            result.delay_ms < best.delay_ms
                                || (result.delay_ms == best.delay_ms
                                    && result.offset_ms.abs() < best.offset_ms.abs())
                        })
                        .unwrap_or(true);
                    if replace {
                        best_result = Some(result);
                    }
                }
                Err(err) => {
                    tracing::warn!(server, error = %err, "failed to query NTP server sample");
                    last_error = Some(err);
                }
            }
        }
    }

    if let Some(mut best_result) = best_result {
        best_result.sample_count = samples_per_server;
        return Ok(best_result);
    }

    Err(last_error.unwrap_or_else(|| std::io::Error::other("no NTP server available")))
}

fn query_ntp_time_from_server(
    server: &str,
    timeout: StdDuration,
) -> std::io::Result<NtpQueryResult> {
    use std::io::{Error, ErrorKind};
    use std::net::UdpSocket;

    let server_addr = normalize_ntp_server_addr(server);
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(timeout))?;
    socket.set_write_timeout(Some(timeout))?;

    let mut request = [0_u8; 48];
    request[0] = 0x1b;
    let t1 = SystemTime::now();
    socket.send_to(&request, &server_addr)?;

    let mut response = [0_u8; 48];
    let (received, _) = socket.recv_from(&mut response)?;
    let t4 = SystemTime::now();
    if received < response.len() {
        return Err(Error::new(ErrorKind::UnexpectedEof, "incomplete NTP response"));
    }

    let mode = response[0] & 0x07;
    if mode != 4 && mode != 5 {
        return Err(Error::new(ErrorKind::InvalidData, "invalid NTP mode in response"));
    }

    let stratum = response[1];
    if stratum == 0 {
        return Err(Error::new(ErrorKind::InvalidData, "kiss-o'-death NTP response"));
    }

    let t2 = parse_ntp_timestamp(&response[32..40])?;
    let t3 = parse_ntp_timestamp(&response[40..48])?;
    let offset_ms = ((signed_duration_ms(t2, t1) + signed_duration_ms(t3, t4)) as f64) / 2.0;
    let delay_ms = (signed_duration_ms(t4, t1) - signed_duration_ms(t3, t2)).max(0) as f64;
    let synced_time = apply_offset(t4, offset_ms);

    Ok(NtpQueryResult {
        synced_time,
        server: server_addr,
        offset_ms,
        delay_ms,
        sample_count: 1,
    })
}

fn normalize_ntp_server_addr(server: &str) -> String {
    let server = server.trim();
    if server.is_empty() {
        return DEFAULT_TIME_FALLBACK_SERVER.to_string();
    }

    if let Some((_, port)) = server.rsplit_once(':') {
        if port.parse::<u16>().is_ok() {
            return server.to_string();
        }
    }

    format!("{server}:123")
}

fn current_unix_ms() -> f64 {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    duration.as_secs_f64() * 1000.0
}

fn parse_ntp_timestamp(bytes: &[u8]) -> std::io::Result<SystemTime> {
    use std::io::{Error, ErrorKind};

    if bytes.len() != 8 {
        return Err(Error::new(ErrorKind::InvalidData, "invalid NTP timestamp length"));
    }

    let seconds = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64;
    let fraction = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as u64;
    if seconds < NTP_UNIX_OFFSET_SECS {
        return Err(Error::new(ErrorKind::InvalidData, "invalid NTP timestamp"));
    }

    let unix_seconds = seconds - NTP_UNIX_OFFSET_SECS;
    let nanos = ((fraction as u128) * 1_000_000_000_u128 / (1_u128 << 32)) as u32;
    Ok(UNIX_EPOCH + StdDuration::new(unix_seconds, nanos))
}

fn signed_duration_ms(later: SystemTime, earlier: SystemTime) -> i128 {
    match later.duration_since(earlier) {
        Ok(duration) => duration.as_millis() as i128,
        Err(err) => -(err.duration().as_millis() as i128),
    }
}

fn apply_offset(base: SystemTime, offset_ms: f64) -> SystemTime {
    if offset_ms >= 0.0 {
        base + StdDuration::from_secs_f64(offset_ms / 1000.0)
    } else {
        base - StdDuration::from_secs_f64((-offset_ms) / 1000.0)
    }
}

impl LdCountdown {
    pub fn new(duration: Duration) -> Self {
        Self { start: Instant::now(), duration }
    }

    pub fn remaining(&self) -> Duration {
        let elapsed = self.start.elapsed();
        if elapsed >= self.duration {
            Duration::from_secs(0)
        } else {
            self.duration - elapsed
        }
    }
}

pub fn get_boot_time_ns() -> Result<u64, i32> {
    let mut ts: timespec = unsafe { mem::zeroed() };

    // 调用 clock_gettime 获取 CLOCK_BOOTTIME
    let result = unsafe { clock_gettime(CLOCK_BOOTTIME, &mut ts) };

    if result == 0 {
        // 转换为纳秒: 秒 * 10^9 + 纳秒部分
        let ns = (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64);
        Ok(ns)
    } else {
        // 返回错误代码
        Err(unsafe { *libc::__errno_location() })
    }
}

pub fn get_current_time_ns() -> Result<u64, i32> {
    let time = get_synced_system_time().duration_since(UNIX_EPOCH).map_err(|_| libc::EINVAL)?;
    Ok(time.as_nanos() as u64)
}

pub fn get_current_time_ms() -> Result<u64, i32> {
    Ok(get_current_time_ns()? / 1_000_000)
}

pub fn get_relative_time_ns() -> Result<u64, i32> {
    let current_time_ns = get_current_time_ns()?;
    let mut monotonic: timespec = unsafe { std::mem::zeroed() };

    if unsafe { clock_gettime(CLOCK_MONOTONIC, &mut monotonic) } != 0 {
        return Err(unsafe { *libc::__errno_location() });
    }

    let monotonic_ns = (monotonic.tv_sec as u64) * 1_000_000_000 + (monotonic.tv_nsec as u64);
    current_time_ns.checked_sub(monotonic_ns).ok_or(libc::EINVAL)
}

pub const MILL_A_DAY: u32 = 1000 * 60 * 60 * 24;
///
pub fn get_f64_timestamp() -> f64 {
    const MILLIS_PER_SEC: u64 = 1_000;
    let time = get_synced_system_time().duration_since(UNIX_EPOCH).expect("系统时间早于 UNIX");

    (time.as_secs() as f64) * (MILLIS_PER_SEC as f64) + (time.subsec_millis() as f64)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::get_boot_time_ns;

    #[test]
    pub fn test() {
        let now = Instant::now();
        match get_boot_time_ns() {
            Ok(ns) => println!("系统启动以来的时间（纳秒）: {}", ns),
            Err(e) => eprintln!("获取启动时间失败，错误码: {}", e),
        }
        println!("{}", now.elapsed().as_nanos())
    }
}
