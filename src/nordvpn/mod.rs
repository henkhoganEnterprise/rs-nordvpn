use std::{collections::{HashSet, VecDeque}, process::Command};
use dns_lookup;

use daemon::StatusOutput;
use serde_derive::{Deserialize, Serialize};


mod daemon;

#[derive(Debug, Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]
pub enum Error {
    AccountCommandFailed,
    ConnectFailed,
    DnsLookupFailed,
    FilterDisabled,
    ParseFailed
}

#[derive(Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]

pub struct NordVpnConnectOutput {
    pub connected: bool,
    pub group: String,
    pub server_id: String,
    pub server_fqdn: String,
    pub error: Option<Error>
}

#[derive(Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]
pub struct NordVpnStatusOutput {
    pub connected: bool,
    pub server: String,
    pub hostname: String,
    pub ip: String,
    pub country: String,
    pub city: String,
    pub technology: String,
    pub protocol: String,
    pub post_quantum: bool,
    pub transfer: String,
    pub uptime: String
}

#[derive(Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]
pub struct NordVpnDisconnectOutput {
    pub disconnected: bool
}

#[derive(Debug, Clone)]
pub struct NordVPN {
    path: String,
    token: String,
    daemon: daemon::Daemon,
    dns_lookup: Option<String>,
    filter_enabled: bool,
    filter: VecDeque<String>,
}

