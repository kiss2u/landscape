#[cfg(test)]
mod tests {
    use std::{
        mem::MaybeUninit,
        net::{IpAddr, Ipv6Addr},
        str::FromStr,
    };

    use landscape_common::{
        flow::mark::FlowMark,
        ip_mark::{IpConfig, IpMarkInfo},
        route::{LanRouteInfo, LanRouteMode, RouteTargetInfo},
    };
    use libbpf_rs::{
        skel::{OpenSkel, SkelBuilder as _},
        ProgramInput,
    };
    use zerocopy::IntoBytes;

    use crate::{
        map_setting::{
            flow_wanip::create_inner_flow_match_map_v6,
            route::{add_lan_route_with_maps, add_wan_route_inner_v6, del_lan_route_with_maps},
        },
        route::lan_v2::route_lan::RouteLanSkelBuilder,
        tests::{
            route::package::{
                create_route_cache_inner_map_v4, create_route_cache_inner_map_v6,
                isolated_pin_root, lookup_inner_map_id, lookup_rt6_cache_value,
                simple_ipv6_tcp_syn, LAN_CACHE, WAN_CACHE,
            },
            TestSkb,
        },
    };

    fn local_addr() -> Ipv6Addr {
        Ipv6Addr::from_str("fd00::10").unwrap()
    }

    fn remote_addr() -> Ipv6Addr {
        Ipv6Addr::from_str("2001:db8:2::20").unwrap()
    }

    #[test]
    fn route_lan_ingress_v6_populates_lan_cache_on_redirect() {
        let mut builder = RouteLanSkelBuilder::default();
        let pin_root = isolated_pin_root("route-lan-v6-smoke");
        builder.object_builder_mut().pin_root_path(&pin_root).unwrap();

        let mut open_object = MaybeUninit::uninit();
        let open = builder.open(&mut open_object).unwrap();
        let skel = open.load().unwrap();

        create_route_cache_inner_map_v6(&skel.maps.rt6_cache_map, LAN_CACHE);

        let rules = vec![IpMarkInfo {
            mark: FlowMark::from(0x0305),
            cidr: IpConfig { ip: IpAddr::V6(remote_addr()), prefix: 128 },
            priority: 100,
        }];
        create_inner_flow_match_map_v6(&skel.maps.flow6_ip_map, 0, &rules).unwrap();

        add_wan_route_inner_v6(
            &skel.maps.rt6_target_map,
            5,
            &RouteTargetInfo {
                weight: 0,
                ifindex: 11,
                mac: None,
                default_route: false,
                is_docker: false,
                iface_name: "test-wan".to_string(),
                iface_ip: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                gateway_ip: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            },
        );

        let mut packet = simple_ipv6_tcp_syn(local_addr(), remote_addr());
        let mut ctx = TestSkb::default();
        ctx.ifindex = 6;

        let result = skel
            .progs
            .route_lan_ingress
            .test_run(ProgramInput {
                data_in: Some(&mut packet),
                context_in: Some(ctx.as_mut_bytes()),
                ..Default::default()
            })
            .expect("run route_lan_ingress");

        assert_eq!(result.return_value as i32, 7);

        let cache_value = lookup_rt6_cache_value(
            &skel.maps.rt6_cache_map,
            LAN_CACHE,
            local_addr(),
            remote_addr(),
        )
        .expect("LAN cache entry missing after IPv6 redirect");

        assert_eq!(unsafe { cache_value.__anon_rt_cache_value_v4_1.mark_value }, 0x0305);
    }

