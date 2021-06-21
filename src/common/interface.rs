use anyhow::Result;
use async_trait::async_trait;
use tokio::net::TcpStream;

#[async_trait]
pub trait SocksHandler {
    async fn handle_request(
        &self,
        source: &mut TcpStream,
    ) -> Result<()>;

    async fn refuse_request(
        &self,
        source: &mut TcpStream,
    ) -> Result<()>;
}
