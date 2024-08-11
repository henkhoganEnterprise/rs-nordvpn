use log;
use clap::Parser;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::{io::Write, process::Command, str::FromStr, sync::{Arc, Mutex}, vec};
use tokio_util::sync::CancellationToken;
use std::net::SocketAddr;


#[path = "./admin/mod.rs"]
mod admin;
#[path = "./nordvpn/mod.rs"]
mod nordvpn;

#[path = "./helper/mod.rs"]
mod helper;
use helper::CurlClient;

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
    monitored_hosts: Vec<String>
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


    let nordvpn = match nordvpn::NordVPN::new(nordvpn_path, args.token.clone()) {
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
    nordvpn.connect().unwrap();
    nordvpn.status();
    log::info!("Public IP: {}", curl_client.get("https://api.ipify.org").unwrap());


    let proxy_state = ProxyState::new(args.monitored_hosts);
    let proxy_state_mutex = Mutex::new(proxy_state);
    let proxy_state_mutex_arc = Arc::new(proxy_state_mutex);
    

   


    
    let _admin = match admin::Admin::new(curl_client, nordvpn, proxy_state_mutex_arc.clone()) {
        Ok(_admin) => _admin,
        Err(err) => {
            log::error!("Failed to create NordAdminVPN instance: {}", err);
            std::process::exit(1);
        }
    };

    // Step 1: Create a new CancellationToken
    let token = CancellationToken::new();

    
    // Task 1 - Wait for token cancellation or a long time
    let admin_token = token.clone();
    let admin_addr = SocketAddr::from_str(&format!("{}:{}", args.bind_ip.clone(), args.admin_port)).unwrap();
    let admin_task = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = admin_token.cancelled() => {
                // The token was cancelled, task can shut down
                log::info!("Proxy task was cancelled");
            }
            _ = admin::run(admin_addr, Arc::new(_admin)) => {
                // Long work has completed
            }
        }
    });

    // Task 2 - Wait for token cancellation or a long time
    let proxy_token = token.clone();
    let proxy_addr = SocketAddr::from_str(&format!("{}:{}", args.bind_ip.clone(), args.proxy_port)).unwrap();
    let proxy_task = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = proxy_token.cancelled() => {
                // The token was cancelled, task can shut down
                log::info!("Proxy task was cancelled");
            }
            _ = proxy::run(proxy_state_mutex_arc.clone(),proxy_addr) => {
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
