pub mod discovery;
pub mod http;

use crate::runtime::AppRuntime;
use std::{net::Ipv4Addr, sync::Arc};

pub const PORT: u16 = 8899;
pub const MULTICAST_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 99);
pub const MAGIC_DISCOVER: &str = "SMSBRIDGE_DISCOVER";
pub const MAGIC_READY: &str = "SMSBRIDGE_READY";

pub fn start(runtime: Arc<AppRuntime>) {
    http::start(runtime.clone());
    discovery::start(runtime);
}
