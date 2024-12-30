////////////////////////////////////////////////////////////////////////////////////////////////////
// based on https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs  //////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
#![deny(warnings)]

use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use std::collections::VecDeque;
use chrono::DateTime;


use proxy_functions::RequestAttributes;
use serde_derive::{Deserialize, Serialize};
use utoipa::schema;

use crate::nordvpn::NordVPN;



#[path = "../benches/support/mod.rs"]
mod support;

#[path = "./functions.rs"]
pub mod proxy_functions;


//type Timestamp = SystemTime;
type Timestamp = DateTime<chrono::Utc>;
type RunReturnType = Result<(), Box<dyn std::error::Error>>;
type Retention = Option<u64>;

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct HostMonitorCompact {
    pub active_connections: u16,
    pub lifetime_connections: u16,
    pub last: Option<SchemaCompatibleSystemTime>,
}


#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
#[derive(utoipa::ToSchema)]
pub struct HostMonitor {
    pub active_connections: u16,
    pub lifetime_connections: u16,

    pub capacity: usize,

    pub last: Option<SchemaCompatibleSystemTime>,
    #[schema(value_type = Vec<SchemaCompatibleSystemTime>)]
    pub times: VecDeque<SchemaCompatibleSystemTime>
}


impl HostMonitor {

    pub fn new () -> Self {
        HostMonitor::new_with_capacity(0)
    }

    pub fn new_with_capacity(capacity: usize) -> Self {
        HostMonitor {
            active_connections: 0,
            lifetime_connections: 0,
            capacity: capacity,
            last: None,
            times: VecDeque::new() // Set the desired capacity
        }
    }

    pub fn check_in(&mut self) -> () {
        self.last = Some(SchemaCompatibleSystemTime::now());

        if self.capacity > 0 {
            if self.times.len() == self.capacity {
                self.times.pop_front();
            }
            self.times.push_back(self.last.clone().unwrap());
        }

        self.active_connections += 1;
        self.lifetime_connections += 1;
    }

    pub fn check_out(&mut self) -> () {
        self.active_connections -= 1;
    }

    pub fn compact(&self) -> HostMonitorCompact {
        HostMonitorCompact {
            active_connections: self.active_connections,
            lifetime_connections: self.lifetime_connections,
            last: self.last.clone()
        }
    }

    pub fn purge(&mut self, retention: Option<u64>) -> () {
        self.times = self.times.iter().filter(|time| (chrono::Utc::now() - time.0).num_seconds() < retention.unwrap_or(60) as i64).map(|time| time.clone()).collect();
    }


    /*
    pub fn reset(&mut self) -> () {
        self.active_connections = 0;
        self.lifetime_connections = 0;
        self.last = None;
        self.times.clear();
    }
    */
}


#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProxyMonitorCompact {
    pub active_connections: u16,
    pub lifetime_connections: u16,
    pub hosts: HashMap<String, HostMonitorCompact>
}


#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
#[derive(utoipa::ToSchema)]
pub struct ProxyMonitor {
    pub active_connections: u16,
    pub lifetime_connections: u16,
    pub hosts: HashMap<String, HostMonitor>
}

impl ProxyMonitor {
    pub fn new(monitored_hosts: Vec<String>) -> Self {
        ProxyMonitor {
            active_connections: 0,
            lifetime_connections: 0,
            hosts: monitored_hosts.iter().map(|host| (host.clone(), HostMonitor::new())).collect(),
        }
    }

    pub fn check_in(&mut self, request_attributes: &RequestAttributes) -> &mut Self {
        self.active_connections += 1;
        self.lifetime_connections += 1;
        let host = request_attributes.uri.host().expect("uri has no host");
        if let Some(host_monitor) = self.hosts.get_mut(host) {
            host_monitor.check_in();
        }
        self
    }

    pub fn check_out(&mut self, request_attributes: &RequestAttributes) -> &mut Self {
        self.active_connections -= 1;
        if let Some(host_monitor) = self.hosts.get_mut(request_attributes.uri.host().expect("uri has no host")) {
            host_monitor.check_out();
        }
        self
    }

    pub fn compact(&self) -> ProxyMonitorCompact {
        ProxyMonitorCompact {
            active_connections: self.active_connections,
            lifetime_connections: self.lifetime_connections,
            hosts: self.hosts.iter().map(|(host, monitor)| (host.clone(), monitor.compact())).collect()
        }
    }

