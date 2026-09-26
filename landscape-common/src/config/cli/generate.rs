use std::net::Ipv4Addr;

use crate::{
    config::{settings::LandscapeConfig, InitConfig},
    config_service::iface::{CreateDevType, IfaceZoneType, NetworkIfaceConfig, WifiMode},
    lan_service::{
        lan_dhcpv4::config::{DHCPv4ServerConfig, DHCPv4ServiceConfig},
        lan_route::RouteLanServiceConfig,
    },
    utils::time::get_f64_timestamp,
    wan_service::{
        firewall::service::FirewallServiceConfig,
        ip_config::{IfaceIpModelConfig, IfaceIpServiceConfig},
        mss_clamp::MSSClampServiceConfig,
        nat::config::{NatConfig, NatServiceConfig},
        pppd::{PPPDConfig, PPPDServiceConfig},
        wan_route::RouteWanServiceConfig,
    },
    VERSION,
};

use super::{
    ConfigCliArgs, ConfigCliError, WanMode, BASE_ENABLED_SERVICES, DEFAULT_MSS_CLAMP_SIZE,
    KNOWN_SERVICES, WAN_SERVICES,
};

impl ConfigCliArgs {
    fn default_route(&self) -> bool {
        !self.no_wan_default_route
    }

    fn resolve_enabled_services(&self) -> Result<Vec<&'static str>, ConfigCliError> {
        for name in self.enable.iter().chain(self.disable.iter()) {
            if !KNOWN_SERVICES.contains(&name.as_str()) {
                return Err(ConfigCliError::UnknownService(name.clone()));
            }
        }
        for name in &self.enable {
            if self.disable.contains(name) {
                return Err(ConfigCliError::ConflictingService(name.clone()));
            }
        }

        let mut enabled: Vec<&'static str> = BASE_ENABLED_SERVICES.to_vec();
        if matches!(self.wan_mode, WanMode::Pppoe | WanMode::Pppd) {
            enabled.push("mss-clamp");
        }
        for name in &self.enable {
            let known = KNOWN_SERVICES.iter().find(|k| **k == name.as_str()).unwrap();
            if !enabled.contains(known) {
                enabled.push(*known);
            }
        }
        enabled.retain(|name| !self.disable.iter().any(|d| d.as_str() == *name));

        if self.wan_mode == WanMode::None {
            for name in &enabled {
                if WAN_SERVICES.contains(name) && self.enable.iter().any(|e| e.as_str() == *name) {
                    return Err(ConfigCliError::WanServiceWithoutWan((*name).to_string()));
                }
            }
            enabled.retain(|name| !WAN_SERVICES.contains(name));
        }

