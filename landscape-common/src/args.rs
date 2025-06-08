use std::{net::IpAddr, path::PathBuf};

use clap::{arg, Parser};
use once_cell::sync::Lazy;

use crate::LANDSCAPE_CONFIG_DIR_NAME;

pub static LAND_HOSTNAME: Lazy<String> = Lazy::new(|| {
    let hostname = hostname::get().expect("无法获取主机名");
    hostname.to_string_lossy().to_string()
});

pub static LAND_ARGS: Lazy<WebCommArgs> = Lazy::new(|| {
    dotenvy::dotenv().ok();
    WebCommArgs::parse()
});

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
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct WebCommArgs {
    /// Static HTML location [default: ~/.landscape-router/static]
    #[arg(short, long, env = "LANDSCAPE_WEB_ROOT")]
    pub web: Option<PathBuf>,

    /// Listen port [default: 6300]
    #[arg(short, long, env = "LANDSCAPE_WEB_PORT")]
    pub port: Option<u16>,

    /// Listen address [default: 0.0.0.0]
    #[arg(short, long, env = "LANDSCAPE_WEB_ADDR")]
    pub address: Option<IpAddr>,

    /// Controls whether the WAN IP can be used to access the management interface [default: false]
    #[arg(short, long)]
    pub export_manager: bool,

    /// All Config DIR, Not file Path [default: /root/.landscape-router]
    #[clap(short, long, env = "LANDSCAPE_CONF_PATH")]
    pub config_dir: Option<PathBuf>,

    /// Database URL, SQLite Connect Like  
    /// sqlite://./db.sqlite?mode=rwc
    /// [default: ~/.landscape-router/landscape_db.sqlite]
    #[clap(long = "db_url", env = "DATABASE_URL")]
    pub database_path: Option<String>,

    /// ebpf map space
    /// [default: default]
    #[clap(long, env = "LANDSCAPE_EBPF_MAP_SPACE", default_value = "default")]
    pub ebpf_map_space: String,

    /// Manager user [default: root]
    #[clap(long = "user", env = "LANDSCAPE_ADMIN_USER")]
    pub admin_user: Option<String>,

    /// Manager pass [default: root]
    #[clap(long = "pass", env = "LANDSCAPE_ADMIN_PASS")]
    pub admin_pass: Option<String>,

    /// Debug mode [default: false]
    #[arg(long, env = "LANDSCAPE_DEBUG")]
    pub debug: Option<bool>,

    /// Log output location [default: false]
    #[arg(long, env = "LANDSCAPE_LOG_PATH")]
    pub log_output_in_terminal: Option<bool>,

    /// Max log files number
    /// [default: 7]
    #[arg(long, env = "LANDSCAPE_LOG_FILE_LIMIT")]
    pub max_log_files: Option<usize>,
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
