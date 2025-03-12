use std::{fs::OpenOptions, io::Write};

use serde::{Deserialize, Serialize};

pub mod pppd;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PPPDConfig {
    pub default_route: bool,
    pub peer_id: String,
    pub password: String,
}

impl PPPDConfig {
    fn delete_config(&self, ppp_iface_name: &str) {
        let _ = std::fs::remove_file(format!("/etc/ppp/peers/{}", ppp_iface_name));
    }
    fn write_config(&self, attach_iface_name: &str, ppp_iface_name: &str) -> Result<(), ()> {
        // 打开文件（如果文件不存在则创建）

        let Ok(mut file) = OpenOptions::new()
            .write(true) // 打开文件以进行写入
            .truncate(true) // 文件存在时会被截断
            .create(true) // 如果文件不存在，则会创建
            .open(format!("/etc/ppp/peers/{}", ppp_iface_name))
        else {
            return Err(());
        };

        let route = if self.default_route {
            r#"
defaultroute
replacedefaultroute
"#
        } else {
            ""
        };
        let config = format!(
            r#"
# 此文件每次启动 pppd 都会被复写, 所以修改此文件不会有任何效果, 仅作为检查启动配置
# This file is truncated each time pppd is started, so editing this file has no effect.
noipdefault
{route}
hide-password
lcp-echo-interval 30
lcp-echo-failure 4
noauth
persist
#mtu 1492
maxfail 1
#holdoff 20
plugin rp-pppoe.so
nic-{ifacename}
user "{user}"
password "{pass}"
ifname {ppp_iface_name}
"#,
            ifacename = attach_iface_name,
            user = self.peer_id,
            pass = self.password,
            ppp_iface_name = ppp_iface_name
        );
        let Ok(_) = file.write_all(config.as_bytes()) else {
            return Err(());
        };

        Ok(())
    }
}
