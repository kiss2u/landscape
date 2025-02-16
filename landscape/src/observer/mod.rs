use landscape_common::observer::IfaceObserverAction;
use netlink_packet_core::{NetlinkMessage, NetlinkPayload};
use netlink_packet_route::{link::LinkFlag, RouteNetlinkMessage};
use netlink_sys::AsyncSocket;
use rtnetlink::{constants::RTMGRP_LINK, new_connection};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

pub async fn dev_observer() -> broadcast::Receiver<IfaceObserverAction> {
    let (tx, rx) = broadcast::channel(30);

    tokio::spawn(async move {
        let (mut connection, _, mut messages) =
            new_connection().map_err(|e| format!("{e}")).unwrap();
        let mgroup_flags = RTMGRP_LINK;

        let addr = netlink_sys::SocketAddr::new(0, mgroup_flags);
        connection.socket_mut().socket_mut().bind(&addr).expect("failed to bind");
        tokio::spawn(connection);
        while let Some((message, _)) = messages.next().await {
            // println!("Route change message - {message:?}");
            if let Some(msg) = filter_message_status(message) {
                if let Err(e) = tx.send(msg) {
                    println!("too many msg, drop this msg: {e:?}");
                }
            }
        }
    });
    rx
}

pub fn filter_message_status(
    message: NetlinkMessage<RouteNetlinkMessage>,
) -> Option<IfaceObserverAction> {
    match message.payload {
        NetlinkPayload::InnerMessage(inner_message) => {
            // println!("Received Inner message: {:?}", inner_message);
            match inner_message {
                RouteNetlinkMessage::NewLink(link_message) => {
                    if link_message.header.change_mask.contains(&LinkFlag::Up) {
                        let mut ifacename = None;
                        for attr in link_message.attributes {
                            match attr {
                                netlink_packet_route::link::LinkAttribute::IfName(iface_name) => {
                                    ifacename = Some(iface_name);
                                }
                                _ => {}
                            }
                        }

                        let Some(ifacename) = ifacename else {
                            return None;
                        };

                        let mut result = IfaceObserverAction::Down(ifacename.clone());
                        for attr in link_message.header.flags {
                            match attr {
                                LinkFlag::Up => {
                                    result = IfaceObserverAction::Up(ifacename);
                                    break;
                                }
                                _ => {}
                            }
                        }

                        Some(result)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}
