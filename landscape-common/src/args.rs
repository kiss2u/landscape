use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use clap::{arg, Parser};
use once_cell::sync::Lazy;

pub static LAND_ARGS: Lazy<WebCommArgs> = Lazy::new(WebCommArgs::parse);

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct WebCommArgs {
    /// static html location
    #[arg(short, long, default_value = "./static")]
    pub web: PathBuf,

    /// listen port
    #[arg(short, long, default_value = "6300")]
    pub port: u16,

    /// listen address
    #[arg(short, long, default_value = "0.0.0.0")]
    pub address: IpAddr,

    /// Controls whether the WAN IP can be used to access the management interface
    #[arg(short, long, default_value = "true")]
    pub export_manager: bool,

    /// config home path
    #[clap(short, long)]
    pub config_path: Option<PathBuf>,

    /// ebpf map space
    #[clap(long, env = "LAND_SCAPE_EBPF_MAP_SPACE", default_value = "default")]
    pub ebpf_map_space: String,
}

impl WebCommArgs {
    pub fn get_ipv4_listen(&self) -> Option<(Ipv4Addr, u16)> {
        match self.address {
            IpAddr::V4(ipv4_addr) => Some((ipv4_addr, self.port)),
            IpAddr::V6(_) => None,
        }
    }
}
