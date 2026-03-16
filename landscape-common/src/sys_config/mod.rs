use crate::SYSCTL_IPV4_ARP_IGNORE_PATTERN;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn init_sysctl_setting() {
    set_ipv4_arp_ignore_to_1();
}

pub fn set_system_time(time: SystemTime) -> std::io::Result<()> {
    let duration = time.duration_since(UNIX_EPOCH).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "system time is before UNIX_EPOCH")
    })?;

    let ts = libc::timespec {
        tv_sec: duration.as_secs() as libc::time_t,
        tv_nsec: duration.subsec_nanos() as libc::c_long,
    };

    let result = unsafe { libc::clock_settime(libc::CLOCK_REALTIME, &ts) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn set_ipv4_arp_ignore_to_1() {
    use sysctl::Sysctl;
    if let Ok(ctl) = sysctl::Ctl::new(&SYSCTL_IPV4_ARP_IGNORE_PATTERN.replace("{}", "all")) {
        match ctl.set_value_string("1") {
            Ok(value) => {
                if value != "1" {
                    tracing::error!("modify value error: {:?}", value)
                }
            }
            Err(e) => {
                tracing::error!("err: {e:?}")
            }
        }
    }
}