        Ok(enabled)
    }

    /// Build the [`InitConfig`] described by these arguments.
    pub fn build_init_config(&self) -> Result<InitConfig, ConfigCliError> {
        let lan_iface = self.lan_iface.clone().ok_or(ConfigCliError::MissingLanIface)?;
        let wan_iface = match self.wan_mode {
            WanMode::None => None,
            _ => Some(self.wan_iface.clone().ok_or(ConfigCliError::MissingWanIface)?),
        };

        let mut members = Vec::new();
        for member in &self.lan_member {
            if members.contains(member) {
                continue;
            }
            if Some(member) == wan_iface.as_ref() || member == &lan_iface {
                return Err(ConfigCliError::InvalidLanMember(member.clone()));
            }
            members.push(member.clone());
        }

        let enabled = self.resolve_enabled_services()?;
        let now = get_f64_timestamp();

        let mut ifaces = Vec::new();
        if let Some(wan_iface) = &wan_iface {
            ifaces.push(NetworkIfaceConfig {
                name: wan_iface.clone(),
                create_dev_type: CreateDevType::NoNeedToCreate,
                controller_name: None,
                zone_type: IfaceZoneType::Wan,
                enable_in_boot: true,
                wifi_mode: WifiMode::default(),
                xps_rps: None,
                update_at: now,
            });
        }
        ifaces.push(NetworkIfaceConfig::crate_bridge(lan_iface.clone(), Some(IfaceZoneType::Lan)));
        for member in &members {
            ifaces.push(NetworkIfaceConfig {
                name: member.clone(),
                create_dev_type: CreateDevType::NoNeedToCreate,
                controller_name: Some(lan_iface.clone()),
                zone_type: IfaceZoneType::default(),
                enable_in_boot: true,
                wifi_mode: WifiMode::default(),
                xps_rps: None,
                update_at: now,
            });
        }

        let mut ipconfigs = Vec::new();
        let mut pppds = Vec::new();
        if let Some(wan_iface) = &wan_iface {
            match self.wan_mode {
                WanMode::Dhcp => {
                    ipconfigs.push(IfaceIpServiceConfig {
                        iface_name: wan_iface.clone(),
                        enable: true,
                        ip_model: IfaceIpModelConfig::DhcpClient {
                            default_router: self.default_route(),
                            hostname: None,
                            custome_opts: Vec::new(),
                        },
                        update_at: now,
                    });
                }
                WanMode::Static => {
                    let raw = self.wan_ip.as_ref().ok_or(ConfigCliError::MissingWanIp)?;
                    let (ipv4, mask) = parse_ipv4_cidr(raw)?;
                    let gateway = self.wan_gateway.ok_or(ConfigCliError::MissingWanGateway)?;
                    ipconfigs.push(IfaceIpServiceConfig {
                        iface_name: wan_iface.clone(),
                        enable: true,
                        ip_model: IfaceIpModelConfig::Static {
                            default_router_ip: Some(gateway),
                            default_router: self.default_route(),
                            ipv4: Some(ipv4),
                            ipv4_mask: mask,
                            ipv6: self.wan_ipv6,
                        },
                        update_at: now,
                    });
                }
                WanMode::Pppoe => {
                    let (username, password) = self.pppoe_credentials("pppoe")?;
                    ipconfigs.push(IfaceIpServiceConfig {
                        iface_name: wan_iface.clone(),
                        enable: true,
                        ip_model: IfaceIpModelConfig::PPPoE {
                            default_router: self.default_route(),
                            username,
                            password,
                            mtu: self.pppoe_mtu,
                            ac_name: self.pppoe_ac_name.clone(),
                        },
                        update_at: now,
                    });
                }
                WanMode::Pppd => {
                    let (peer_id, password) = self.pppoe_credentials("pppd")?;
                    pppds.push(PPPDServiceConfig {
                        attach_iface_name: wan_iface.clone(),
                        iface_name: self.pppd_iface.clone(),
                        enable: true,
                        pppd_config: PPPDConfig {
                            default_route: self.default_route(),
                            peer_id,
                            password,
                            ac: self.pppoe_ac_name.clone(),
                            plugin: self.pppd_plugin.into(),
                        },
                        update_at: now,
                    });
                }
                WanMode::None => {}
            }
        }

        // For pppd, the WAN-facing services attach to the PPP virtual interface.
        let wan_service_iface = match self.wan_mode {
            WanMode::Pppd => Some(self.pppd_iface.clone()),
            _ => wan_iface.clone(),
        };

        let mut init = InitConfig {
            version: VERSION.to_string(),
            config: LandscapeConfig::default(),
            ifaces,
            ipconfigs,
            pppds,
            ..Default::default()
        };

        if !self.no_lan_dhcp {
            init.dhcpv4_services.push(self.build_dhcp_config(&lan_iface, now)?);
        }

        if let Some(iface) = &wan_service_iface {
            for service in &enabled {
                match *service {
                    "nat" => init.nats.push(NatServiceConfig {
                        iface_name: iface.clone(),
                        enable: true,
                        nat_config: NatConfig::default(),
                        update_at: now,
                    }),
                    "firewall" => init.firewalls.push(FirewallServiceConfig {
                        iface_name: iface.clone(),
                        enable: true,
                        update_at: now,
                    }),
                    "mss-clamp" => init.mss_clamps.push(MSSClampServiceConfig {
                        iface_name: iface.clone(),
                        enable: true,
                        clamp_size: DEFAULT_MSS_CLAMP_SIZE,
                        update_at: now,
                    }),
                    "route-wan" => init.route_wans.push(RouteWanServiceConfig {
                        iface_name: iface.clone(),
                        enable: true,
                        update_at: now,
                    }),
                    "route-lan" => init.route_lans.push(RouteLanServiceConfig {
                        iface_name: lan_iface.clone(),
                        enable: true,
                        static_routes: None,
                        update_at: now,
                    }),
                    _ => {}
                }
            }
        } else {
            for service in &enabled {
                if *service == "route-lan" {
                    init.route_lans.push(RouteLanServiceConfig {
                        iface_name: lan_iface.clone(),
                        enable: true,
                        static_routes: None,
                        update_at: now,
                    });
                }
            }
        }

        Ok(init)
    }

    fn pppoe_credentials(&self, mode: &'static str) -> Result<(String, String), ConfigCliError> {
        match (self.pppoe_username.clone(), self.pppoe_password.clone()) {
            (Some(username), Some(password)) if !username.is_empty() && !password.is_empty() => {
                Ok((username, password))
            }
            _ => Err(ConfigCliError::MissingPppoeCredentials(mode)),
        }
    }

    fn build_dhcp_config(
        &self,
        lan_iface: &str,
        now: f64,
    ) -> Result<DHCPv4ServiceConfig, ConfigCliError> {
        let (server_ip, network_mask) = parse_ipv4_cidr(&self.lan_ip)?;
        let (ip_range_start, ip_range_end) = match &self.lan_dhcp_range {
            Some(range) => parse_dhcp_range(range)?,
            None => (default_dhcp_range_start(server_ip, network_mask), None),
        };

        let config = DHCPv4ServerConfig {
            ip_range_start,
            ip_range_end,
            server_ip_addr: server_ip,
            network_mask,
            address_lease_time: self.lan_dhcp_lease,
            custom_options: Vec::new(),
        };
        config.validate().map_err(|e| ConfigCliError::InvalidDhcpConfig(e.to_string()))?;

        Ok(DHCPv4ServiceConfig {
            iface_name: lan_iface.to_string(),
            enable: true,
            config,
            update_at: now,
        })
    }
}

