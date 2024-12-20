use std::{net::IpAddr, path::PathBuf};

use clap::{arg, Parser};

#[derive(Parser, Debug)]
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
}
