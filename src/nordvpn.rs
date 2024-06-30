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

    fn _command(&self, command:String, args: Vec<String>) -> bool {
        log::debug!("Running NordVPN command: {}", command);
        let output = Command::new(self.shell.clone())
            .arg("-c")
            .arg(command)
            .args(args)
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            log::info!("Command executed successfully");
            return true;
        } else {
            log::error!("Command execution failed");
            return false;
        }
    }

    fn _nordvpn_command(&self, args: Vec<String>) -> bool {
        self._command("nordvpn".to_string(), args)
    }

    pub fn login(&self) -> bool {
        log::debug!("Logging in to NordVPN...");
        self._nordvpn_command(vec!["login".to_string(), "--token".to_string(), self.token.clone()]);
        true
    }
    pub fn connect(&self) -> bool {
        log::debug!("Connecting to NordVPN...");
        self._nordvpn_command(vec!["connect".to_string()]);
        return true;
    }
    pub fn disconnect(&self) -> bool {
        log::debug!("Disconnecting from NordVPN...");
        self._nordvpn_command(vec!["disconnect".to_string()]);
        return true;
    }
    pub fn status(&self) -> bool {
        println!("Checking NordVPN status...");
        self._nordvpn_command(vec!["status".to_string()]);
        return true;
    }
}

impl Drop for NordVPN {
    fn drop(&mut self) {
        self.disconnect();
        log::info!("Dropping NordVPN instance");
    }
}

