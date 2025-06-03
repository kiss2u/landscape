use firewall::FirewallMessage;
use nat::NatEvent;
use serde::{Deserialize, Serialize};

pub mod dns;
pub mod firewall;
pub mod nat;

#[derive(Debug, Serialize, Deserialize)]
pub enum LandscapeEvent {
    NAT(NatEvent),
    Firewall(FirewallMessage),
}
