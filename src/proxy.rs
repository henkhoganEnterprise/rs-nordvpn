////////////////////////////////////////////////////////////////////////////////////////////////////
// based on https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs  //////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
#![deny(warnings)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::{combinators::BoxBody, Empty, Full};
use hyper::client::conn::http1::Builder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Method, Request, Response};


use serde_derive::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};



#[path = "./benches/support/mod.rs"]
mod support;
use support::TokioIo;
use tokio_util::bytes;


// To try this example:
// 1. cargo run --example http_proxy
// 2. config http_proxy in command line
//    $ export http_proxy=http://0.0.0.0:8100
//    $ export https_proxy=http://0.0.0.0:8100
// 3. send requests
//    $ curl -i https://www.some_domain.com/
pub async fn run(proxy_state: Arc<Mutex<ProxyState>>, bind_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {

    let listener = TcpListener::bind(bind_addr).await?;
    log::info!("Proxy Listening on http://{}", bind_addr);

    loop {
        let (stream, _) = listener.accept().await?;

        for _ in 0..1000 {
            // trick from https://tokio.rs/tokio/tutorial/shared-state to prevent: 
            // error: future cannot be sent between threads safely
            {
                let mut proxy_lock = proxy_state.lock().unwrap();
                if !proxy_lock.drained {
                    proxy_lock.add_connection();
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let peer_addr = stream.peer_addr()?;
        log::info!("Proxy accepted a new TCP connection from: {}", peer_addr);
        let io = TokioIo::new(stream);

        let _proxy_state = proxy_state.clone();
        tokio::task::spawn(async move {
            let __proxy_state = _proxy_state;
            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service_fn(move |req| proxy(__proxy_state.clone(), req)))
                .with_upgrades()
                .await
            {
                log::error!("Failed to serve connection: {:?}", err);
            }
        });
        log::info!("Proxy connection closed from {}", peer_addr);
        proxy_state.lock().unwrap().remove_connection();
    }
}


async fn proxy(
    proxy_state: Arc<Mutex<ProxyState>>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    log::info!("req: {:?}", req);

    if Method::CONNECT == req.method() {
        // Received an HTTP request like:
        // ```
        // CONNECT www.domain.com:443 HTTP/1.1
        // Host: www.domain.com:443
        // Proxy-Connection: Keep-Alive
        // ```
        //
        // When HTTP method is CONNECT we should return an empty body
        // then we can eventually upgrade the connection and talk a new protocol.
        //
        // Note: only after client received an empty body with STATUS_OK can the
        // connection be upgraded, so we can't return a response inside
        // `on_upgrade` future.

        let host = req.uri().host().expect("uri has no host");
        proxy_state.lock().unwrap().add_connect_request(host);
        if let Some(addr) = host_addr(req.uri()) {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = tunnel(upgraded, addr).await {
                            log::error!("server io error: {}", e);
                        };
                    }
                    Err(e) => log::error!("upgrade error: {}", e),
                }
            });
            proxy_state.lock().unwrap().remove_connect_request();
            Ok(Response::new(empty()))
        } else {
            log::error!("CONNECT host is not socket addr: {:?}", req.uri());
            let mut resp = Response::new(full("CONNECT must be to a socket address"));
            *resp.status_mut() = http::StatusCode::BAD_REQUEST;
            proxy_state.lock().unwrap().remove_connect_request();
            Ok(resp)
        }
    } else {
        let host = req.uri().host().expect("uri has no host");
        let port = req.uri().port_u16().unwrap_or(80);

        let stream = TcpStream::connect((host, port)).await.unwrap();
        let io = TokioIo::new(stream);

        let (mut sender, conn) = Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                log::error!("Connection failed: {:?}", err);
            }
        });

        let resp = sender.send_request(req).await?;
        Ok(resp.map(|b| b.boxed()))
    }
}

fn host_addr(uri: &http::Uri) -> Option<String> {
    uri.authority().and_then(|auth| Some(auth.to_string()))
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

// Create a TCP connection to host:port, build a tunnel between the connection and
// the upgraded connection
async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    // Connect to remote server
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    // Proxying data
    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    // Print message when done
    log::info!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );

    Ok(())
}

/*

*/
#[derive(Serialize, Deserialize)]
pub struct ProxyStatus {
    drained: bool,
    inflight_connections: u16,
    inflight_connect_requests: u16,
    monitored_hosts: HashMap<String, (Option<SystemTime>, Vec<SystemTime>)>
}

#[derive(Serialize, Deserialize)]
pub struct ProxyStatusCompact {
    drained: bool,
    inflight_connections: u16,
    inflight_connect_requests: u16,
    monitored_hosts: HashMap<String, i32>
}

#[derive(Debug, Clone)]
pub struct ProxyState {
    drained: bool,
    inflight_connections: u16,
    inflight_connect_requests: u16,
    monitored_hosts: HashMap<String, (Option<SystemTime>, Vec<SystemTime>)>
}

impl ProxyState {
    pub fn new(monitored_hosts: Vec<String>) -> Self {
        ProxyState {
            drained: false,
            inflight_connections: 0,
            inflight_connect_requests: 0,
            monitored_hosts: monitored_hosts.iter().map(|host| (host.clone(), (None, vec![]))).collect()
        }
    }

    pub fn compact_status(&self) -> ProxyStatusCompact {
        ProxyStatusCompact {
            drained: self.drained,
            inflight_connections: self.inflight_connections,
            inflight_connect_requests: self.inflight_connect_requests,
            monitored_hosts: self.monitored_hosts.iter().map(|(host, (_last, times))| (host.clone(), times.len() as i32)).collect()
        }
    }

    pub fn purge(&mut self) {
        self.monitored_hosts.iter_mut().for_each(|(_host, (_last, times))| {
            times.retain(|time| time.elapsed().unwrap().as_secs() < 60);
        });
    }

    pub fn status(&self) -> ProxyStatus {
        ProxyStatus {
            drained: self.drained,
            inflight_connections: self.inflight_connections,
            inflight_connect_requests: self.inflight_connect_requests,
            monitored_hosts: self.monitored_hosts.clone()
        }
    }

    pub fn add_connection(&mut self) {
        self.inflight_connections += 1;
    }

    pub fn remove_connection(&mut self) {
        self.inflight_connections -= 1;
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

}