fn parse_ipv4_cidr(raw: &str) -> Result<(Ipv4Addr, u8), ConfigCliError> {
    let (ip, prefix) =
        raw.split_once('/').ok_or_else(|| ConfigCliError::InvalidCidr(raw.into()))?;
    let ip = ip
        .trim()
        .parse::<Ipv4Addr>()
        .map_err(|_| ConfigCliError::InvalidIp(ip.trim().to_string()))?;
    let prefix = prefix
        .trim()
        .parse::<u8>()
        .map_err(|_| ConfigCliError::InvalidPrefix(prefix.trim().to_string()))?;
    if prefix > 32 {
        return Err(ConfigCliError::InvalidPrefix(prefix.to_string()));
    }
    Ok((ip, prefix))
}

fn parse_dhcp_range(raw: &str) -> Result<(Ipv4Addr, Option<Ipv4Addr>), ConfigCliError> {
    let (start, end) = match raw.split_once('-') {
        Some((start, end)) => (start, Some(end)),
        None => (raw, None),
    };
    let start = start
        .trim()
        .parse::<Ipv4Addr>()
        .map_err(|_| ConfigCliError::InvalidDhcpRange(raw.to_string()))?;
    let end = match end {
        Some(end) => Some(
            end.trim()
                .parse::<Ipv4Addr>()
                .map_err(|_| ConfigCliError::InvalidDhcpRange(raw.to_string()))?,
        ),
        None => None,
    };
    Ok((start, end))
}

