mod ntp;
mod sync;

pub use ntp::{NtpClient, NtpQueryResult, UdpNtpClient};
pub use sync::{set_system_time, RealSystemClock, SyncTimeService, SystemClock};
