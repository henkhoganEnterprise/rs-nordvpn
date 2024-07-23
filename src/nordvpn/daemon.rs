use core::time;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};



pub struct CommandOutput {
    pub status: bool,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeamonStatus {
    Unknown,
    Running,
    Stopped,
    Stopping,
    Starting,
    Restarting,
    Error,
}

pub struct StatusOutput {
    pub status: DeamonStatus,
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

    pub fn status(&self) -> StatusOutput{
        let cmd_output = self.wrap_command("status");
        let status: DeamonStatus;
        if cmd_output.output.contains("nordvpn is not running") {
            status = DeamonStatus::Stopped;
        }
        else {
            status = DeamonStatus::Unknown;
        }
        return StatusOutput {
            status,
            output: cmd_output.output,
        };
    }

    pub fn restart(&self, timeout: u8) -> CommandOutput {
        self.wrap_command("restart")
    }

    pub fn start(&self, timeout: Option<u8>) -> CommandOutput {
        let cmd_output = self.wrap_command("start");
        if !timeout.is_none() {
            let status = self.status();
            if status.status != DeamonStatus::Running {
                let current_time = || {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Failed to get current time")
                        .as_secs()
                };
    
                let timeout_expiration = current_time() + u64::from(timeout.unwrap());
                while current_time() < timeout_expiration {
                    let status = self.status();
                    if status.status == DeamonStatus::Running {
                        break;
                    } 
                }

            }
        }
        return CommandOutput {
            status: cmd_output.status,
            output: cmd_output.output,
        };
    }

    pub fn stop(&self) -> CommandOutput {
        self.wrap_command("stop")
    }
}