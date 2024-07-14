use std::error::Error;
use std::io::Read;
use tokio::net::TcpListener;


pub struct Proxy {
    port: u16,
    server: String,
}

impl Proxy {
    pub fn new() -> Self {
        log::info!("Creating new Proxy instance");
        return Self {
            port: 3128,
            server: "127.0.0.1".to_string(),
        };
    }
    pub async fn start(&self) {
        log::info!("Starting Proxy server");
        self.listen(self.server.clone(), self.port).await.unwrap();
    }

    pub async fn listen(&self, ip: String, port: u16) -> Result<(), Box<dyn Error>> {
        let mut data = [0u8; 12];
        let listener = TcpListener::bind(format!("{}:{}", ip, port)).await?;
        let (tokio_tcp_stream, _) = listener.accept().await?;
        let mut std_tcp_stream = tokio_tcp_stream.into_std()?;
        std_tcp_stream.set_nonblocking(false)?;
        std_tcp_stream.read_exact(&mut data)?;
        Ok(())    
    }
}