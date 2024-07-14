use log;
use clap::Parser;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;


mod nordvpn;
mod proxy;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliArgs {
    
    #[arg(short, long)]
    token: String,
}


#[tokio::main]
async fn main() {

    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();


    let args = CliArgs::parse();

    log::info!("Starting NordVPN...");

    let nordvpn = nordvpn::NordVPN::new("sh".to_string(), args.token).unwrap();

    nordvpn.login();
    nordvpn.connect();
    nordvpn.status();

    let proxy = proxy::Proxy::new();

    proxy.start().await;
}
