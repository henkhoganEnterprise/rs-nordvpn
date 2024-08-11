use std::process::Command;

#[derive(Debug, Clone)]
pub struct CurlClient {
    bin_path: String,
}

impl CurlClient {

    pub fn new_with_path_discovery() -> Self {
        let curl_path_out = Command::new("which")
        .arg("curl")
        .output()
        .expect("Failed to execute command");
        let curl_path: String;
        if curl_path_out.status.success() {
            curl_path = std::str::from_utf8(&curl_path_out.stdout).unwrap().trim().to_string();
            log::info!("Curl found in path: {}", curl_path);
        } else {
            log::error!("Curl not found in path");
            std::process::exit(1);
        }

        CurlClient::new(curl_path)
    }

    pub fn new(bin_path: String) -> Self {
        CurlClient {
            bin_path
        }
    }

    pub fn get(&self, url: &str) -> Result<String, String> {
        let output = Command::new(&self.bin_path)
            .arg(url)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout).map_err(|e| format!("Failed to parse output: {}", e))?)
        } else {
            Err(String::from_utf8(output.stderr).map_err(|e| format!("Failed to parse error: {}", e))?)
        }
    }
}
