////////////////////////////////////////////////////////////////////////////////////////////////////
// based on https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs  //////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
#![deny(warnings)]

use std::collections::HashMap;
use std::time::SystemTime;


use serde_derive::{Deserialize, Serialize};

use crate::nordvpn::NordVPN;



#[path = "../benches/support/mod.rs"]
mod support;

#[path = "./functions.rs"]
pub mod proxy_functions;



type RunReturnType = Result<(), Box<dyn std::error::Error>>;

/*

*/
#[derive(Serialize, Deserialize)]
pub struct ProxyStatus {
    drained: bool,
    inbound_connections: HashMap<String, SystemTime>,
    inflight_connection_count: u16,
    inflight_connect_requests: u16,
    monitored_hosts: HashMap<String, (Option<SystemTime>, Vec<SystemTime>)>
}

#[derive(Serialize, Deserialize)]
pub struct ProxyStatusCompact {
    drained: bool,
    inbound_connections: HashMap<String, SystemTime>,
    inflight_connections: u16,
    inflight_connect_requests: u16,
    monitored_hosts: HashMap<String, i32>
}

#[derive(Serialize, Deserialize)]
pub struct ProxyStatusSanitizerResult {
}

#[derive(Serialize, Deserialize)]
pub struct ProxyRotateResult {
    last_rotation: SystemTime
}


#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ProxySetting {
    rotation_interval: u16,
    monitored_hosts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProxyState {
    pub nordvpn: NordVPN,
    drained: bool,
    inbound_connections: HashMap<String, SystemTime>,
    inflight_connections: u16,
    inflight_connect_requests: u16,
    monitored_hosts: HashMap<String, (Option<SystemTime>, Vec<SystemTime>)>,
    rotation_interval: u16,
    last_rotation: SystemTime,
    //sanitizer: Option<JoinHandle<ProxyStatusSanitizerResult>>,
    pub settings: ProxySetting,
}

impl ProxyState {
    pub fn new(nordvpn: NordVPN, monitored_hosts: Vec<String>, rotation_interval: u16) -> Self {
        ProxyState {
            nordvpn,
            drained: false,
            inbound_connections: HashMap::new(),
            inflight_connections: 0,
            inflight_connect_requests: 0,
            monitored_hosts: monitored_hosts.iter().map(|host| (host.clone(), (None, vec![]))).collect(),
            rotation_interval: rotation_interval,
            last_rotation: SystemTime::now(),
            settings: ProxySetting {
                rotation_interval: rotation_interval,
                monitored_hosts: monitored_hosts,
            }
        }
    }

    pub fn compact_status(&self) -> ProxyStatusCompact {
        ProxyStatusCompact {
            drained: self.drained,
            inbound_connections: self.inbound_connections.clone(),
            inflight_connections: self.inflight_connections,
            inflight_connect_requests: self.inflight_connect_requests,
            monitored_hosts: self.monitored_hosts.iter().map(|(host, (_last, times))| (host.clone(), times.len() as i32)).collect()
        }
    }

    pub fn purge(&mut self, retention: Option<u64>) {
        self.monitored_hosts.iter_mut().for_each(|(_host, (_last, times))| {
            times.retain(|time| time.elapsed().unwrap().as_secs() < retention.unwrap_or(60));
        });
    }

    //pub async fn run(self, bind_addr: SocketAddr) -> RunReturnType {
    //    // Do not use this to run multiple listeners for the same instance, because these instance will not be synchronized
    //    proxy_functions::run(Arc::new(RwLock::new(self)), bind_addr).await
    //} 

    pub fn sanitize(&mut self, retention: Option<u64>) {
        self.purge(retention);
    }

    pub fn status(&self) -> ProxyStatus {
        ProxyStatus {
            drained: self.drained,
            inbound_connections: self.inbound_connections.clone(),
            inflight_connection_count: self.inflight_connections,
            inflight_connect_requests: self.inflight_connect_requests,
            monitored_hosts: self.monitored_hosts.clone()
        }
    }

    pub fn add_connection(&mut self, peer_addr: String) {
        self.inbound_connections.insert(peer_addr, SystemTime::now());
        self.inflight_connections += 1;
    }

    pub fn remove_connection(&mut self, peer_addr: String) {
        self.inbound_connections.remove(&peer_addr);
        self.inflight_connections -= 1;
        self.rotate_if_needed();
    }

    pub fn add_connect_request(&mut self, host: &str) {
        self.monitored_hosts.get_mut(host).map(|(last, times)| {
            times.push(SystemTime::now());
            *last = Some(SystemTime::now());
        });
        self.inflight_connect_requests += 1;
    }

    pub fn remove_connect_request(&mut self) {
        self.inflight_connect_requests -= 1;
    }

    pub fn drain(&mut self) {
        self.drained = true;
    }

    pub fn activate(&mut self) {
        self.drained = false;
    }

    pub fn set_rotation_interval(&mut self, interval: u16) {
        self.rotation_interval = interval;
    }

    fn rotate_if_needed(&mut self) -> bool {
        if self.rotation_interval > 0 && self.last_rotation.elapsed().unwrap().as_secs() > self.rotation_interval as u64 {
            self.rotate();
            return true;
        }
        false
    }

    pub fn rotate(&mut self) -> ProxyRotateResult {
        let _result = self.nordvpn.rotate();
        self.last_rotation = SystemTime::now();
        log::info!("Rotated proxy");
        ProxyRotateResult {
            last_rotation: self.last_rotation
        }
    }

}
