////////////////////////////////////////////////////////////////////////////////////////////////////
// based on https://github.com/hyperium/hyper/blob/master/examples/http_proxy.rs  //////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
#![deny(warnings)]

use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::{combinators::BoxBody, Empty, Full};
use hyper::client::conn::http1::Builder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade::Upgraded;
use hyper::{Method, Request, Response};


use tokio::net::{TcpListener, TcpStream};
use tokio_util::bytes;



#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;

use super::{ProxyState, RunReturnType};

pub async fn run(proxy_state: Arc<RwLock<ProxyState>>, bind_addr: SocketAddr) -> RunReturnType {

    let listener = TcpListener::bind(bind_addr).await?;
    log::info!("Proxy Listening on http://{}", bind_addr);

    
    loop {
        
        let (stream, _) = listener.accept().await?;
    
        let peer_addr = stream.peer_addr()?;
        log::info!("Proxy accepted a new TCP connection from: {}", peer_addr);
        let io = TokioIo::new(stream);
    
        let proxy_state = proxy_state.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(
                    io, 
                    service_fn(move |req| proxy(proxy_state.clone(), peer_addr, req))
                )
                .with_upgrades()

                .await
            {
                log::error!("Failed to serve connection: {:?}", err);
            }
        });
        //log::info!("Proxy connection closed from {}", peer_addr);
    }
}


async fn proxy(
    proxy_state: Arc<RwLock<ProxyState>>,
    peer_addr: SocketAddr,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {

    log::info!("req: {:?}", req);
    proxy_state.write().unwrap().add_connection(peer_addr.to_string());

    let res: Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>;

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

        return connect_handler(req, peer_addr.to_string(), proxy_state.clone()).await;
    } 

    /*
    if req.uri().host().is_none() {
        log::error!("Request URI has no host: {:?}", req.uri());
        res = Ok(Response::new(full("Request URI has no host")));
    }
    */
    
    else {

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
                proxy_state.write().unwrap().remove_connection(peer_addr.to_string());
            }
        });

        let resp = sender.send_request(req).await?;
        res = Ok(resp.map(|b| b.boxed()));
    }

    res
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

async fn connect_handler(req: Request<hyper::body::Incoming>, peer_addr: String, proxy_state: Arc<RwLock<ProxyState>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let host = req.uri().host().expect("uri has no host");
    if let Some(addr) = host_addr(req.uri()) {

        proxy_state.write().unwrap().add_connect_request(host);
        tokio::task::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    match tunnel(upgraded, addr).await {
                        Ok((from_client, from_server)) => {
                            log::info!("tunnel closed: {} bytes from client, {} bytes from server", from_client, from_server);
                        }
                        Err(e) => {
                            log::error!("tunnel creation error: {},{}", e.kind(), e);
                        }
                    }

                }
                Err(e) => {
                    log::error!("upgrade error: {}", e);
                },
            }
            proxy_state.write().unwrap().remove_connect_request();
            proxy_state.write().unwrap().remove_connection(peer_addr.to_string());
        });
        //t.await.unwrap();

        
        Ok(Response::new(empty()))
    } else {
        log::error!("CONNECT host is not socket addr: {:?}", req.uri());
        let mut resp = Response::new(full("CONNECT must be to a socket address"));
        *resp.status_mut() = http::StatusCode::BAD_REQUEST;
        Ok(resp)
    }
}




// Create a TCP connection to host:port, build a tunnel between the connection and
// the upgraded connection
async fn tunnel(upgraded: Upgraded, addr: String) -> std::io::Result<(u64, u64)> {
    // Connect to remote server
    let mut server = TcpStream::connect(addr).await?;
    let mut upgraded = TokioIo::new(upgraded);

    // Proxying data
    let (from_client, from_server) =
        tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

    Ok((from_client, from_server))
}
