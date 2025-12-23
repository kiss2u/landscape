#[cfg(test)]
pub mod tests {
    use crate::{
        route::lan_v2::route_lan::RouteLanSkelBuilder, tests::route::package::simple_tcp_syn,
    };
    use std::{
        mem::MaybeUninit,
        net::{IpAddr, Ipv4Addr},
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

    #[test]
    fn lan_route_ingress() {
        let mut wan_route_open_object = MaybeUninit::zeroed();

        let wan_route_builder = RouteLanSkelBuilder::default();

        let wan_route_open_skel = wan_route_builder.open(&mut wan_route_open_object).unwrap();

        let skel = wan_route_open_skel.load().unwrap();

        let data = vec![IpMarkInfo {
            mark: FlowMark::from(0x0305),
            cidr: IpConfig {
                ip: IpAddr::V4(Ipv4Addr::new(74, 125, 131, 27)),
                prefix: 24,
            },
            priority: 100,
        }];
        crate::map_setting::flow_wanip::create_inner_flow_match_map_v4(
            &skel.maps.flow4_ip_map,
            0,
            &data,
        )
        .unwrap();

        crate::map_setting::route::add_wan_route_inner_v4(
            &skel.maps.rt4_target_map,
            5,
            &RouteTargetInfo {
                weight: 0,
                ifindex: 11,
                has_mac: true,
                default_route: false,
                is_docker: false,
                iface_name: "test".to_string(),
                iface_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                gateway_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            },
        );

        let repeat = 100_000;
        let route_lan_ingress = skel.progs.route_lan_ingress;

        let input = ProgramInput {
            data_in: Some(&mut simple_tcp_syn()),
            context_in: None,
            context_out: None,
            data_out: None,
            repeat,
            ..Default::default()
        };
        let lan_result = route_lan_ingress.test_run(input).expect("test_run failed");

        println!("lan_result: {}", lan_result.return_value as i32);
        println!("lan duration: {:?}", lan_result.duration);
    }
}
