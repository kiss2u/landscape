mod package;

#[cfg(test)]
pub mod tests {
    use crate::{
        route::wan::flow_route_bpf::FlowRouteSkelBuilder, tests::route::package::simple_tcp_syn,
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
    fn egress_wan_route() {
        let mut wan_route_open_object = MaybeUninit::zeroed();

        let wan_route_builder = FlowRouteSkelBuilder::default();

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
        crate::map_setting::flow_wanip::create_inner_flow_match_map(
            &skel.maps.flow_v_ip_map,
            0,
            data,
        )
        .unwrap();

        crate::map_setting::route::add_wan_route_inner(
            &skel.maps.rt_target_map,
            5,
            RouteTargetInfo::docker_new(11, "test").0,
        );

        let wan_route_egress = skel.progs.wan_route_egress;

        let repeat = 1;
        let egress_input = ProgramInput {
            data_in: Some(&mut simple_tcp_syn()),
            context_in: None,
            context_out: None,
            data_out: None,
            repeat,
            ..Default::default()
        };
        let wan_result = wan_route_egress.test_run(egress_input).expect("test_run failed");

        let lan_route_ingress = skel.progs.lan_route_ingress;

        let input = ProgramInput {
            data_in: Some(&mut simple_tcp_syn()),
            context_in: None,
            context_out: None,
            data_out: None,
            repeat,
            ..Default::default()
        };
        let lan_result = lan_route_ingress.test_run(input).expect("test_run failed");

        println!("lan_result: {}", lan_result.return_value as i32);
        println!("wan_result: {}", wan_result.return_value as i32);

        assert_eq!(lan_result.return_value as i32, 7);
        assert_eq!(wan_result.return_value as i32, 7);
    }
}
