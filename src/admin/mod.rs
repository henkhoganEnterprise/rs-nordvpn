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
    Account,
    Connect,
    ConnectCountry,
    Disconnect,
    Status,
    DaemonStatus,
    DaemonRestart,
    DaemonStart,
    DaemonStop
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
        router.add("/nordvpn/account", AdminRoutes::Account);

        router.add("/nordvpn/connect", AdminRoutes::Connect);
        router.add("/nordvpn/connect/country/:ARGUMENT", AdminRoutes::ConnectCountry);

        router.add("/nordvpn/disconnect", AdminRoutes::Disconnect);
        router.add("/nordvpn/status", AdminRoutes::Status);

        router.add("/nordvpn/daemon/status", AdminRoutes::DaemonStatus);
        router.add("/nordvpn/daemon/restart", AdminRoutes::DaemonRestart);
        router.add("/nordvpn/daemon/start", AdminRoutes::DaemonStart);
        router.add("/nordvpn/daemon/stop", AdminRoutes::DaemonStop);
        return Ok(Self {
            nordvpn,
            router
        });
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


        let admin_route = match self.router.recognize(req.uri().path()) {
            Ok(binding) => binding,
            Err(_) => return Box::pin(async { mk_response("oh no! not found".into()) }),
        };
        //let admin_route_handler = admin_route.handler();
        

        let res = match (req.method(), admin_route.handler()) {
            
            (&Method::GET,  AdminRoutes::Account) => mk_response(format!("/nordvpn/account: {:?}", self.nordvpn.account())),
            (&Method::POST, AdminRoutes::Connect) => {
                mk_response(format!("/nordvpn/connect: {:?}", self.nordvpn.connect()))
            },
            (&Method::POST, AdminRoutes::ConnectCountry) => {
                //log::info!("path: {:?}", req.uri().path().to_string());
                let argument = admin_route.params().find("ARGUMENT").unwrap();
                log::info!("argument: {:?}", argument);
                mk_response(format!("/nordvpn/connect: {:?}", self.nordvpn.connect_with_argument(argument)))
            },
            (&Method::POST, AdminRoutes::Disconnect) => mk_response(format!("/nordvpn/disconnect: {:?}", self.nordvpn.disconnect())),
            (&Method::GET,  AdminRoutes::Status) => mk_response(format!("/nordvpn/status {:?}", self.nordvpn.status())),

            (&Method::POST, AdminRoutes::DaemonRestart) => mk_response(format!("/nordvpn/daemon/restart: {:?}", self.nordvpn.daemon_restart(Some(30)))),
            (&Method::GET,  AdminRoutes::DaemonStatus) => mk_response(format!("/nordvpn/daemon/status: {:?}", self.nordvpn.daemon_status().output)),
            (&Method::POST, AdminRoutes::DaemonStart) => mk_response(format!("/nordvpn/daemon/start: {:?}", self.nordvpn.daemon_start(Some(30)))),
            (&Method::POST, AdminRoutes::DaemonStop) => mk_response(format!("/nordvpn/daemon/stop: {:?}", self.nordvpn.daemon_stop())),
            
            _ => {
                log::warn!("Not found: {:?}", req.uri().path());
                mk_response("oh no! not found".into())
            }
        };

        Box::pin(async { res })
    }
}

