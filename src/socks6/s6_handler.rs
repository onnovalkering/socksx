use crate::socks6::{self, SocksReply};
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct Socks6Handler {}

impl Default for Socks6Handler {
    fn default() -> Self {
        Self::new()
    }
}

impl Socks6Handler {
    ///
    ///
    ///
    pub fn new() -> Self {
        Socks6Handler {}
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
        socks6::no_authentication(source).await?;

        // Connect to destination and send initial data.
        let mut destination = TcpStream::connect(request.destination.to_string()).await?;
        if request.initial_data_length > 0 {
            let mut initial_data = vec![0; request.initial_data_length as usize];
            source.read_exact(&mut initial_data).await?;
            destination.write(&initial_data).await?;
        }

        // Notify source that the connection has been set up.
        socks6::write_reply(source, SocksReply::Success).await?;
        source.flush().await?;

        // Start bidirectional copy, after this the connection closes.
        tokio::io::copy_bidirectional(source, &mut destination).await?;

        Ok(())
    }
}