fn default_dhcp_range_start(server_ip: Ipv4Addr, network_mask: u8) -> Ipv4Addr {
    let mask_bits = if network_mask == 0 { 0 } else { u32::MAX << (32 - network_mask) };
    let network = u32::from(server_ip) & mask_bits;
    let broadcast = network | !mask_bits;
    let start = network.saturating_add(100);
    if start >= broadcast {
        Ipv4Addr::from(network.saturating_add(2))
    } else {
        Ipv4Addr::from(start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::cli::{DEFAULT_PPPD_IFACE, DEFAULT_PPPOE_MTU},
        wan_service::pppd::PPPoEPlugin,
    };

    fn base_args() -> ConfigCliArgs {
        ConfigCliArgs {
            wan_iface: Some("eth0".to_string()),
            lan_iface: Some("br_lan".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn default_dhcp_generation_matches_expected_shape() {
        let init = base_args().build_init_config().unwrap();

        assert_eq!(init.version, VERSION);
        assert_eq!(init.ifaces.len(), 2);
        assert_eq!(init.ifaces[0].name, "eth0");
        assert_eq!(init.ifaces[0].zone_type, IfaceZoneType::Wan);
        assert_eq!(init.ifaces[1].name, "br_lan");
        assert_eq!(init.ifaces[1].zone_type, IfaceZoneType::Lan);

        assert_eq!(init.ipconfigs.len(), 1);
        assert!(matches!(
            &init.ipconfigs[0].ip_model,
            IfaceIpModelConfig::DhcpClient { default_router: true, .. }
        ));

        assert_eq!(init.dhcpv4_services.len(), 1);
        let dhcp = &init.dhcpv4_services[0].config;
        assert_eq!(dhcp.server_ip_addr, Ipv4Addr::new(192, 168, 5, 1));
        assert_eq!(dhcp.network_mask, 24);
        assert_eq!(dhcp.ip_range_start, Ipv4Addr::new(192, 168, 5, 100));

        assert_eq!(init.nats.len(), 1);
        assert_eq!(init.route_wans.len(), 1);
        assert_eq!(init.route_lans.len(), 1);
        assert!(init.mss_clamps.is_empty());
        assert!(init.firewalls.is_empty());
        assert!(init.pppds.is_empty());
    }

    #[test]
    fn wan_iface_is_required_when_mode_is_not_none() {
        let args = ConfigCliArgs {
            wan_iface: None,
            lan_iface: Some("br_lan".to_string()),
            ..Default::default()
        };
        assert!(matches!(args.build_init_config(), Err(ConfigCliError::MissingWanIface)));
    }

    #[test]
    fn lan_iface_is_required() {
        let args = ConfigCliArgs {
            wan_iface: Some("eth0".to_string()),
            lan_iface: None,
            ..Default::default()
        };
        assert!(matches!(args.build_init_config(), Err(ConfigCliError::MissingLanIface)));
    }

    #[test]
    fn wan_mode_none_omits_wan_iface_and_wan_services() {
        let args = ConfigCliArgs {
            wan_iface: None,
            wan_mode: WanMode::None,
            lan_iface: Some("br_lan".to_string()),
            ..Default::default()
        };
        let init = args.build_init_config().unwrap();

        assert!(init.ifaces.iter().all(|iface| iface.zone_type != IfaceZoneType::Wan));
        assert!(init.ipconfigs.is_empty());
        assert!(init.nats.is_empty());
        assert!(init.mss_clamps.is_empty());
        assert!(init.route_wans.is_empty());
        assert_eq!(init.route_lans.len(), 1);
    }

    #[test]
    fn explicit_wan_service_with_mode_none_is_rejected() {
        let args = ConfigCliArgs {
            wan_mode: WanMode::None,
            lan_iface: Some("br_lan".to_string()),
            enable: vec!["nat".to_string()],
            ..Default::default()
        };
        assert!(matches!(
            args.build_init_config(),
            Err(ConfigCliError::WanServiceWithoutWan(name)) if name == "nat"
        ));
    }

    #[test]
    fn static_mode_requires_ip_and_gateway() {
        let mut args = base_args();
        args.wan_mode = WanMode::Static;
        assert!(matches!(args.build_init_config(), Err(ConfigCliError::MissingWanIp)));

        args.wan_ip = Some("203.0.113.2/24".to_string());
        assert!(matches!(args.build_init_config(), Err(ConfigCliError::MissingWanGateway)));

        args.wan_gateway = Some(Ipv4Addr::new(203, 0, 113, 1));
        let init = args.build_init_config().unwrap();
        match &init.ipconfigs[0].ip_model {
            IfaceIpModelConfig::Static { ipv4, ipv4_mask, default_router_ip, .. } => {
                assert_eq!(*ipv4, Some(Ipv4Addr::new(203, 0, 113, 2)));
                assert_eq!(*ipv4_mask, 24);
                assert_eq!(*default_router_ip, Some(Ipv4Addr::new(203, 0, 113, 1)));
            }
            other => panic!("unexpected ip model: {other:?}"),
        }
    }

    #[test]
    fn pppoe_mode_builds_native_ip_config() {
        let mut args = base_args();
        args.wan_mode = WanMode::Pppoe;
        args.pppoe_username = Some("user".to_string());
        args.pppoe_password = Some("pass".to_string());
        args.pppoe_ac_name = Some("ac".to_string());

        let init = args.build_init_config().unwrap();
        assert!(init.pppds.is_empty());
        assert_eq!(init.mss_clamps.len(), 1, "pppoe defaults to mss-clamp");
        match &init.ipconfigs[0].ip_model {
            IfaceIpModelConfig::PPPoE { username, password, mtu, ac_name, .. } => {
                assert_eq!(username, "user");
                assert_eq!(password, "pass");
                assert_eq!(*mtu, DEFAULT_PPPOE_MTU);
                assert_eq!(ac_name.as_deref(), Some("ac"));
            }
            other => panic!("unexpected ip model: {other:?}"),
        }
    }

    #[test]
    fn pppd_mode_builds_pppd_service_and_targets_ppp_iface() {
        let mut args = base_args();
        args.wan_mode = WanMode::Pppd;
        args.pppoe_username = Some("user".to_string());
        args.pppoe_password = Some("pass".to_string());

        let init = args.build_init_config().unwrap();
        assert!(init.ipconfigs.is_empty());
        assert_eq!(init.pppds.len(), 1);
        let pppd = &init.pppds[0];
        assert_eq!(pppd.attach_iface_name, "eth0");
        assert_eq!(pppd.iface_name, DEFAULT_PPPD_IFACE);
        assert_eq!(pppd.pppd_config.peer_id, "user");
        assert_eq!(pppd.pppd_config.password, "pass");
        assert!(matches!(pppd.pppd_config.plugin, PPPoEPlugin::RpPppoe));

        assert_eq!(init.nats[0].iface_name, DEFAULT_PPPD_IFACE);
        assert_eq!(init.route_wans[0].iface_name, DEFAULT_PPPD_IFACE);
        assert_eq!(init.mss_clamps.len(), 1, "pppd defaults to mss-clamp");
        assert_eq!(init.mss_clamps[0].iface_name, DEFAULT_PPPD_IFACE);
    }

    #[test]
    fn pppoe_credentials_are_required() {
        let mut args = base_args();
        args.wan_mode = WanMode::Pppoe;
        assert!(matches!(
            args.build_init_config(),
            Err(ConfigCliError::MissingPppoeCredentials("pppoe"))
        ));

        args.wan_mode = WanMode::Pppd;
        assert!(matches!(
            args.build_init_config(),
            Err(ConfigCliError::MissingPppoeCredentials("pppd"))
        ));
    }

    #[test]
    fn unknown_service_is_rejected() {
        let mut args = base_args();
        args.enable = vec!["dns".to_string()];
        assert!(matches!(args.build_init_config(), Err(ConfigCliError::UnknownService(_))));
    }

    #[test]
    fn conflicting_service_is_rejected() {
        let mut args = base_args();
        args.enable = vec!["nat".to_string()];
        args.disable = vec!["nat".to_string()];
        assert!(matches!(args.build_init_config(), Err(ConfigCliError::ConflictingService(_))));
    }

    #[test]
    fn disable_and_enable_adjust_defaults() {
        let mut args = base_args();
        args.disable = vec!["nat".to_string(), "route-lan".to_string()];
        args.enable = vec!["firewall".to_string()];
        let init = args.build_init_config().unwrap();

        assert!(init.nats.is_empty());
        assert!(init.route_lans.is_empty());
        assert_eq!(init.firewalls.len(), 1);
        assert!(init.mss_clamps.is_empty(), "dhcp does not default mss-clamp");
    }

    #[test]
    fn mss_clamp_defaults_only_for_ppp_modes() {
        for mode in [WanMode::Dhcp, WanMode::Static] {
            let mut args = base_args();
            args.wan_mode = mode;
            if mode == WanMode::Static {
                args.wan_ip = Some("203.0.113.2/24".to_string());
                args.wan_gateway = Some(Ipv4Addr::new(203, 0, 113, 1));
            }
            let init = args.build_init_config().unwrap();
            assert!(init.mss_clamps.is_empty(), "{mode:?} must not default mss-clamp");
        }

        for mode in [WanMode::Pppoe, WanMode::Pppd] {
            let mut args = base_args();
            args.wan_mode = mode;
            args.pppoe_username = Some("user".to_string());
            args.pppoe_password = Some("pass".to_string());
            let init = args.build_init_config().unwrap();
            assert_eq!(init.mss_clamps.len(), 1, "{mode:?} must default mss-clamp");
        }
    }

    #[test]
    fn mss_clamp_can_be_explicitly_enabled_for_dhcp() {
        let mut args = base_args();
        args.enable = vec!["mss-clamp".to_string()];
        let init = args.build_init_config().unwrap();
        assert_eq!(init.mss_clamps.len(), 1);
    }

    #[test]
    fn lan_members_attach_to_bridge() {
        let mut args = base_args();
        args.lan_member = vec!["eth1".to_string(), "eth2".to_string(), "eth1".to_string()];
        let init = args.build_init_config().unwrap();

        let bridges: Vec<_> = init.ifaces.iter().filter(|iface| iface.name == "br_lan").collect();
        assert_eq!(bridges.len(), 1);
        let eth1 = init.ifaces.iter().find(|iface| iface.name == "eth1").unwrap();
        assert_eq!(eth1.controller_name.as_deref(), Some("br_lan"));
        assert_eq!(
            init.ifaces.iter().filter(|iface| iface.name == "eth1").count(),
            1,
            "duplicate members must be deduplicated"
        );
    }

    #[test]
    fn lan_member_cannot_be_wan_or_lan_iface() {
        let mut args = base_args();
        args.lan_member = vec!["eth0".to_string()];
        assert!(matches!(args.build_init_config(), Err(ConfigCliError::InvalidLanMember(_))));

        args.lan_member = vec!["br_lan".to_string()];
        assert!(matches!(args.build_init_config(), Err(ConfigCliError::InvalidLanMember(_))));
    }

    #[test]
    fn lan_dhcp_can_be_disabled() {
        let mut args = base_args();
        args.no_lan_dhcp = true;
        let init = args.build_init_config().unwrap();
        assert!(init.dhcpv4_services.is_empty());
    }

    #[test]
    fn explicit_dhcp_range_and_lease_are_applied() {
        let mut args = base_args();
        args.lan_dhcp_range = Some("192.168.5.50-192.168.5.80".to_string());
        args.lan_dhcp_lease = Some(3600);
        let init = args.build_init_config().unwrap();

        let dhcp = &init.dhcpv4_services[0].config;
        assert_eq!(dhcp.ip_range_start, Ipv4Addr::new(192, 168, 5, 50));
        assert_eq!(dhcp.ip_range_end, Some(Ipv4Addr::new(192, 168, 5, 80)));
        assert_eq!(dhcp.address_lease_time, Some(3600));
    }

    #[test]
    fn invalid_dhcp_range_is_rejected() {
        let mut args = base_args();
        args.lan_dhcp_range = Some("192.168.5.200-192.168.5.10".to_string());
        assert!(matches!(args.build_init_config(), Err(ConfigCliError::InvalidDhcpConfig(_))));
    }
}
