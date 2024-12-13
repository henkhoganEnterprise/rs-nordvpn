////////////////////////////////////////////////////////////////////////////////////////////////////
// based on https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs  //////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
#![deny(warnings)]

use std::collections::HashMap;
use std::time::{SystemTime, Duration};


use serde_derive::{Deserialize, Serialize};
use utoipa::schema;

use crate::nordvpn::NordVPN;



#[path = "../benches/support/mod.rs"]
mod support;

#[path = "./functions.rs"]
pub mod proxy_functions;



type RunReturnType = Result<(), Box<dyn std::error::Error>>;

/*

*/
#[derive(Serialize, Deserialize)]
//#[derive(utoipa::ToSchema)]
pub struct ProxyStatus {
    pub drained: bool,
    inbound_connections: HashMap<String, SystemTime>,
    inflight_connection_count: u16,
    inflight_connect_requests: u16,
    monitored_hosts: HashMap<String, (Option<SystemTime>, Vec<SystemTime>)>
}

#[derive(Serialize, Deserialize)]
//#[derive(utoipa::ToSchema)]
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
#[derive(Debug, Clone)]
#[derive(utoipa::ToSchema)]
#[schema(as = SystemTime, value_type = String)]
pub struct SchemaCompatibleSystemTime(SystemTime);
impl SchemaCompatibleSystemTime {
    pub fn now() -> Self {
        SchemaCompatibleSystemTime(SystemTime::now())
    }
}


#[derive(Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]
pub struct ProxyRotateResult {
    success: bool,
    last_rotation: SchemaCompatibleSystemTime
}

#[derive(Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]
pub struct ProxySettingsDrainUpdate {
    before: bool,
    after: bool,
}
impl ProxySettingsDrainUpdate {
    pub fn new(before: bool, after: bool) -> Self {
        ProxySettingsDrainUpdate {
            before,
            after
        }
    }
    
}

trait ProxyRotation {
    fn call(&mut self, rotation_callback: impl FnOnce() -> Result<ProxyRotateResult, ProxyRotateResult>) -> Result<bool, ProxyRotateResult>;
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct TimedProxyRotation {
    pub interval: Duration,
    pub last_rotation: SystemTime,
    pub next_rotation: SystemTime
}
impl TimedProxyRotation {
    pub fn new(interval: u64) -> Self {
        let now = SystemTime::now();
        let interval = std::time::Duration::from_secs(interval);
        TimedProxyRotation {
            interval: interval,
            last_rotation: now,
            next_rotation: now + interval
        }
    }
}

impl ProxyRotation for TimedProxyRotation {
    fn call(&mut self, rotation_callback: impl FnOnce() -> Result<ProxyRotateResult, ProxyRotateResult>) -> Result<bool, ProxyRotateResult> {
        if SystemTime::now() > self.next_rotation {
            match rotation_callback() {
                Ok(_) => {
                    self.last_rotation = SystemTime::now();
                    self.next_rotation = self.last_rotation + self.interval;
                    Ok(true)
                },
                Err(e) => Err(e)
            }
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct RequestCountProxyRotation {
    request_count: u16,
    rotation_count: u16,
    last_rotation: SystemTime
}

impl RequestCountProxyRotation {
    pub fn new(rotation_count: u16) -> Self {
        RequestCountProxyRotation {
            request_count: 0,
            rotation_count: rotation_count,
            last_rotation: SystemTime::now()
        }
    }
}

impl ProxyRotation for RequestCountProxyRotation {
    fn call(&mut self, rotation_callback: impl FnOnce() -> Result<ProxyRotateResult, ProxyRotateResult>) -> Result<bool, ProxyRotateResult> {
        self.request_count += 1;
        if self.request_count >= self.rotation_count {
            match rotation_callback() {
                Ok(_) => {
                    self.last_rotation = SystemTime::now();
                    self.request_count = 0;
                    Ok(true)
                },
                Err(e) => Err(e)
            }
        } else {
            Ok(false)
        }
    }
}


#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub enum ProxyRotationMode {
    Timed(TimedProxyRotation),
    RequestCount(RequestCountProxyRotation),
    Manual
}

impl ProxyRotation for ProxyRotationMode {
    fn call(&mut self, rotation_callback: impl FnOnce() -> Result<ProxyRotateResult, ProxyRotateResult>) -> Result<bool, ProxyRotateResult> {
        match self {
            ProxyRotationMode::Timed(ref mut rotation) => rotation.call(rotation_callback),
            ProxyRotationMode::RequestCount(ref mut rotation) => rotation.call(rotation_callback),
            ProxyRotationMode::Manual => Ok(false)  
        }
    }
}


#[derive(Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]
pub struct ProxySettingsRotationIntervalUpdate {
    before: u16,
    after: u16,
}


#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ProxySetting {
    pub rotation: ProxyRotationMode,
    pub monitored_hosts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProxyState {
    pub nordvpn: NordVPN,
    pub drained: bool,
    inbound_connections: HashMap<String, SystemTime>,
    inflight_connections: u16,
    inflight_connect_requests: u16,
    monitored_hosts: HashMap<String, (Option<SystemTime>, Vec<SystemTime>)>,
    last_rotation: SchemaCompatibleSystemTime,
    pub settings: ProxySetting,
}

impl ProxyState {
    pub fn new(nordvpn: NordVPN, monitored_hosts: Vec<String>, rotation: ProxyRotationMode) -> Self {
        ProxyState {
            nordvpn,
            drained: false,
            inbound_connections: HashMap::new(),
            inflight_connections: 0,
            inflight_connect_requests: 0,
            monitored_hosts: monitored_hosts.iter().map(|host| (host.clone(), (None, vec![]))).collect(),
            last_rotation: SchemaCompatibleSystemTime::now(),
            settings: ProxySetting {
                rotation: rotation,
                monitored_hosts: monitored_hosts,
            }
        }
    }

    pub fn compact_status(&self) -> ProxyStatusCompact {
        return ProxyStatusCompact {
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
        let mut rotation = self.settings.rotation.clone();
        let _ = rotation.call(|| self.rotate());
        self.settings.rotation = rotation;
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

    pub fn set_rotation_interval(&mut self, interval: u16) -> ProxySettingsRotationIntervalUpdate {

        match self.settings.rotation {
            ProxyRotationMode::Timed(ref mut value) => {
                let before = value.interval.as_secs() as u16;
                value.interval = std::time::Duration::from_secs(interval as u64);
                ProxySettingsRotationIntervalUpdate {
                    before,
                    after: interval
                }
            },
            _ => {
                self.settings.rotation = ProxyRotationMode::Timed(TimedProxyRotation::new(interval as u64));
                ProxySettingsRotationIntervalUpdate {
                    before: 0,
                    after: interval
                }
            }
        }
    }


    pub fn rotate(&mut self) -> Result<ProxyRotateResult, ProxyRotateResult> {
        let _result = self.nordvpn.rotate();
        match _result {
            Ok(_) => {
                self.last_rotation = SchemaCompatibleSystemTime::now();
                log::info!("Rotated proxy");
                return Ok(ProxyRotateResult {
                    success: true,
                    last_rotation: self.last_rotation.clone()
                })
            },
            Err(_) => {
                return Err(ProxyRotateResult {
                    success: false,
                    last_rotation: self.last_rotation.clone()
                });
            }
        }
    }

}
