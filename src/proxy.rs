use std::error::Error;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;


pub struct Proxy<'a> {
    port: u16,
    server: &'a str,
    connectionCount: u32,
}

impl<'a> Proxy<'a> {
    pub fn new() -> Self {
        log::info!("Creating new Proxy instance");
        return Self {
            port: 3128,
            server: "0.0.0.0",
            connectionCount: 0,
        };
    }
    pub async fn start(&mut self) {
        log::info!("Starting Proxy server");
        self.listen(self.server, self.port).await.unwrap();
        // continue with: https://hyper.rs/guides/1/server/hello-world/
    }

    async fn handle_connection(r: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        match r.method() {
            &hyper::Method::CONNECT => {
                log::info!("CONNECT request");
            }
            &hyper::Method::GET => {
                log::info!("GET request");
            }
            &hyper::Method::POST => {
                log::info!("POST request");
            }
            _ => {
                log::info!("Unknown request");
            }
            
        }
        Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
    }
    

    async fn listen(&mut self, ip: &str, port: u16) -> Result<(), Box<dyn Error>> {

        
        let addr = SocketAddr::from_str(format!("{}:{}", ip, port).as_str())?;
        let listener = TcpListener::bind(addr).await?;
        log::info!("Listening on {}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            self.connectionCount += 1;
            log::info!("Connection #{}", self.connectionCount);
    
            // Use an adapter to access something implementing `tokio::io` traits as if they implement
            // `hyper::rt` IO traits.
            let io = TokioIo::new(stream);
    
            // Spawn a tokio task to serve multiple connections concurrently
            tokio::task::spawn(async move {
                // Finally, we bind the incoming connection to our `hello` service
                if let Err(err) = http1::Builder::new()
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(io, service_fn(Self::handle_connection))
                    .await
                {
                    log::error!("Error serving connection: {:?}", err);
                }
            });
        }
    

    
    }

    pub async fn stop(self) {
        log::info!("Stopping Proxy server");
        //self.listener.take().unwrap().unbind().unwrap();
    }
}