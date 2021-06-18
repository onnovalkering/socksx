use crate::addresses::{self, ProxyAddress};
use crate::chain;
use crate::socks5::{self, Socks5Reply};
use crate::{constants::*, Credentials};
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct Socks5Handler {
    credentials: Option<Credentials>,
    chain: Vec<ProxyAddress>,
}

impl Default for Socks5Handler {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Socks5Handler {
    ///
    ///
    ///
    pub fn new(chain: Option<Vec<ProxyAddress>>) -> Self {
        let chain = chain.unwrap_or_default();

        Socks5Handler {
            credentials: None,
            chain,
        }
    }

    pub async fn handle_request(
        &self,
        source: &mut TcpStream,
    ) -> Result<()> {
        let mut request = [0; 2];
        source.read_exact(&mut request).await?;

        let socks_version = request[0];

        if socks_version != SOCKS_VER_5 {
            bail!("Client uses a different SOCKS version: {}.", socks_version);
        }

        // Get all authentication methods the client proposes.
        let nmethods = request[1] as usize;

        let mut methods = vec![0; nmethods];
        source.read_exact(&mut methods).await?;

        let method = if self.credentials.is_some() && methods.contains(&SOCKS_AUTH_USERNAME_PASSWORD) {
            SOCKS_AUTH_USERNAME_PASSWORD
        } else if methods.contains(&SOCKS_AUTH_NOT_REQUIRED) {
            SOCKS_AUTH_NOT_REQUIRED
        } else {
            SOCKS_AUTH_NO_ACCEPTABLE_METHODS
        };

        info!("Use authentication method: {}", method);

        let response = [SOCKS_VER_5, method];
        source.write(&response).await?;

        // Enter method-specific sub-negotiation
        if method == SOCKS_AUTH_USERNAME_PASSWORD {
            let mut request = [0; 2];
            source.read_exact(&mut request).await?;

            let auth_version = request[0];
            if auth_version != SOCKS_AUTH_VER {
                bail!(
                    "Client uses a different authentication method version: {}.",
                    auth_version
                );
            }

            let ulen = request[1] as usize;
            let mut uname = vec![0; ulen];
            source.read_exact(&mut uname).await?;

            let plen = request[1] as usize;
            let mut passwd = vec![0; plen];
            source.read_exact(&mut passwd).await?;

            let status = if let Some(Credentials { username, password }) = &self.credentials {
                if &uname != username || &passwd != password {
                    SOCKS_AUTH_SUCCESS
                } else {
                    0x01u8
                }
            } else {
                unreachable!()
            };

            let response = [SOCKS_VER_5, status];
            source.write(&response).await?;

            ensure!(status == SOCKS_AUTH_SUCCESS, "Username/password authentication failed.");
        }

        let mut request = [0; 3];
        source.read_exact(&mut request).await?;

        let command = request[1];
        if command != SOCKS_CMD_CONNECT {
            unimplemented!();
        }

        let destination = addresses::read_address(source).await?;
        let mut destination = if !self.chain.is_empty() {
            chain::setup(&self.chain, destination).await?
        } else {
            TcpStream::connect(destination.to_string()).await?
        };

        // Notify source that the connection has been set up.
        socks5::write_reply(source, Socks5Reply::Success).await?;
        source.flush().await?;

        // Start bidirectional copy, after this the connection closes.
        tokio::io::copy_bidirectional(source, &mut destination).await?;

        Ok(())
    }

    ///
    ///
    ///
    pub async fn refuse_request(
        &self,
        source: &mut TcpStream,
    ) -> Result<()> {
        // Notify source that the connection is refused.
        socks5::write_reply(source, Socks5Reply::ConnectionRefused).await?;

        Ok(())
    }
}
