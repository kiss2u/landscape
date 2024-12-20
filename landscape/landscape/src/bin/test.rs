use std::time::Duration;

use pnet::datalink::{self, NetworkInterface};
#[tokio::main]
async fn main() {
    // 要绑定的网桥名
    let iface_name = "veth1_pc3".to_string();
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

    let clone_interface = interface.clone();
    loop {
        println!("send packet");
        // let packet: Vec<u8> =
        //     vec![255, 255, 255, 255, 255, 255, 26, 106, 155, 111, 54, 43, 136, 100, 1, 2, 3, 4];
        let packet: Vec<u8> = vec![
            255, 255, 255, 255, 255, 255, 2, 81, 140, 240, 243, 24, 136, 100, 17, 0, 3, 4, 5, 6, 0,
            5, 1, 2, 3, 4, 5,
        ];
        if let Err(e) = tx.send_to(&packet, Some(clone_interface.clone())).unwrap() {
            println!("err: {e:?}");
        }
        std::thread::sleep(Duration::from_secs(3));
    }
}