    pub fn purge(&mut self, retention: Retention) {
            self.hosts.iter_mut().for_each(|(_, host_monitor)| {
                host_monitor.purge(retention);
            });
        }
}
/*


*/
#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
#[derive(utoipa::ToSchema)]
pub struct ProxyStatus {
    pub drained: bool,
    inbound_connections: HashMap<String, SchemaCompatibleSystemTime>,
    inflight_connection_count: u16,
    inflight_connect_requests: u16,
    //monitored_hosts: HashMap<String, (Option<SystemTime>, Vec<SystemTime>)>
    monitor: ProxyMonitor
}

#[derive(Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]
pub struct ProxyStatusCompact {
    drained: bool,
    inbound_connections: HashMap<String, SchemaCompatibleSystemTime>,
    inflight_connections: u16,
    inflight_connect_requests: u16,
    //monitored_hosts: HashMap<String, i32>
    monitor: ProxyMonitorCompact
}

#[derive(Serialize, Deserialize)]
pub struct ProxyStatusSanitizerResult {
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
#[derive(utoipa::ToSchema)]
#[schema(as = Timestamp, value_type = String)]
pub struct SchemaCompatibleSystemTime(Timestamp);
impl SchemaCompatibleSystemTime {
    pub fn now() -> Self {
        SchemaCompatibleSystemTime(chrono::Utc::now())
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
    //pub monitored_hosts: Vec<String>,
    pub rotation_retries: u8
}


#[derive(Debug, Clone)]
pub struct ProxyState {
    pub nordvpn: NordVPN,
    pub drained: bool,
    inbound_connections: HashMap<String, SchemaCompatibleSystemTime>,
    inflight_connections: u16,
    inflight_connect_requests: u16,
    //monitored_hosts: HashMap<String, (Option<SystemTime>, Vec<SystemTime>)>,
    monitor: ProxyMonitor,
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
            //monitored_hosts: monitored_hosts.iter().map(|host| (host.clone(), (None, vec![]))).collect(),
            monitor: ProxyMonitor::new(monitored_hosts),
            last_rotation: SchemaCompatibleSystemTime::now(),
            settings: ProxySetting {
                rotation: rotation,
                //monitored_hosts: monitored_hosts,
                rotation_retries: 3
            }
        }
    }

    pub fn compact_status(&self) -> ProxyStatusCompact {
        return ProxyStatusCompact {
            drained: self.drained,
            inbound_connections: self.inbound_connections.clone(),
            inflight_connections: self.inflight_connections,
            inflight_connect_requests: self.inflight_connect_requests,
            //monitored_hosts: self.monitored_hosts.iter().map(|(host, (_last, times))| (host.clone(), times.len() as i32)).collect()
            monitor: self.monitor.compact()
        }
    }

    pub fn purge(&mut self, retention: Option<u64>) {
        self.monitor.purge(retention)
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
            monitor: self.monitor.clone(),
        }
    }

    pub fn add_connection(&mut self, peer_addr: String) {
        self.inbound_connections.insert(peer_addr, SchemaCompatibleSystemTime::now());
        self.inflight_connections += 1;
    }

    pub fn remove_connection(&mut self, peer_addr: String) {
        self.inbound_connections.remove(&peer_addr);
        self.inflight_connections -= 1;
        let mut rotation = self.settings.rotation.clone();
        let _ = rotation.call(|| self.rotate_default());
        self.settings.rotation = rotation;
    }

    pub fn add_connect_request(&mut self, request_attributes: &RequestAttributes) {
        //let host = req.uri().host().expect("uri has no host");

        self.monitor.check_in(request_attributes);
        //self.monitored_hosts.get_mut(host).map(|(last, times)| {
        //    times.push(SystemTime::now());
        //    *last = Some(SystemTime::now());
        //});
        self.inflight_connect_requests += 1;
    }

    pub fn remove_connect_request(&mut self, request_attributes: &RequestAttributes) {
        self.monitor.check_out(request_attributes);
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


    pub fn rotate(&mut self, retries: u8) -> Result<ProxyRotateResult, ProxyRotateResult> {
        self.drain();
        let _result = self.nordvpn.rotate(retries);
        match _result {
            Ok(_) => {
                self.last_rotation = SchemaCompatibleSystemTime::now();
                log::info!("Rotated proxy");
                self.activate();
                return Ok(ProxyRotateResult {
                    success: true,
                    last_rotation: self.last_rotation.clone()
                })
            },
            Err(_) => {
                log::error!("Failed to rotate proxy");
                return Err(ProxyRotateResult {
                    success: false,
                    last_rotation: self.last_rotation.clone()
                });
            }
        }
    }

    pub fn rotate_default(&mut self) -> Result<ProxyRotateResult, ProxyRotateResult> {
        self.rotate(self.settings.rotation_retries)
    }


}
