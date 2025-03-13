use std::{
    net::{IpAddr, Ipv4Addr},
    num::ParseIntError,
    path::PathBuf,
    str::FromStr,
};

use clap::{arg, Parser};
use once_cell::sync::Lazy;

use crate::{LANDSCAPE_CONFIG_DIR_NAME, LANDSCAPE_LOG_DIR_NAME, LANDSCAPE_WEBROOT_DIR_NAME};

pub static LAND_HOSTNAME: Lazy<String> = Lazy::new(|| {
    let hostname = hostname::get().expect("无法获取主机名");
    hostname.to_string_lossy().to_string()
});

pub static LAND_ARGS: Lazy<WebCommArgs> = Lazy::new(WebCommArgs::parse);

pub static LAND_HOME_PATH: Lazy<PathBuf> = Lazy::new(|| {
    if let Some(path) = &LAND_ARGS.config_dir {
        path.clone()
    } else {
        let Some(path) = homedir::my_home().unwrap() else {
            panic!("can not get home path");
        };
        path.join(LANDSCAPE_CONFIG_DIR_NAME)
    }
});

pub static LAND_LOG_ARGS: Lazy<LogConfig> = Lazy::new(|| {
    let log_path = LAND_HOME_PATH.join(LANDSCAPE_LOG_DIR_NAME);
    LogConfig {
        log_path,
        debug: LAND_ARGS.debug,
        log_output_in_terminal: LAND_ARGS.log_output_in_terminal,
        max_log_files: LAND_ARGS.max_log_files,
    }
});

pub static LAND_WEB_ARGS: Lazy<WebConfig> = Lazy::new(|| {
    let web_root = if let Some(web_root) = &LAND_ARGS.web {
        web_root.clone()
    } else {
        LAND_HOME_PATH.join(LANDSCAPE_WEBROOT_DIR_NAME)
    };
    WebConfig {
        web_root,
        port: LAND_ARGS.port,
        address: LAND_ARGS.address,
    }
});

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct WebCommArgs {
    /// Static html location [default: /root/.landscape-router/static]
    #[arg(short, long)]
    pub web: Option<PathBuf>,

    /// Listen port
    #[arg(short, long, default_value = "6300")]
    pub port: u16,

    /// Listen address
    #[arg(short, long, default_value = "0.0.0.0")]
    pub address: IpAddr,

    /// Controls whether the WAN IP can be used to access the management interface
    #[arg(short, long, default_value = "true")]
    pub export_manager: bool,

    /// Allow some ports through NAT (temporarily)
    #[arg(short, long)]
    pub through_nat_port: Option<Vec<NumberRange>>,

    /// All Config DIR, Not file Path [default: /root/.landscape-router]
    #[clap(short, long)]
    pub config_dir: Option<PathBuf>,

    /// ebpf map space
    #[clap(long, env = "LANDSCAPE_EBPF_MAP_SPACE", default_value = "default")]
    pub ebpf_map_space: String,

    /// Manager user
    #[clap(long = "user", env = "LANDSCAPE_ADMIN_USER", default_value = "root")]
    pub admin_user: String,

    /// Manager pass
    #[clap(long = "pass", env = "LANDSCAPE_ADMIN_PASS", default_value = "root")]
    pub admin_pass: String,

    /// Debug mode
    #[arg(long, default_value_t = debug_default())]
    pub debug: bool,

    /// Log output location
    #[arg(long, default_value_t = debug_default())]
    pub log_output_in_terminal: bool,

    /// Max log files number
    #[arg(long, default_value = "7")]
    pub max_log_files: usize,

    /// Enable interface addr observer
    #[arg(long, default_value = "false")]
    pub iface_ob: bool,
}

const fn debug_default() -> bool {
    #[cfg(debug_assertions)]
    {
        true
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

impl WebCommArgs {
    pub fn get_ipv4_listen(&self) -> Option<(Ipv4Addr, u16)> {
        match self.address {
            IpAddr::V4(ipv4_addr) => Some((ipv4_addr, self.port)),
            IpAddr::V6(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebConfig {
    pub web_root: PathBuf,

    pub port: u16,

    pub address: IpAddr,
}

#[derive(Debug, Clone)]
pub struct LogConfig {
    pub log_path: PathBuf,
    pub debug: bool,
    pub log_output_in_terminal: bool,
    pub max_log_files: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NumberRange {
    pub start: u16,
    pub end: u16,
}

impl FromStr for NumberRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        match parts.as_slice() {
            [single] => {
                let num = single.parse().map_err(|e: ParseIntError| e.to_string())?;
                Ok(NumberRange { start: num, end: num })
            }
            [start, end] => {
                let start = start.parse().map_err(|e: ParseIntError| e.to_string())?;
                let end = end.parse().map_err(|e: ParseIntError| e.to_string())?;
                Ok(NumberRange { start, end })
            }
            _ => Err("Invalid range format. Use '10-20' or '50'.".to_string()),
        }
    }
}
