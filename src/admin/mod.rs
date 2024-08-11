#![deny(warnings)]

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
//use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::{body::Incoming as IncomingBody, Method, Request, Response};
use tokio::net::TcpListener;
use hyper::service::Service;


use route_recognizer::Router;

//#[path = "../helper/mod.rs"]
//mod helper;
use helper::CurlClient;


#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;
use tokio_util::bytes;

const IP_URL: &str = "https://api.ipify.org";

pub async fn run(bind_addr: SocketAddr, admin: Arc<Admin>) -> Result<(), Box<dyn std::error::Error>> {
    //let bind_addr = SocketAddr::from_str((ip, port));

    let listener = TcpListener::bind(bind_addr).await?;
    log::info!("Admin istening on http://{}", bind_addr);

  
    loop {
        let (stream, _) = listener.accept().await?;
        log::info!("Admin accepted a new TCP connection from: {}", stream.peer_addr()?);
        let io = TokioIo::new(stream);
        let x = admin.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, x).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

/*fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
*/



use crate::{helper, nordvpn, proxy};

#[derive(Debug, Clone)]
pub enum AdminRoutes {
    AdminRotation,
    IpLocal,
    IpPublic,
    NordvpnAccount,
    NordvpnConnect,
    NordvpnConnectWithArgument,
    NordvpnDisconnect,
    NordvpnLogs,
    NordvpnLogsWithArgument,
    NordvpnStatus,
    NordvpnDaemonStatus,
    NordvpnDaemonRestart,
    NordvpnDaemonStart,
    NordvpnDaemonStop,
    ProxyStatus,
    ProxyStatusCompact,
    ProxyStatusPurge,
}

#[derive(Debug, Clone)]
pub struct Admin {
    curl_client: CurlClient,
    nordvpn: nordvpn::NordVPN,
    proxy: Arc<Mutex<proxy::ProxyState>>,
    router: Router<AdminRoutes>
}

pub fn router() -> Router<AdminRoutes> {
    let mut router: Router<AdminRoutes> = Router::new();

    router.add("/admin/rotation/:TYPE/:VALUE", AdminRoutes::AdminRotation);

    router.add("/ip/local", AdminRoutes::IpLocal);
    router.add("/ip/public", AdminRoutes::IpPublic);

    router.add("/nordvpn/account", AdminRoutes::NordvpnAccount);

    router.add("/nordvpn/connect", AdminRoutes::NordvpnConnect);
    router.add("/nordvpn/connect/:ARGUMENT", AdminRoutes::NordvpnConnectWithArgument);

    router.add("/nordvpn/disconnect", AdminRoutes::NordvpnDisconnect);
    router.add("/nordvpn/logs", AdminRoutes::NordvpnLogs);
    router.add("/nordvpn/logs/:LINES", AdminRoutes::NordvpnLogsWithArgument);
    router.add("/nordvpn/status", AdminRoutes::NordvpnStatus);

    router.add("/nordvpn/daemon/status", AdminRoutes::NordvpnDaemonStatus);
    router.add("/nordvpn/daemon/restart", AdminRoutes::NordvpnDaemonRestart);
    router.add("/nordvpn/daemon/start", AdminRoutes::NordvpnDaemonStart);
    router.add("/nordvpn/daemon/stop", AdminRoutes::NordvpnDaemonStop);

    router.add("/proxy/status", AdminRoutes::ProxyStatus);
    router.add("/proxy/status/compact", AdminRoutes::ProxyStatusCompact);
    router.add("/proxy/status/purge", AdminRoutes::ProxyStatusPurge);

    return router;
}


impl Admin {
    pub fn new(curl_client: CurlClient, nordvpn: nordvpn::NordVPN, proxy: Arc<Mutex<proxy::ProxyState>>) -> Result<Self, &'static str> {

        log::info!("Creating new Admin instance");

        return Ok(Self {
            curl_client,
            nordvpn,
            proxy,
            router: router()
        });
    }

    pub fn get_rotation(&self) -> String {
        log::debug!("Getting rotation...");
        return "rotation".to_string();
    }

    pub fn set_rotation_from_str(&self, rotation_type: &str, rotation_value: &str) -> String {
        log::debug!("Setting rotation to type: {:?}, value: {:?}", rotation_type, rotation_value);
        return "rotation".to_string();
    }

    //pub fn get_status(&self) -> bool {
    //    log::debug!("Getting status...");
    //    let output = self.nordvpn.account();
    //    return output;
    //}

    /* pub fn nord_account(&self) -> () {
        log::debug!("Checking NordVPN account...");
        let output = self.nordvpn.account();
        if output {
            log::info!("Account: {}", output);
        } else {
            log::error!("Failed to fetch account: {}", output);
        }
        return ();
    } */
}

