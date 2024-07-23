use std::process::Command;

use daemon::StatusOutput;


mod daemon;

#[derive(Debug, Clone)]
pub struct NordVpnConnectOutput {
    pub connected: bool,
    pub group: String,
    pub server_id: String,
    pub server_fqdn: String
}

#[derive(Debug, Clone)]
pub struct NordVPN {
    path: String,
    token: String,
    daemon: daemon::Daemon
}

impl NordVPN {
    pub fn new(path: String, token: String) -> Result<Self, &'static str> {
        log::info!("Creating new NordVPN instance");
        return Ok(Self {
            path,
            token,
            daemon: daemon::Daemon::new()?
        });
    }

    fn _nordvpn_command(&self, args: Vec<String>) -> (bool, String) {

        let mut _cmd = Command::new(self.path.clone());

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



    pub fn account(&self) -> bool {
        log::debug!("Checking NordVPN account...");
        let output = self._nordvpn_command(vec!["account".to_string()]);
        if output.0 {
            log::info!("Account: {}", output.1);
        } else {
            log::error!("Failed to fetch account: {}", output.1);
        }
        output.0
    }

    fn parse_connect_output(&self, output: String) -> NordVpnConnectOutput {
        /*
        Connecting to Australia #600 (au600.nordvpn.com)
        You are connected to Australia #600 (au600.nordvpn.com)!
         */
        let connected;
        let group: String;
        let server_id: String;
        let server_fqdn: String;
        let substring = "You are connected to ";

        let mut lines = output.lines();
        let _first_line = lines.next().unwrap();
        let second_line = lines.next().unwrap();
        if second_line.contains(substring) {
            connected = true;
            let infos = second_line.split(substring).collect::<Vec<&str>>()[1];
            let infos = infos.split(" ").collect::<Vec<&str>>();
            group = infos[0].to_string();
            server_id = infos[1].to_string();
            server_fqdn= infos[2].replace("(", "").replace(")", "");

        }
        else {
            connected = false;
            group = "".to_string();
            server_id = "".to_string();
            server_fqdn = "".to_string();
        }
        
        
        return NordVpnConnectOutput {
            connected,
            group,
            server_id,
            server_fqdn
        };
    
        
    }

    pub fn connect(&self) -> Result<NordVpnConnectOutput, ()> {
        log::debug!("Connecting to NordVPN...");
        let output = self._nordvpn_command(vec!["connect".to_string()]);
        if output.0 {
            log::info!("Connected: {}", output.1);
            return Ok(self.parse_connect_output(output.1));
        } else {
            log::error!("Failed to connect: {}", output.1.clone());
            return Err(());
        }
    }

    pub fn connect_with_argument(&self, argument: &str) -> Result<NordVpnConnectOutput, ()> {
        log::debug!("Connecting to NordVPN...");
        let output = self._nordvpn_command(vec!["connect".to_string(), argument.to_string()]);
        if output.0 {
            log::info!("Connected: {}", output.1);
            return Ok(self.parse_connect_output(output.1));
        } else {
            log::error!("Failed to connect: {}", output.1.clone());
            return Err(());
        }
    }

    pub fn disconnect(&self) -> bool {
        log::debug!("Disconnecting from NordVPN...");
        let output = self._nordvpn_command(vec!["disconnect".to_string()]);
        if output.0 {
            log::info!("Connected: {}", output.1);
        } else {
            log::error!("Failed to disconnect: {}", output.1);
        }
        output.0
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

    pub fn set_routing(&self, enabled: bool) -> bool {
        let output = self._nordvpn_command(vec!["set".to_string(), "routing".to_string(), enabled.to_string()]);
        if output.0 {
            log::info!("Routing: {}", output.1);
        } else {
            log::error!("Failed to set routing to {}: {}", enabled, output.1);
        }
        output.0
    }


    pub fn status(&self) -> bool {
        log::debug!("Checking NordVPN status...");
        let output = self._nordvpn_command(vec!["status".to_string()]);
        if output.0 {
            log::info!("Status: {}", output.1);
        } else {
            log::error!("Failed to fetch status: {}", output.1);
        }
        output.0
    }

    pub fn version(&self) -> String {
        log::debug!("Checking NordVPN version...");
        let output = self._nordvpn_command(vec!["version".to_string()]);
        return output.1;
    }



}

impl Drop for NordVPN {
    fn drop(&mut self) {
        self.disconnect();
        log::info!("Dropping NordVPN instance");
    }
}

