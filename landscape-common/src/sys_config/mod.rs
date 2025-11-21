use crate::SYSCTL_IPV4_ARP_IGNORE_PATTERN;

pub fn init_sysctl_setting() {
    set_ipv4_arp_ignore_to_1();
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
