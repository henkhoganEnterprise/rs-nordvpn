use std::error::Error;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;


pub struct Proxy {
    port: u16,
    server: String,
    listener: Option<TcpListener>,
}

impl Proxy {
    pub fn new() -> Self {
        log::info!("Creating new Proxy instance");
        return Self {
            port: 3128,
            server: "127.0.0.1".to_string(),
            listener: None,
        };
    }
    pub async fn start(&mut self) {
        log::info!("Starting Proxy server");
        self.listen(self.server.clone(), self.port).await.unwrap();
    }

    async fn handle_client(&mut self, mut stream: tokio::net::TcpStream, addr: std::net::SocketAddr) {
        let mut buf = [0; 1024];
        loop {
            match stream.read(&mut buf).await {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    if let Ok(s) = std::str::from_utf8(&buf[0..n]) {
                        println!("Received: {}", s);
                    }
                }
                Err(e) => {
                    println!("Failed to read from socket; err = {:?}", e);
                    break;
                }
            }
        }
    }

    async fn listen(&mut self, ip: String, port: u16) -> Result<(), Box<dyn Error>> {

        let listener = TcpListener::bind(format!("{}:{}", ip, port)).await?;

        match listener.accept().await {
            Ok((_socket, addr)) => self.handle_client(_socket, addr).await,
            Err(e) => println!("couldn't get client: {:?}", e),
        }

        Ok(())

    
    }

    pub async fn stop(self) {
        log::info!("Stopping Proxy server");
        //self.listener.take().unwrap().unbind().unwrap();
    }
}