use admin::Admin;
use log;
use clap::Parser;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::{collections::{HashMap, HashSet}, io::Write, process::Command, str::FromStr, sync::{Arc, RwLock}, u16, vec};
use tokio_util::sync::CancellationToken;
use std::net::SocketAddr;


#[path = "./admin/mod.rs"]
mod admin;
#[path = "./nordvpn/mod.rs"]
mod nordvpn;

#[path = "./helper/mod.rs"]
mod helper;
use helper::CurlClient;

#[path = "./proxy/mod.rs"]
mod proxy;
use proxy::ProxyState;

mod tokiort;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliArgs {
    
    #[arg(short, long)]
    token: String,
    #[arg(short, long)]
    #[clap(default_value_t = 80)]
    admin_port: u16,
    #[arg(short, long)]
    #[clap(default_value_t = 3128)]
    proxy_port: u16,
    #[arg(short, long)]
    #[clap(default_value_t = String::from("0.0.0.0"))]
    bind_ip: String,
    #[arg(short, long)]
    #[clap(default_values_t = Vec::<String>::new())]
    monitored_hosts: Vec<String>,
    #[arg(short, long)]
    #[clap(default_values_t = Vec::<String>::new())]
    filter: Vec<String>,
    #[arg(short, long)]
    #[clap(default_value_t = 0)]
    proxy_rotation_interval: u16
}



#[tokio::main]
async fn main() {
    
    let args = CliArgs::parse();
    
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

    // args.filter = vec!["United States:1".to_string(), "Germany:2".to_string()];
    let filter_map: HashMap<u16, Vec<String>> = args.filter.clone().into_iter().enumerate().fold(HashMap::new(), |mut acc, (_i, filter)| {
        let slot = 0;
        if filter.contains(":" ) {
            let parts: Vec<&str> = filter.split(":").collect();
            let slot = parts[1].parse::<u16>().unwrap();
            let country = parts[0].to_string();
            acc.entry(slot).or_insert(vec![]).push(country);
        } else {
            acc.entry(slot).or_insert(vec![]).push(filter);
        }
        acc
    });
    log::info!("Filter Map: {:?}", filter_map);
    
    let filter_slot: Option<u16> = match std::env::var("NORDVPN_FILTER_SLOT") {
        Ok(value) => u16::from_str(&value).ok(),
        Err(_) => None,
    };
    log::info!("Filter slot: {:?}", filter_slot);

    let filters = match (filter_map.len(), filter_slot) {
        (0,_) => None,
        (1, _) => Some(filter_map[&0].clone()),
        (_, None) => None,
        (_, Some(filter_slot)) if filter_slot >= filter_map.len() as u16 => None,
        (_, Some(filter_slot)) => Some(filter_map[&filter_slot].clone())
    };
    log::info!("Filters: {:?}", filters);

    /*
    let filter = match filters {
        Some(filters) => Some(filters[0].clone()),
        None => None
    };
    log::info!("Filter: {:?}", filter);
     */
    

    let curl_client = CurlClient::new_with_path_discovery();
    log::info!("Native Public IP: {}", curl_client.get("https://api.ipify.org").unwrap());

    log::info!("Starting NordVPN...");

    let nordvpn_path_out = Command::new("which")
        .arg("nordvpn")
        .output()
        .expect("Failed to execute command");

    let nordvpn_path: String;
    if nordvpn_path_out.status.success() {
        nordvpn_path = std::str::from_utf8(&nordvpn_path_out.stdout).unwrap().trim().to_string();
        log::info!("NordVPN found in path: {}", nordvpn_path);
    } else {
        log::error!("NordVPN not found in path");
        std::process::exit(1);
    }


    let filter_set: Option<HashSet<String>> = filters.map(|filter| filter.into_iter().collect());
    let nordvpn = match nordvpn::NordVPN::new(nordvpn_path, args.token.clone(), filter_set) {
        Ok(nordvpn) => nordvpn,
        Err(err) => {
            log::error!("Failed to create NordVPN instance: {}", err);
            std::process::exit(1);
        }
    };
    nordvpn.daemon_start(Some(30));
    nordvpn.daemon_status();
    nordvpn.set_analytics(false);
    nordvpn.set_firewall(true);
    nordvpn.set_routing(true);
    nordvpn.set_lan_discovery(true);
    //nordvpn.set_tray(false);
    //nordvpn.set_virtual_location(false);

    log::info!("NordVPN version: {}", nordvpn.version());
    nordvpn.login();
    nordvpn.account();
    if let Err(e) = nordvpn.connect(None) {
        log::error!("{:?}", e);
        std::process::exit(1)
    }
    nordvpn.status();
    log::info!("Public IP: {}", curl_client.get("https://api.ipify.org").unwrap());

    log::info!("Monitored hosts: {:?}", args.monitored_hosts);
    log::info!("Proxy rotation interval: {}", args.proxy_rotation_interval);

    let proxy_state = Arc::new(RwLock::new(ProxyState::new(nordvpn, args.monitored_hosts, args.proxy_rotation_interval)));
    
    let admin = match Admin::new(curl_client, proxy_state.clone()) {
        Ok(_admin) => _admin,
        Err(err) => {
            log::error!("Failed to create NordAdminVPN instance: {}", err);
            std::process::exit(1);
        }
    };
 
 
    let token = CancellationToken::new();
    
    let admin_token = token.clone();
    let admin_addr = SocketAddr::from_str(&format!("{}:{}", args.bind_ip.clone(), args.admin_port)).unwrap();
    let admin_task = tokio::spawn(async move {
        tokio::select! {
            _ = admin_token.cancelled() => {
                // The token was cancelled, task can shut down
                log::info!("Proxy task was cancelled");
            }
            _ = admin::run(admin_addr, admin) => {
                // Long work has completed
            }
        }
    });


    let proxy_token = token.clone();
    let proxy_addr = SocketAddr::from_str(&format!("{}:{}", args.bind_ip.clone(), args.proxy_port)).unwrap();
    let proxy_task = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = proxy_token.cancelled() => {
                // The token was cancelled, task can shut down
                log::info!("Proxy task was cancelled");
            }
            _ = proxy::proxy_functions::run(proxy_state.clone(),proxy_addr) => {
                // Long work has completed
            }
        }
    });


    use tokio::signal::unix::{signal, SignalKind};

    // Infos here:
    // https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html
    let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
    let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
    _ = signal_terminate.recv() => {
        log::info!("Received SIGTERM.");
        token.cancel();
        //proxy_task.abort();
    },
        _ = signal_interrupt.recv() => {
        log::info!("Received SIGINT.");
        token.cancel();
        //proxy_task.abort();
    }
    };
 

    // Wait for tasks to complete
    proxy_task.await.unwrap();
    admin_task.await.unwrap();

    
}
