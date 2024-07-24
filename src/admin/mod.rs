#![deny(warnings)]

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use bytes::Bytes;
//use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::{body::Incoming as IncomingBody, Method, Request, Response};
use tokio::net::TcpListener;
use hyper::service::Service;


use route_recognizer::Router;



#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;
use tokio_util::bytes;

pub async fn run(bind_addr: SocketAddr, admin: Admin) -> Result<(), Box<dyn std::error::Error>> {
    //let bind_addr = SocketAddr::from_str((ip, port));

    let listener = TcpListener::bind(bind_addr).await?;
    log::info!("Listening on http://{}", bind_addr);
  
    loop {
        let (stream, _) = listener.accept().await?;
        log::info!("Accepted a new TCP connection from: {}", stream.peer_addr()?);
        let io = TokioIo::new(stream);
        let admin_clone = admin.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, admin_clone).await {
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



use crate::nordvpn;

#[derive(Debug, Clone)]
pub enum AdminRoutes {
    AdminRotation,
    NordvpnAccount,
    NordvpnConnect,
    NordvpnConnectWithArgument,
    NordvpnDisconnect,
    NordvpnStatus,
    NordvpnDaemonStatus,
    NordvpnDaemonRestart,
    NordvpnDaemonStart,
    NordvpnDaemonStop
}

#[derive(Debug, Clone)]
pub struct Admin {
    nordvpn: nordvpn::NordVPN,
    router: Router<AdminRoutes>
}


impl Admin {
    pub fn new(nordvpn: nordvpn::NordVPN) -> Result<Self, &'static str> {

        log::info!("Creating new Admin instance");
        let mut router: Router<AdminRoutes> = Router::new();

        router.add("/admin/rotation/:TYPE/:VALUE", AdminRoutes::AdminRotation);
        router.add("/nordvpn/account", AdminRoutes::NordvpnAccount);

        router.add("/nordvpn/connect", AdminRoutes::NordvpnConnect);
        router.add("/nordvpn/connect/:ARGUMENT", AdminRoutes::NordvpnConnectWithArgument);

        router.add("/nordvpn/disconnect", AdminRoutes::NordvpnDisconnect);
        router.add("/nordvpn/status", AdminRoutes::NordvpnStatus);

        router.add("/nordvpn/daemon/status", AdminRoutes::NordvpnDaemonStatus);
        router.add("/nordvpn/daemon/restart", AdminRoutes::NordvpnDaemonRestart);
        router.add("/nordvpn/daemon/start", AdminRoutes::NordvpnDaemonStart);
        router.add("/nordvpn/daemon/stop", AdminRoutes::NordvpnDaemonStop);
        return Ok(Self {
            nordvpn,
            router
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
            
            
            (&Method::GET,  AdminRoutes::NordvpnAccount) => mk_response(format!("{}: {:?}", path, self.nordvpn.account())),
            (&Method::POST, AdminRoutes::NordvpnConnect) => {
                mk_response(serde_json::to_string(&self.nordvpn.connect()).unwrap())
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
            (&Method::GET,  AdminRoutes::NordvpnStatus) => mk_response(serde_json::to_string(&self.nordvpn.status()).unwrap()),

            (&Method::POST, AdminRoutes::NordvpnDaemonRestart) => mk_response(format!("/nordvpn/daemon/restart: {:?}", self.nordvpn.daemon_restart(Some(30)))),
            (&Method::GET,  AdminRoutes::NordvpnDaemonStatus) => mk_response(format!("/nordvpn/daemon/status: {:?}", self.nordvpn.daemon_status().output)),
            (&Method::POST, AdminRoutes::NordvpnDaemonStart) => mk_response(format!("/nordvpn/daemon/start: {:?}", self.nordvpn.daemon_start(Some(30)))),
            (&Method::POST, AdminRoutes::NordvpnDaemonStop) => mk_response(format!("/nordvpn/daemon/stop: {:?}", self.nordvpn.daemon_stop())),
            
            _ => {
                log::warn!("Not found: {:?} {:?}", req.method(), req.uri().path());
                mk_response("oh no! not found".into())
            }
        };

        Box::pin(async { res })
    }
}

