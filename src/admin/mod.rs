#![deny(warnings)]

use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use http_body_util::BodyExt;
//use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use http_body_util::{combinators::BoxBody, Full};
use hyper::server::conn::http1;
use hyper::{Method, Request, Response};
use tokio::net::TcpListener;
use hyper::service::service_fn;


use route_recognizer::Router;

//#[path = "../helper/mod.rs"]
//mod helper;
use helper::CurlClient;


#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;
use tokio_util::bytes;

const IP_URL: &str = "https://api.ipify.org";

pub async fn run<'a>(bind_addr: SocketAddr, admin: Admin) -> Result<(), Box<dyn std::error::Error>> {
    //let bind_addr = SocketAddr::from_str((ip, port));

    let listener = TcpListener::bind(bind_addr).await?;
    log::info!("Admin istening on http://{}", bind_addr);

    
    let admin = Arc::new(admin);
  
    loop {
        let (stream, _) = listener.accept().await?;
        log::debug!("Admin accepted a new TCP connection from: {}", stream.peer_addr()?);
        let io = TokioIo::new(stream);
        //let x = admin.clone();
        let admin = admin.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io, 
                    service_fn(move |req| call(admin.clone(), req))
                )
                .await 
            {
                log::error!("Failed to serve connection: {:?}", err);
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



use crate::{helper, proxy};

#[derive(Debug, Clone)]
pub enum AdminRoutes {
    AdminRotate,
    IpLocal,
    IpPublic,
    NordvpnAccount,
    NordvpnConnect,
    NordvpnConnectWithArgument,
    NordvpnDisconnect,
    NordvpnLogs,
    NordvpnLogsWithArgument,
    NordvpnRotate,
    NordvpnSanititze,
    NordvpnStatus,
    NordvpnDaemonStatus,
    NordvpnDaemonRestart,
    NordvpnDaemonStart,
    NordvpnDaemonStop,
    ProxyRotate,
    ProxySettings,
    ProxySettingsRotation,
    ProxySettingsRotationInterval,
    ProxyStatus,
    ProxyStatusCompact,
    ProxyStatusPurge,

}

#[derive(Debug, Clone)]
pub struct Admin {
    curl_client: CurlClient,
    proxy:Arc<RwLock<proxy::ProxyState>>,
    router: Router<AdminRoutes>
}

pub fn router() -> Router<AdminRoutes> {
    let mut router: Router<AdminRoutes> = Router::new();

    router.add("/admin/rotation/:TYPE/:VALUE", AdminRoutes::AdminRotate);

    router.add("/ip/local", AdminRoutes::IpLocal);
    router.add("/ip/public", AdminRoutes::IpPublic);

    router.add("/nordvpn/account", AdminRoutes::NordvpnAccount);

    router.add("/nordvpn/connect", AdminRoutes::NordvpnConnect);
    router.add("/nordvpn/connect/:ARGUMENT", AdminRoutes::NordvpnConnectWithArgument);

    router.add("/nordvpn/disconnect", AdminRoutes::NordvpnDisconnect);

    router.add("/nordvpn/logs", AdminRoutes::NordvpnLogs);
    router.add("/nordvpn/logs/:LINES", AdminRoutes::NordvpnLogsWithArgument);

    router.add("/nordvpn/rotate", AdminRoutes::NordvpnRotate);

    router.add("/nordvpn/sanitize", AdminRoutes::NordvpnSanititze);
    
    router.add("/nordvpn/status", AdminRoutes::NordvpnStatus);

    router.add("/nordvpn/daemon/status", AdminRoutes::NordvpnDaemonStatus);
    router.add("/nordvpn/daemon/restart", AdminRoutes::NordvpnDaemonRestart);
    router.add("/nordvpn/daemon/start", AdminRoutes::NordvpnDaemonStart);
    router.add("/nordvpn/daemon/stop", AdminRoutes::NordvpnDaemonStop);


    router.add("/proxy/rotate", AdminRoutes::ProxyRotate);

    router.add("/proxy/settings", AdminRoutes::ProxySettings);

    router.add("/proxy/settings/rotation", AdminRoutes::ProxySettingsRotation);
    router.add("/proxy/settings/rotation/interval/:INTERVAL", AdminRoutes::ProxySettingsRotationInterval);

    router.add("/proxy/status", AdminRoutes::ProxyStatus);
    router.add("/proxy/status/compact", AdminRoutes::ProxyStatusCompact);
    router.add("/proxy/status/purge", AdminRoutes::ProxyStatusPurge);

    return router;
}


impl Admin {
    pub fn new(curl_client: CurlClient, proxy: Arc<RwLock<proxy::ProxyState>>) -> Result<Self, &'static str> {

        log::info!("Creating new Admin instance");

        return Ok(Self {
            curl_client,
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


fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn mk_response(s: String) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    Ok(Response::new(full(s)))   
}

async fn call(
    admin: Arc<Admin>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    let path = req.uri().path();
    let admin_route = match admin.router.recognize(path) {
        Ok(binding) => binding,
        Err(_) => return mk_response("route not matched".into()),
    };   

    let res = match (req.method(), admin_route.handler()) {

        (&Method::GET,  AdminRoutes::AdminRotate) => mk_response(admin.get_rotation()),
        (&Method::POST,  AdminRoutes::AdminRotate) => {
            let _type = admin_route.params().find("TYPE").unwrap();
            let _value = admin_route.params().find("VALUE").unwrap();
            log::info!("_type: {:?},  value: {:?}", _type, _value);
            mk_response(admin.set_rotation_from_str(_type, _value))
        },
        

        (&Method::GET,  AdminRoutes::IpPublic) => {
            let ip = admin.curl_client.get(IP_URL).unwrap();
            mk_response(ip)
        },

        
        (&Method::GET,  AdminRoutes::NordvpnAccount) => mk_response(format!("{}: {:?}", path, admin.proxy.read().unwrap().nordvpn.account())),
        (&Method::POST, AdminRoutes::NordvpnConnect) => {
            admin.proxy.write().unwrap().drain();
            let resp = mk_response(serde_json::to_string(&admin.proxy.read().unwrap().nordvpn.connect(None)).unwrap());
            admin.proxy.write().unwrap().activate();
            resp
        },
        (&Method::POST, AdminRoutes::NordvpnConnectWithArgument) => {
            let argument = admin_route.params().find("ARGUMENT").unwrap();
            log::info!("argument: {:?}", argument);
            let output = match admin.proxy.read().unwrap().nordvpn.connect(Some(argument.to_string())) {
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

        (&Method::POST, AdminRoutes::NordvpnDisconnect) => mk_response(format!("/nordvpn/disconnect: {:?}", admin.proxy.read().unwrap().nordvpn.disconnect())),
        (&Method::GET,  AdminRoutes::NordvpnLogs) => mk_response(serde_json::to_string(&admin.proxy.read().unwrap().nordvpn.logs(10)).unwrap()),
        (&Method::GET,  AdminRoutes::NordvpnLogsWithArgument) => {
            let argument = admin_route.params().find("LINES").unwrap();
            log::info!("argument: {:?}", argument);
            mk_response(serde_json::to_string(&admin.proxy.read().unwrap().nordvpn.logs(argument.parse().unwrap())).unwrap())
        },
        (&Method::POST,  AdminRoutes::NordvpnRotate) => {
            log::error!("Not implemented");
            panic!("not implemented");
            //mk_response(serde_json::to_string(&admin.proxy.read().unwrap().nordvpn.rotate()).unwrap())
        },

        (&Method::POST, AdminRoutes::NordvpnSanititze) => {
            let retention = Some(60);
            admin.proxy.write().unwrap().sanitize(retention);
            mk_response(serde_json::to_string(&admin.proxy.read().unwrap().status()).unwrap())
        },
        (&Method::GET,  AdminRoutes::NordvpnStatus) => mk_response(serde_json::to_string(&admin.proxy.read().unwrap().nordvpn.status()).unwrap()),

        (&Method::POST, AdminRoutes::NordvpnDaemonRestart) => mk_response(format!("/nordvpn/daemon/restart: {:?}", admin.proxy.read().unwrap().nordvpn.daemon_restart(Some(30)))),
        (&Method::GET,  AdminRoutes::NordvpnDaemonStatus) => mk_response(format!("/nordvpn/daemon/status: {:?}", admin.proxy.read().unwrap().nordvpn.daemon_status().output)),
        (&Method::POST, AdminRoutes::NordvpnDaemonStart) => mk_response(format!("/nordvpn/daemon/start: {:?}", admin.proxy.read().unwrap().nordvpn.daemon_start(Some(30)))),
        (&Method::POST, AdminRoutes::NordvpnDaemonStop) => mk_response(format!("/nordvpn/daemon/stop: {:?}", admin.proxy.read().unwrap().nordvpn.daemon_stop())),



        (&Method::POST, AdminRoutes::ProxyRotate) => {
            let resp = mk_response(serde_json::to_string(&admin.proxy.write().unwrap().rotate()).unwrap());
            resp
        },

        (&Method::GET,  AdminRoutes::ProxySettings) => mk_response(format!("/proxy/settings: {:?}", serde_json::to_string(&admin.proxy.read().unwrap().settings).unwrap())),
        

        (&Method::POST, AdminRoutes::ProxySettingsRotationInterval) => {
            let interval = admin_route.params().find("INTERVAL").unwrap();
            admin.proxy.write().unwrap().set_rotation_interval(interval.parse().unwrap());
            mk_response(serde_json::to_string(interval).unwrap())
        },
        
        (&Method::GET,  AdminRoutes::ProxyStatus) => mk_response(format!("/proxy/status: {:?}", serde_json::to_string(&admin.proxy.read().unwrap().status()).unwrap())),
        (&Method::GET,  AdminRoutes::ProxyStatusCompact) => mk_response(format!("/proxy/status: {:?}", serde_json::to_string(&admin.proxy.read().unwrap().compact_status()).unwrap())),

        (&Method::POST,  AdminRoutes::ProxyStatusPurge) => mk_response(serde_json::to_string(&admin.proxy.write().unwrap().purge(None)).unwrap()),

        _ => {
            log::warn!("Not found: {:?} {:?}", req.method(), req.uri().path());
            mk_response("oh no! not found".into())
        }
        
    };
    res

}
