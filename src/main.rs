use log;
use clap::Parser;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::{io::Write, process::Command};
use tokio_util::sync::CancellationToken;

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


    let nordvpn = nordvpn::NordVPN::new(nordvpn_path, args.token).unwrap();
    nordvpn.daemon_start(30);
    std::thread::sleep(std::time::Duration::from_secs(10));
    nordvpn.daemon_status();

    log::info!("NordVPN version: {}", nordvpn.version());
    nordvpn.login();
    nordvpn.account();
    nordvpn.connect();
    nordvpn.status();

    let mut proxy = proxy::Proxy::new();
    
    // Step 1: Create a new CancellationToken
    let token = CancellationToken::new();

    // Step 2: Clone the token for use in another task
    let cloned_token = token.clone();

    // Task 1 - Wait for token cancellation or a long time
    let proxy_task = tokio::spawn(async move {
        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = cloned_token.cancelled() => {
                // The token was cancelled, task can shut down
                log::info!("Proxy task was cancelled");
            }
            _ = proxy.start() => {
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

    
}
