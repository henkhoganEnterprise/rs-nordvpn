use std::process::Command;


pub struct NordVPN {
    shell: String,
    token: String,
}

impl NordVPN {
    pub fn new(shell: String, token: String) -> Result<Self, &'static str> {
        log::info!("Creating new NordVPN instance");
        return Ok(Self {
            shell,
            token: token
        });
    }

    fn _command(&self, command:String, args: Vec<String>) -> (bool, Vec<u8>) {
        log::debug!("Running NordVPN command: {}", command);
        let output = Command::new(self.shell.clone())
            .arg("-c")
            .arg(command)
            .args(args)
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            log::debug!("Command executed successfully");
            return (true, output.stdout);
        } else {
            log::error!("Command execution failed");
            return (false, output.stderr);
        }
    }


    fn _nordvpn_command(&self, args: Vec<String>) -> (bool, Vec<u8>) {
        self._command("nordvpn".to_string(), args)
    }

    pub fn login(&self) -> bool {
        log::debug!("Logging in to NordVPN...");
        let output = self._nordvpn_command(vec!["login".to_string(), "--token".to_string(), self.token.clone()]);
        if output.0 {
            log::info!("Logged in successfully: {}", String::from_utf8_lossy(&output.1));
        } else {
            log::error!("Failed to log in: {}", String::from_utf8_lossy(&output.1));
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
        return true;
    }
    pub fn status(&self) -> bool {
        log::debug!("Checking NordVPN status...");
        let output = self._nordvpn_command(vec!["status".to_string()]);
        return true;
    }
}

impl Drop for NordVPN {
    fn drop(&mut self) {
        self.disconnect();
        log::info!("Dropping NordVPN instance");
    }
}

