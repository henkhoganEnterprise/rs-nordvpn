use log;
use clap::Parser;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::{io::Write, process::Command};
use tokio_util::sync::CancellationToken;


#[path = "./admin/mod.rs"]
mod admin;
#[path = "./nordvpn/mod.rs"]
mod nordvpn;
mod proxy;
mod tokiort;


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliArgs {
    
    #[arg(short, long)]
    token: String,
    #[clap(default_value_t = 80)]
    admin_port: u16,
    #[clap(default_value_t = 3128)]
    proxy_port: u16,
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


    let nordvpn = match nordvpn::NordVPN::new(nordvpn_path, args.token) {
        Ok(nordvpn) => nordvpn,
        Err(err) => {
            log::error!("Failed to create NordVPN instance: {}", err);
            std::process::exit(1);
        }
    };
    nordvpn.daemon_start(30);
    std::thread::sleep(std::time::Duration::from_secs(10));
    nordvpn.daemon_status();
    nordvpn.set_analytics(false);
    nordvpn.set_firewall(false);
    nordvpn.set_routing(false);

    log::info!("NordVPN version: {}", nordvpn.version());
    nordvpn.login();
    nordvpn.account();
    nordvpn.connect();
    nordvpn.status();

    let _admin = match admin::Admin::new(nordvpn) {
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
    let admin_task = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = admin_token.cancelled() => {
                // The token was cancelled, task can shut down
                log::info!("Proxy task was cancelled");
            }
            _ = admin::run(args.admin_port, _admin) => {
                // Long work has completed
            }
        }
    });

    // Task 2 - Wait for token cancellation or a long time
    let proxy_token = token.clone();
    let proxy_task = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = proxy_token.cancelled() => {
                // The token was cancelled, task can shut down
                log::info!("Proxy task was cancelled");
            }
            _ = proxy::run(args.proxy_port) => {
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
    //admin_task.await.unwrap();
    proxy_task.await.unwrap();

    
}
