use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use netlink_packet_core::NetlinkMessage;
use netlink_packet_core::NetlinkPayload;
use netlink_packet_core::NLM_F_ACK;
use netlink_packet_core::NLM_F_DUMP;
use netlink_packet_core::NLM_F_REQUEST;
use netlink_packet_route::link::LinkMessage;
use netlink_packet_route::nsid::NsidAttribute;
use netlink_packet_route::nsid::NsidHeader;
use netlink_packet_route::nsid::NsidMessage;
use netlink_packet_route::AddressFamily;
use netlink_packet_route::RouteNetlinkMessage;
use netlink_sys::{AsyncSocket, SocketAddr};
use rtnetlink::constants::RTMGRP_IPV4_IFADDR;
use rtnetlink::constants::RTMGRP_IPV4_ROUTE;
use rtnetlink::constants::RTMGRP_IPV6_IFADDR;
use rtnetlink::constants::RTMGRP_IPV6_ROUTE;
use rtnetlink::{constants::RTMGRP_LINK, new_connection, Handle};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Open the netlink socket
    let (mut connection, mut handle, mut messages) =
        new_connection().map_err(|e| format!("{e}"))?;

    // // Listen for link changes
    let mgroup_flags = RTMGRP_LINK
        // | RTMGRP_IPV4_IFADDR
        // | RTMGRP_IPV4_ROUTE
        // | RTMGRP_IPV6_IFADDR
        // | RTMGRP_IPV6_ROUTE
        ;

    // let addr = SocketAddr::new(0, mgroup_flags);
    // connection.socket_mut().socket_mut().bind(&addr).expect("failed to bind");
    tokio::spawn(connection);

    // while let Some((message, _)) = messages.next().await {
    //     // println!("Route change message - {message:?}");
    //     let result = landscape::observer::filter_message_status(message);
    //     println!("result - {result:?}");
    //     // match message.payload {
    //     //     NetlinkPayload::InnerMessage(inner_message) => {
    //     //         // 处理 InnerMessage
    //     //         println!("Received Inner message: {:?}", inner_message);
    //     //     }
    //     //     _ => todo!(),
    //     // }
    // }
    
    // let mut nsid_msg = NsidMessage::default();
    // nsid_msg.header = NsidHeader { family: AddressFamily::Netlink };
    // nsid_msg.attributes = vec![
    //     NsidAttribute::Id(-1),
    //     NsidAttribute::Pid(0),
    //     NsidAttribute::Fd(4026532359),
    //     NsidAttribute::TargetNsid(-1),
    //     NsidAttribute::CurrentNsid(-1),
    // ];
    // let message = RouteNetlinkMessage::GetNsId(nsid_msg);
    // let mut req = NetlinkMessage::from(message);
    // req.header.flags = NLM_F_REQUEST | NLM_F_ACK;
    // // req.header.sequence_number = 1;
    // req.finalize();

    // println!("{:#?}", req);
    // match handle.request(req) {
    //     Ok(mut response) => {
    //         //
    //         while let Some(msg) = response.next().await {
    //             println!("{:#?}", msg);
    //         }
    //     }
    //     Err(_) => todo!(),
    // }

    // dump_addresses(&handle, "ens3".to_string()).await;

    // dump_addresses(&handle, "ens5".to_string()).await;
    // dump_addresses(&handle, "ens4".to_string()).await;
    // flush_addresses(handle, "veth-host".to_string()).await;
    Ok(())
}

async fn flush_addresses(handle: &Handle, link: String) -> () {
    let mut links = handle.link().get().match_name(link.clone()).execute();
    if let Some(link) = links.try_next().await.unwrap() {
        // We should have received only one message
        assert!(links.try_next().await.unwrap().is_none());

        let mut addresses =
            handle.address().get().set_link_index_filter(link.header.index).execute();
        while let Some(addr) = addresses.try_next().await.unwrap() {
            handle.address().del(addr).execute().await.unwrap();
        }
        ()
    } else {
        eprintln!("link {link} not found");
        ()
    }
}

async fn dump_addresses(handle: &Handle, link: String) {
    println!("dumping address for link \"{link}\"");
    let mut links = handle.link().get().match_name(link.clone()).execute();
    if let Some(link) = links.try_next().await.unwrap() {
        let mut addresses =
            handle.address().get().set_link_index_filter(link.header.index).execute();
        while let Some(msg) = addresses.try_next().await.unwrap() {
            println!("{msg:?}");
        }
    } else {
        eprintln!("link {link} not found");
    }
}
