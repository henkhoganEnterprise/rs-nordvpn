use log;
use std::env;

mod nordvpn;

fn main() {
    log::info!("Starting NordVPN...");

    let token = env::args().nth(1).expect("Token not provided");
    let nordvpn = nordvpn::NordVPN::new("sh".to_string(), token).unwrap();

    nordvpn.login();
    nordvpn.connect();
    nordvpn.status();
}