    #[test]
    fn add_lan_route_recreates_only_lan_cache() {
        let mut builder = RouteLanSkelBuilder::default();
        let pin_root = isolated_pin_root("route-lan-cache-invalidation");
        builder.object_builder_mut().pin_root_path(&pin_root).unwrap();

        let mut open_object = MaybeUninit::uninit();
        let open = builder.open(&mut open_object).unwrap();
        let skel = open.load().unwrap();

        create_route_cache_inner_map_v4(&skel.maps.rt4_cache_map, WAN_CACHE);
        create_route_cache_inner_map_v4(&skel.maps.rt4_cache_map, LAN_CACHE);
        create_route_cache_inner_map_v6(&skel.maps.rt6_cache_map, WAN_CACHE);
        create_route_cache_inner_map_v6(&skel.maps.rt6_cache_map, LAN_CACHE);

        let rt4_wan_before = lookup_inner_map_id(&skel.maps.rt4_cache_map, WAN_CACHE);
        let rt4_lan_before = lookup_inner_map_id(&skel.maps.rt4_cache_map, LAN_CACHE);
        let rt6_wan_before = lookup_inner_map_id(&skel.maps.rt6_cache_map, WAN_CACHE);
        let rt6_lan_before = lookup_inner_map_id(&skel.maps.rt6_cache_map, LAN_CACHE);

        let lan_info = LanRouteInfo {
            ifindex: 17,
            iface_name: "lan-test".to_string(),
            iface_ip: IpAddr::V6(remote_addr()),
            mac: None,
            prefix: 64,
            mode: LanRouteMode::Reachable,
        };

        let changed = add_lan_route_with_maps(
            &skel.maps.rt4_lan_map,
            &skel.maps.rt6_lan_map,
            &skel.maps.rt4_cache_map,
            &skel.maps.rt6_cache_map,
            &lan_info,
        );
        assert!(changed);

        let rt4_wan_after = lookup_inner_map_id(&skel.maps.rt4_cache_map, WAN_CACHE);
        let rt4_lan_after = lookup_inner_map_id(&skel.maps.rt4_cache_map, LAN_CACHE);
        let rt6_wan_after = lookup_inner_map_id(&skel.maps.rt6_cache_map, WAN_CACHE);
        let rt6_lan_after = lookup_inner_map_id(&skel.maps.rt6_cache_map, LAN_CACHE);

        assert_eq!(rt4_wan_after, rt4_wan_before);
        assert_ne!(rt4_lan_after, rt4_lan_before);
        assert_eq!(rt6_wan_after, rt6_wan_before);
        assert_ne!(rt6_lan_after, rt6_lan_before);

        let changed = add_lan_route_with_maps(
            &skel.maps.rt4_lan_map,
            &skel.maps.rt6_lan_map,
            &skel.maps.rt4_cache_map,
            &skel.maps.rt6_cache_map,
            &lan_info,
        );
        assert!(!changed);

        assert_eq!(lookup_inner_map_id(&skel.maps.rt4_cache_map, WAN_CACHE), rt4_wan_after);
        assert_eq!(lookup_inner_map_id(&skel.maps.rt4_cache_map, LAN_CACHE), rt4_lan_after);
        assert_eq!(lookup_inner_map_id(&skel.maps.rt6_cache_map, WAN_CACHE), rt6_wan_after);
        assert_eq!(lookup_inner_map_id(&skel.maps.rt6_cache_map, LAN_CACHE), rt6_lan_after);
    }

    #[test]
    fn del_lan_route_keeps_cache_inner_maps() {
        let mut builder = RouteLanSkelBuilder::default();
        let pin_root = isolated_pin_root("route-lan-delete-no-invalidation");
        builder.object_builder_mut().pin_root_path(&pin_root).unwrap();

        let mut open_object = MaybeUninit::uninit();
        let open = builder.open(&mut open_object).unwrap();
        let skel = open.load().unwrap();

        create_route_cache_inner_map_v4(&skel.maps.rt4_cache_map, WAN_CACHE);
        create_route_cache_inner_map_v4(&skel.maps.rt4_cache_map, LAN_CACHE);
        create_route_cache_inner_map_v6(&skel.maps.rt6_cache_map, WAN_CACHE);
        create_route_cache_inner_map_v6(&skel.maps.rt6_cache_map, LAN_CACHE);

        let lan_info = LanRouteInfo {
            ifindex: 17,
            iface_name: "lan-test".to_string(),
            iface_ip: IpAddr::V6(remote_addr()),
            mac: None,
            prefix: 64,
            mode: LanRouteMode::Reachable,
        };

        let changed = add_lan_route_with_maps(
            &skel.maps.rt4_lan_map,
            &skel.maps.rt6_lan_map,
            &skel.maps.rt4_cache_map,
            &skel.maps.rt6_cache_map,
            &lan_info,
        );
        assert!(changed);

        let rt4_wan_before = lookup_inner_map_id(&skel.maps.rt4_cache_map, WAN_CACHE);
        let rt4_lan_before = lookup_inner_map_id(&skel.maps.rt4_cache_map, LAN_CACHE);
        let rt6_wan_before = lookup_inner_map_id(&skel.maps.rt6_cache_map, WAN_CACHE);
        let rt6_lan_before = lookup_inner_map_id(&skel.maps.rt6_cache_map, LAN_CACHE);

        let changed =
            del_lan_route_with_maps(&skel.maps.rt4_lan_map, &skel.maps.rt6_lan_map, &lan_info);
        assert!(changed);

        assert_eq!(lookup_inner_map_id(&skel.maps.rt4_cache_map, WAN_CACHE), rt4_wan_before);
        assert_eq!(lookup_inner_map_id(&skel.maps.rt4_cache_map, LAN_CACHE), rt4_lan_before);
        assert_eq!(lookup_inner_map_id(&skel.maps.rt6_cache_map, WAN_CACHE), rt6_wan_before);
        assert_eq!(lookup_inner_map_id(&skel.maps.rt6_cache_map, LAN_CACHE), rt6_lan_before);
    }
}