impl NordVPN {
    pub fn new(path: String, token: String, filter: Option<HashSet<String>>, dns_lookup: Option<String>) -> Result<Self, &'static str> {
        log::info!("Creating new NordVPN instance");
        return Ok(Self {
            path,
            token,
            daemon: daemon::Daemon::new()?,
            dns_lookup,
            filter_enabled: filter == None,
            filter: match filter {
                None => VecDeque::<String>::new(),
                Some(set) => set.into_iter().collect(),
            }
        });
    }

    fn _nordvpn_command(&self, args: Vec<String>) -> (bool, String) {

        let mut _cmd: Command = Command::new(self.path.clone());

        _cmd
            .args(args);

        log::debug!("Running command: {:?}",_cmd);

        let _output = _cmd
            .output()
            .expect("Failed to execute command");

        if _output.status.success() {
            log::debug!("Command executed successfully");
            return (true, String::from_utf8_lossy(&_output.stdout).to_string());
        } else {
            log::error!("Command execution failed: {:?}", format!("{:?}", _output));
            return (false, String::from_utf8_lossy(&_output.stderr).trim().to_string());
        }
    }

    pub fn account(&self) -> Result<String, Error> {
        log::debug!("Checking NordVPN account...");
        let output = self._nordvpn_command(vec!["account".to_string()]);
        if output.0 {
            log::info!("Account: {}", output.1);
            return Ok(output.1);
        } else {
            log::error!("Failed to fetch account: {}", output.1);
            return Err(Error::AccountCommandFailed);
        }
    }

    fn parse_connect_output(&self, output: String) -> NordVpnConnectOutput {

        /*
        Connecting to Australia #600 (au600.nordvpn.com)
        You are connected to Australia #600 (au600.nordvpn.com)!
         */
        /*
        Connecting to Germany #1078 (de1078.nordvpn.com)
        You are connected to Germany #1078 (de1078.nordvpn.com)!
        */
        let connected_substring = "You are connected to ";
        /*
        Connecting to Germany #1087 (de1087.nordvpn.com)
        The VPN connection has failed. Please check your internet connection and try connecting to the VPN again. If the issue persists, contact our customer support.
        */
        let not_connected_substring = "The VPN connection has failed. Please check your internet connection and try connecting to the VPN again. If the issue persists, contact our customer support.";
        

        for line in output.lines() {
            if line.contains(connected_substring) {
                let infos = line.split(connected_substring).collect::<Vec<&str>>()[1];
                let infos = infos.split(" ").collect::<Vec<&str>>();
                let group = infos[0].to_string();
                let server_id = infos[1].to_string();
                let server_fqdn= infos[2].replace("(", "").replace(")!", "");
                return NordVpnConnectOutput {
                    connected: true,
                    group,
                    server_id,
                    server_fqdn,
                    error: None
                };
            }
            if line.contains(not_connected_substring) {
                return NordVpnConnectOutput {
                    connected: false,
                    group: "".to_string(),
                    server_id: "".to_string(),
                    server_fqdn: "".to_string(),
                    error: Some(Error::ConnectFailed)
                };
            }
        }

    
        log::warn!("Failed to parse connect output:\n{}\n-> assuming disconnected", output);
        
        return NordVpnConnectOutput {
            connected: false,
            group: "".to_string(),
            server_id: "".to_string(),
            server_fqdn: "".to_string(),
            error: Some(Error::ParseFailed)

        };
    
        
    }

    pub fn connect(&self, filter: Option<String>) -> Result<NordVpnConnectOutput, NordVpnConnectOutput> {
        log::debug!("Connecting to NordVPN...");
        let output = match filter {
            Some(filter) => self._nordvpn_command(vec!["connect".to_string(), filter]),
            None => self._nordvpn_command(vec!["connect".to_string()])
        };
        if output.0 {
            log::info!("Connected: {}", output.1);
            let parsed_output = self.parse_connect_output(output.1);
            let dns_error ;
            match dns_lookup::lookup_host("google.com") {
                Ok(lookup) => {
                    log::info!("DNS Lookup for google.com: {:?}", lookup);
                    dns_error = false;
                },
                Err(e) => {
                    log::error!("Failed to perform DNS lookup for google.com");
                    dns_error = true;
                }
            }

            match dns_error {
                false => {
                    return Ok(parsed_output);
                },
                true => {
                    return Err(NordVpnConnectOutput {
                        connected: parsed_output.connected,
                        group: parsed_output.group,
                        server_id: parsed_output.server_id,
                        server_fqdn: parsed_output.server_fqdn,
                        error: Some(Error::DnsLookupFailed)
                    });
                }
            }
        } 
        log::error!("Failed to connect: {}", output.1.clone());
        return Err(NordVpnConnectOutput {
            connected: false,
            group: "".to_string(),
            server_id: "".to_string(),
            server_fqdn: "".to_string(),
            error: Some(Error::ConnectFailed)
        });
    }

    pub fn disconnect(&self) -> Result<NordVpnDisconnectOutput, NordVpnDisconnectOutput> {
        log::debug!("Disconnecting from NordVPN...");
        let output = self._nordvpn_command(vec!["disconnect".to_string()]);
        if output.0 {
            log::info!("Disonnected: {}", output.1);
            return Ok(NordVpnDisconnectOutput {
                disconnected: true
            });
        } else {
            log::error!("Failed to disconnect: {}", output.1);
            return Err(NordVpnDisconnectOutput {
                disconnected: false
            });
        }
    }

    pub fn login(&self) -> bool {
        log::debug!("Logging in to NordVPN...");
        let output = self._nordvpn_command(vec!["login".to_string(), "--token".to_string(), self.token.clone()]);
        if output.0 {
            log::info!("Logged in successfully: {}", output.1);
        } else {
            log::error!("Failed to log in: {}", output.1);
        }
        output.0
    }

    pub fn daemon_status(&self) -> StatusOutput {
        let cmd_output = self.daemon.status();
        log::info!("NordVPN service status: {:?}", cmd_output.status);
        return cmd_output;
    }

    pub fn daemon_restart(&self, timeout: Option<u8>) {
        Command::new("/etc/init.d/nordvpn")
            .arg("restart")
            .output()
            .expect("Failed to execute command");
    }

    pub fn daemon_start(&self, timeout: Option<u8>) {
        let cmd_output = self.daemon.start(timeout);
    }

    pub fn daemon_stop(&self) {
        Command::new("/etc/init.d/nordvpn")
            .arg("stop")
            .output()
            .expect("Failed to execute command");
    }

    pub fn logs(&self, lines: u16) -> Vec<u8> {
        log::debug!("Checking NordVPN logs...");
        return Command::new("tail")
            .arg("-n")
            .arg(lines.to_string()) 
            .arg("/var/log/nordvpn/daemon.log")
            .output()
            .expect("Failed to execute command")
            .stdout
    }

    pub fn rotate(&mut self, retries: u8) -> Result<NordVpnConnectOutput, NordVpnConnectOutput> {
        if !self.filter_enabled || self.filter.len() == 0 {
            return self.connect(None);
        }
        let filter = self.filter.pop_front().unwrap();
        let o = self.connect(Some(filter.clone()));
        self.filter.push_back(filter);
        o
    }

    pub fn set_analytics(&self, enabled: bool) -> bool {
        let output = self._nordvpn_command(vec!["set".to_string(), "analytics".to_string(), enabled.to_string()]);
        if output.0 {
            log::info!("Analytics: {}", output.1);
        } else {
            log::error!("Failed to set analytics to {}: {}", enabled, output.1);
        }
        output.0
    }

    pub fn set_firewall(&self, enabled: bool) -> bool {
        let output = self._nordvpn_command(vec!["set".to_string(), "firewall".to_string(), enabled.to_string()]);
        if output.0 {
            log::info!("Firewall: {}", output.1);
        } else {
            log::error!("Failed to set firewall to {}: {}", enabled, output.1);
        }
        output.0
    }

    pub fn set_lan_discovery(&self, enabled: bool) -> bool {
        let output = self._nordvpn_command(vec!["set".to_string(), "lan-discovery".to_string(), enabled.to_string()]);
        if output.0 {
            log::info!("LAN Discovery: {}", output.1);
        } else {
            log::error!("Failed to set lan-discovery to {}: {}", enabled, output.1);
        }
        output.0
    }

    pub fn set_routing(&self, enabled: bool) -> bool {
        let output = self._nordvpn_command(vec!["set".to_string(), "routing".to_string(), enabled.to_string()]);
        if output.0 {
            log::info!("Routing: {}", output.1);
        } else {
            log::error!("Failed to set routing to {}: {}", enabled, output.1);
        }
        output.0
    }

    pub fn set_tray(&self, enabled: bool) -> bool {
        let output = self._nordvpn_command(vec!["set".to_string(), "tray".to_string(), enabled.to_string()]);
        if output.0 {
            log::info!("Tray: {}", output.1);
        } else {
            log::error!("Failed to set tray to {}: {}", enabled, output.1);
        }
        output.0
    }

    pub fn set_virtual_location(&self, enabled: bool) -> bool {
        let output = self._nordvpn_command(vec!["set".to_string(), "virtual-location".to_string(), enabled.to_string()]);
        if output.0 {
            log::info!("Location: {}", output.1);
        } else {
            log::error!("Failed to set virtual-location to {}: {}", enabled, output.1);
        }
        output.0
    }

    fn parse_status_output(&self, output: String) -> NordVpnStatusOutput {
        /*        
        Status: Connected
        Server: Germany #1099
        Hostname: de1099.nordvpn.com
        IP: 194.233.96.241
        Country: Germany
        City: Berlin
        Current technology: NORDLYNX
        Current protocol: UDP
        Transfer: 92 B received, 180 B sent
        Uptime: 0 seconds
        */
        let mut connected = false;
        let mut server = String::new();
        let mut hostname = String::new();
        let mut ip = String::new();
        let mut country = String::new();
        let mut city = String::new();
        let mut technology = String::new();
        let mut protocol = String::new();
        let mut post_quantum = false;
        let mut transfer = String::new();
        let mut uptime = String::new();

        for line in output.lines() {
            if line.starts_with("Status") {
                if line.contains("Connected") {
                    connected = true;
                } else {
                    connected = false;
                }
            } else if line.starts_with("Server: ") {
                server = line.replace("Server: ", "").to_string();
            } else if line.starts_with("Hostname: ") {
                hostname = line.replace("Hostname: ", "").to_string();
            } else if line.starts_with("IP: ") {
                ip = line.replace("IP: ", "").to_string();
            } else if line.starts_with("Country: ") {
                country = line.replace("Country: ", "").to_string();
            } else if line.starts_with("City: ") {
                city = line.replace("City: ", "").to_string();
            } else if line.starts_with("Current technology: ") {
                technology = line.replace("Current technology: ", "").to_string();
            } else if line.starts_with("Current protocol: ") {
                protocol = line.replace("Current protocol: ", "").to_string();
            } else if line.starts_with("Post-quantum VPN: ") {
                if line.contains("Disabled") {
                    post_quantum = false;
                } else {
                    post_quantum = true;
                }
            } else if line.starts_with("Transfer: ") {
                transfer = line.replace("Transfer: ", "").to_string();
            } else if line.starts_with("Uptime: ") {
                uptime = line.replace("Uptime: ", "").to_string();
            }
        }

        return NordVpnStatusOutput {
            connected,
            server,
            hostname,
            ip,
            country,
            city,
            technology,
            protocol,
            post_quantum,
            transfer,
            uptime,
        };
    }


    pub fn status(&self) -> NordVpnStatusOutput {
        log::debug!("Checking NordVPN status...");
        let output = self._nordvpn_command(vec!["status".to_string()]);
        if output.0 {
            log::info!("{}", output.1);
            return self.parse_status_output(output.1);
        }
        
        log::error!("Failed to fetch status: {}", output.1);
        return NordVpnStatusOutput {
            connected: false,
            server: "".to_string(),
            hostname: "".to_string(),
            ip: "".to_string(),
            country: "".to_string(),
            city: "".to_string(),
            technology: "".to_string(),
            protocol: "".to_string(),
            post_quantum: false,
            transfer: "".to_string(),
            uptime: "".to_string(),
        };
    }

    pub fn version(&self) -> String {
        log::debug!("Checking NordVPN version...");
        let output = self._nordvpn_command(vec!["version".to_string()]);
        return output.1;
    }



}

impl Drop for NordVPN {
    fn drop(&mut self) {
        let _ = self.disconnect();
        log::info!("Dropping NordVPN instance");
    }
}

