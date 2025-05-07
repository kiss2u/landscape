use std::mem;

use libc::{clock_gettime, timespec, CLOCK_BOOTTIME, CLOCK_MONOTONIC, CLOCK_REALTIME};
use tokio::time::{Duration, Instant};
pub struct LdCountdown {
    start: Instant,
    duration: Duration,
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

pub fn get_relative_time_ns() -> Result<u64, i32> {
    unsafe {
        let mut realtime: timespec = std::mem::zeroed();
        let mut monotonic: timespec = std::mem::zeroed();

        // 获取实时时间
        if clock_gettime(CLOCK_REALTIME, &mut realtime) != 0 {
            return Err(*libc::__errno_location());
        }

        // 获取单调时间
        if clock_gettime(CLOCK_MONOTONIC, &mut monotonic) != 0 {
            return Err(*libc::__errno_location());
        }

        // 计算差值并处理纳秒借位
        let mut tv_sec = realtime.tv_sec - monotonic.tv_sec;
        let mut tv_nsec = realtime.tv_nsec - monotonic.tv_nsec;

        // 当纳秒差为负时，从秒借位
        if tv_nsec < 0 {
            tv_sec -= 1;
            tv_nsec += 1_000_000_000;
        }

        // 检查时间差是否为负
        if tv_sec < 0 {
            return Err(libc::EINVAL); // 返回无效参数错误
        }

        // println!("tv_sec:{tv_sec}, tv_nsec: {tv_nsec}");

        // 计算总纳秒
        let ns = (tv_sec as u64) * 1_000_000_000 + (tv_nsec as u64);
        Ok(ns)
    }
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
