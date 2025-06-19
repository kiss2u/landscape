use nat::NatEvent;
use serde::{Deserialize, Serialize};

use crate::metric::connect::{ConnectInfo, ConnectMetric};

pub mod dns;
pub mod nat;

#[derive(Debug, Serialize, Deserialize)]
pub enum LandscapeEvent {
    NAT(NatEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectMessage {
    Event(ConnectInfo),
    Metric(ConnectMetric),
}