impl Service<Request<IncomingBody>> for Admin {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }

        let path = req.uri().path();
        let admin_route = match self.router.recognize(path) {
            Ok(binding) => binding,
            Err(_) => return Box::pin(async { mk_response("route not matched".into()) }),
        };
        //let admin_route_handler = admin_route.handler();
        

        let res = match (req.method(), admin_route.handler()) {

            (&Method::GET,  AdminRoutes::AdminRotation) => mk_response(self.get_rotation()),
            (&Method::POST,  AdminRoutes::AdminRotation) => {
                let _type = admin_route.params().find("TYPE").unwrap();
                let _value = admin_route.params().find("VALUE").unwrap();
                log::info!("_type: {:?},  value: {:?}", _type, _value);
                mk_response(self.set_rotation_from_str(_type, _value))
            },
            

            (&Method::GET,  AdminRoutes::IpPublic) => {
                let ip = self.curl_client.get(IP_URL).unwrap();
                mk_response(ip)
            }

            
            (&Method::GET,  AdminRoutes::NordvpnAccount) => mk_response(format!("{}: {:?}", path, self.nordvpn.account())),
            (&Method::POST, AdminRoutes::NordvpnConnect) => {
                let mut proxy_lock = self.proxy.lock().unwrap();
                proxy_lock.drain();
                let resp = mk_response(serde_json::to_string(&self.nordvpn.connect()).unwrap());
                proxy_lock.activate();
                drop(proxy_lock);
                resp
            },
            (&Method::POST, AdminRoutes::NordvpnConnectWithArgument) => {
                let argument = admin_route.params().find("ARGUMENT").unwrap();
                log::info!("argument: {:?}", argument);
                let output = match self.nordvpn.connect_with_argument(argument) {
                    Ok(output) => {
                        log::info!("Connected with argument: {:?}", argument);
                        mk_response(serde_json::to_string(&output).unwrap())

                    },
                    Err(e) => {
                        log::error!("Failed to connect with argument: {:?}", argument);
                        mk_response(format!("Failed to connect with argument: {:?}", e))
                    }
                };
                output
            },

            (&Method::POST, AdminRoutes::NordvpnDisconnect) => mk_response(format!("/nordvpn/disconnect: {:?}", self.nordvpn.disconnect())),
            (&Method::GET,  AdminRoutes::NordvpnLogs) => mk_response(serde_json::to_string(&self.nordvpn.logs(10)).unwrap()),
            (&Method::GET,  AdminRoutes::NordvpnLogsWithArgument) => {
                let argument = admin_route.params().find("LINES").unwrap();
                log::info!("argument: {:?}", argument);
                mk_response(serde_json::to_string(&self.nordvpn.logs(argument.parse().unwrap())).unwrap())
            },
            (&Method::GET,  AdminRoutes::NordvpnStatus) => mk_response(serde_json::to_string(&self.nordvpn.status()).unwrap()),

            (&Method::POST, AdminRoutes::NordvpnDaemonRestart) => mk_response(format!("/nordvpn/daemon/restart: {:?}", self.nordvpn.daemon_restart(Some(30)))),
            (&Method::GET,  AdminRoutes::NordvpnDaemonStatus) => mk_response(format!("/nordvpn/daemon/status: {:?}", self.nordvpn.daemon_status().output)),
            (&Method::POST, AdminRoutes::NordvpnDaemonStart) => mk_response(format!("/nordvpn/daemon/start: {:?}", self.nordvpn.daemon_start(Some(30)))),
            (&Method::POST, AdminRoutes::NordvpnDaemonStop) => mk_response(format!("/nordvpn/daemon/stop: {:?}", self.nordvpn.daemon_stop())),
            
            (&Method::GET,  AdminRoutes::ProxyStatus) => mk_response(format!("/proxy/status: {:?}", serde_json::to_string(&self.proxy.lock().unwrap().status()).unwrap())),
            (&Method::GET,  AdminRoutes::ProxyStatusCompact) => mk_response(format!("/proxy/status: {:?}", serde_json::to_string(&self.proxy.lock().unwrap().compact_status()).unwrap())),

            (&Method::POST,  AdminRoutes::ProxyStatusPurge) => mk_response(format!("/proxy/status: {:?}", serde_json::to_string(&self.proxy.lock().unwrap().purge()).unwrap())),

            _ => {
                log::warn!("Not found: {:?} {:?}", req.method(), req.uri().path());
                mk_response("oh no! not found".into())
            }
            
        };

        Box::pin(async { res })
    }
}

