use std::vec;

use landscape::dump::eth::EthFram;
use pnet::datalink::{self, NetworkInterface};

use landscape_common::net::MacAddr;

#[tokio::main]
async fn main() {
    // 要绑定的网桥名
    let iface_name = "br0-test".to_string();
    use pnet::datalink::Channel::Ethernet;

    let interface_names_match = |iface: &NetworkInterface| iface.name == iface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap_or_else(|| panic!("No such network interface: {}", iface_name));

    println!("interface name: {:?}", interface);
    // Create a channel to receive on
    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("packetdump: unhandled channel type"),
        Err(e) => panic!("packetdump: unable to create channel: {}", e),
    };
    let mac = interface.mac.map(|mac| mac.octets()).map(|o| MacAddr::from(o));

    loop {
        let interface = interface.clone();
        match rx.next() {
            Ok(packet) => {
                println!("catch data: {:?}", packet);
                let result = EthFram::new(packet, mac);
                match result.eth_type {
                    landscape::dump::eth::EthL3Type::Raw(_, _) => {}
                    landscape::dump::eth::EthL3Type::Ipv4(ip_frame) => {
                        println!("ip_frame data: {}", serde_json::json!(&ip_frame));

                        // println!("ip_frame data ckeck sum:: {}", ip_frame.caculate_checksum());
                        // println!(
                        //     "ip_frame protocol ckeck sum: {}",
                        //     ip_frame.caculate_protocol_checksum()
                        // );
                        let mut ip_resp = ip_frame.get_response();
                        match ip_frame.protocol {
                            landscape::dump::ipv4::EthIpType::Udp(udp_frame) => {
                                println!("udp ckeck sum: {},", udp_frame.checksum);
                                let mut response_udp = udp_frame.get_response_empty();
                                match udp_frame.playload {
                                    landscape::dump::udp_packet::EthUdpType::Dhcp(dhcp) => {
                                        println!("{:?}", dhcp);
                                        if dhcp.op == 1 {
                                            let payload =
                                                landscape::dump::udp_packet::dhcp::gen_offer(*dhcp);
                                            let payload =
                                                landscape::dump::udp_packet::EthUdpType::Dhcp(
                                                    Box::new(payload),
                                                );
                                            response_udp.set_payload(payload);
                                            let udp = landscape::dump::ipv4::EthIpType::Udp(
                                                Box::new(response_udp),
                                            );

                                            ip_resp.set_payload(udp);
                                            let checksum = ip_resp.caculate_checksum();
                                            ip_resp.update_checksum(checksum);
                                            //
                                            // let mac1 = vec![255, 255, 255, 255, 255, 255];
                                            // pc2 mac
                                            let mac1 = vec![122, 195, 92, 213, 128, 10];
                                            let mac2 = vec![78, 69, 141, 110, 82, 11];

                                            let eth: Vec<u8> = [mac1, mac2, vec![8, 0]].concat();
                                            let pacet = [eth, ip_resp.as_payload()].concat();
                                            tx.send_to(&pacet, Some(interface));
                                        }
                                    }
                                    landscape::dump::udp_packet::EthUdpType::Raw(_) => todo!(),
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(e) => panic!("packetdump: unable to receive packet: {}", e),
        }
    }
}
