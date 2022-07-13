pub mod measurement;
pub mod receiver;

use std::net::IpAddr;

use uuid::Uuid;

pub const CONTROL_PORT: u16 = 7224;
pub const DATA_PORT: u16 = 7225;

pub struct Node {
    pub id: Uuid,
    // TODO: Multiple IPs per node
    pub address: IpAddr,
}

pub struct Network {
    pub nodes: Vec<Node>,
}
