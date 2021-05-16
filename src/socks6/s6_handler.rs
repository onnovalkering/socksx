use crate::address::ProxyAddress;
use crate::chain;
use crate::socks6::{self, Socks6Reply};
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct Socks6Handler {
    chain: Vec<ProxyAddress>,
}

impl Default for Socks6Handler {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Socks6Handler {
    ///
    ///
    ///
    pub fn new(chain: Option<Vec<ProxyAddress>>) -> Self {
        let chain = chain.unwrap_or_default();

        Socks6Handler { chain }
    }

    ///
    ///
    ///
    pub async fn handle_request(
        &self,
        source: &mut TcpStream,
    ) -> Result<()> {
        // Receive SOCKS request, and allow unauthenticated access.
        let request = socks6::read_request(source).await?;
        socks6::write_no_authentication(source).await?;

        let destination = request.destination.clone();
        let mut destination = if !self.chain.is_empty() {
            chain::setup(&self.chain, destination).await?
        } else {
            TcpStream::connect(destination.to_string()).await?
        };

        // Send initial data
        if request.initial_data_length > 0 {
            let mut initial_data = vec![0; request.initial_data_length as usize];
            source.read_exact(&mut initial_data).await?;
            destination.write(&initial_data).await?;
        }

        // Notify source that the connection has been set up.
        socks6::write_reply(source, Socks6Reply::Success).await?;
        source.flush().await?;

        // Start bidirectional copy, after this the connection closes.
        tokio::io::copy_bidirectional(source, &mut destination).await?;

        Ok(())
    }
}
