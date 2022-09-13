pub mod client;
pub mod crypto;
pub mod error;
pub mod network;
pub mod transaction;
pub mod wallet;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {}
