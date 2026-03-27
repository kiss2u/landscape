mod lan;
mod package;
#[cfg(test)]
mod test_route;

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
        route::RouteTargetInfo,
    };
    use libbpf_rs::{
        skel::{OpenSkel, SkelBuilder as _},
        ProgramInput,
    };
    use zerocopy::IntoBytes;

    use crate::{
        map_setting::{flow_wanip::create_inner_flow_match_map_v6, route::add_wan_route_inner_v6},
        route::wan_v2::route_wan::RouteWanSkelBuilder,
        tests::{
            route::package::{isolated_pin_root, simple_ipv6_tcp_syn},
            TestSkb,
        },
    };

    const FLOW_SOURCE_SHIFT: u32 = 24;
    const FLOW_FROM_WAN: u32 = 4;

    fn local_addr() -> Ipv6Addr {
        Ipv6Addr::from_str("fd00::10").unwrap()
    }

    fn remote_addr() -> Ipv6Addr {
        Ipv6Addr::from_str("2001:db8:2::20").unwrap()
    }

    #[test]
    fn route_wan_egress_v6_sets_flow_from_wan() {
        let mut builder = RouteWanSkelBuilder::default();
        let pin_root = isolated_pin_root("route-wan-v6-smoke");
        builder.object_builder_mut().pin_root_path(&pin_root).unwrap();

        let mut open_object = MaybeUninit::uninit();
        let open = builder.open(&mut open_object).unwrap();
        let skel = open.load().unwrap();

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
        let mut ctx_in = TestSkb::default();
        ctx_in.ifindex = 6;
        ctx_in.ingress_ifindex = 0;
        let mut ctx_out = TestSkb::default();

        let result = skel
            .progs
            .route_wan_egress
            .test_run(ProgramInput {
                data_in: Some(&mut packet),
                context_in: Some(ctx_in.as_mut_bytes()),
                context_out: Some(ctx_out.as_mut_bytes()),
                ..Default::default()
            })
            .expect("run route_wan_egress");

        assert_eq!(result.return_value as i32, 7);
        assert_eq!(ctx_out.mark >> FLOW_SOURCE_SHIFT, FLOW_FROM_WAN);
    }
}
