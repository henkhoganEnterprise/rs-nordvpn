use std::{io::stdout, process::Command};

use clap::builder::Str;


pub struct NordVPN {
    path: String,
    token: String,
}

impl NordVPN {
    pub fn new(path: String, token: String) -> Result<Self, &'static str> {
        log::info!("Creating new NordVPN instance");
        return Ok(Self {
            path,
            token: token
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

    pub fn connect(&self) -> bool {
        log::debug!("Connecting to NordVPN...");
        let output = self._nordvpn_command(vec!["connect".to_string()]);
        return true;
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

    pub fn daemon_status(&self) -> bool {
        let stdout = Command::new("/etc/init.d/nordvpn")
            .arg("status")
            .output()
            .expect("Failed to execute command")
            .stdout;
        log::info!("NordVPN service status: {}", String::from_utf8_lossy(&stdout).trim());
        return true;
    }

    pub fn daemon_start(&self, timeout: u8) {
        Command::new("/etc/init.d/nordvpn")
            .arg("start")
            .output()
            .expect("Failed to execute command");
    }

    pub fn daemon_stop(&self) {
        Command::new("/etc/init.d/nordvpn")
            .arg("stop")
            .output()
            .expect("Failed to execute command");
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

