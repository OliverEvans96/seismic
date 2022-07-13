pub mod measurement;
pub mod measurer;
pub mod receiver;

use std::net::IpAddr;

use uuid::Uuid;

pub struct Node {
    pub id: Uuid,
    // TODO: Multiple IPs per node
    pub address: IpAddr,
}

pub struct Network {
    pub nodes: Vec<Node>,
}
