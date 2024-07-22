use std::{io::stdout, process::Command};



pub struct CommandOutput {
    pub status: bool,
    pub output: String,
}


#[derive(Debug, Clone)]
pub struct Daemon {
    path: String,
}

impl Daemon {
    pub fn new() -> Result<Self, &'static str> {
        return Ok(Self {
            path: "/etc/init.d/nordvpn".to_string(), 
        });
    }

    pub fn new_with_path(path: String) -> Result<Self, &'static str> {
        return Ok(Self {
            path,
        });
    }

    fn command(&self, arg: &str) -> std::process::Output {
        Command::new(self.path.clone())
            .arg(arg)
            .output()
            .expect("Failed to execute command")
    }

    fn wrap_command(&self, arg: &str) -> CommandOutput {
        let output = self.command(arg);
        if output.status.success() {
            return CommandOutput {
                status: true,
                output: String::from_utf8_lossy(&output.stdout).to_string(),
            };
        } else {
            return CommandOutput {
                status: false,
                output: String::from_utf8_lossy(&output.stderr).to_string(),
            };
        }
    }

    pub fn status(&self) -> CommandOutput{
        self.wrap_command("status")
    }

    pub fn restart(&self, timeout: u8) -> CommandOutput {
        self.wrap_command("restart")
    }

    pub fn start(&self, timeout: u8) -> CommandOutput {
        self.wrap_command("start")
    }

    pub fn stop(&self) -> CommandOutput {
        self.wrap_command("stop")
    }
}