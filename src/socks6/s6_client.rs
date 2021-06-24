use crate::socks6::{self, Socks6Request};
use crate::socks6::{
    options::{AuthMethodAdvertisementOption, SocksOption},
    AuthMethod,
};
use crate::{constants::*, Address, Credentials};
use anyhow::{ensure, Result};
use std::{convert::TryInto, net::SocketAddr};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct Socks6Client {
    proxy_addr: SocketAddr,
    credentials: Option<Credentials>,
}

impl Socks6Client {
    ///
    ///
    ///
    pub async fn new<A: Into<String>>(
        proxy_addr: A,
        credentials: Option<Credentials>,
    ) -> Result<Self> {
        let proxy_addr = crate::resolve_addr(proxy_addr).await?;

        Ok(Socks6Client {
            proxy_addr,
            credentials,
        })
    }

    /// 
    /// 
    /// 
    pub async fn connect<A>(
        &self,
        destination: A,
        initial_data: Option<Vec<u8>>,
        options: Option<Vec<SocksOption>>,
    ) -> Result<(TcpStream, Address)>
    where
        A: TryInto<Address, Error = anyhow::Error>,
    {
        let mut stream = TcpStream::connect(&self.proxy_addr).await?;
        let binding = self.handshake(destination, initial_data, options, &mut stream).await?;

        Ok((stream, binding))
    }

    /// ...
    /// ...
    /// ...
    /// [socks6-draft11] https://tools.ietf.org/html/draft-olteanu-intarea-socks-6-11
    pub async fn handshake<A>(
        &self,
        destination: A,
        initial_data: Option<Vec<u8>>,
        options: Option<Vec<SocksOption>>,
        stream: &mut TcpStream,
    ) -> Result<Address>
    where
        A: TryInto<Address, Error = anyhow::Error>,
    {
        if let Some(Credentials { username, password }) = &self.credentials {
            ensure!(username.len() > 255, "Username MUST NOT be larger than 255 bytes.");
            ensure!(password.len() > 255, "Password MUST NOT be larger than 255 bytes.");
        }

        // Prepare initial data.
        let initial_data = initial_data.unwrap_or_default();
        ensure!(
            initial_data.len() <= 2 ^ 14,
            "Initial data MUST NOT be larger than 16384 bytes."
        );
        let initial_data_length = initial_data.len() as u16;

        // Prepare SOCKS options.
        let mut auth_methods = vec![];
        if self.credentials.is_some() {
            auth_methods.push(AuthMethod::UsernamePassword);
        }

        let auth_methods_adv = AuthMethodAdvertisementOption::new(initial_data_length, vec![]);
        let mut options = options.unwrap_or_default();
        options.push(auth_methods_adv.wrap());

        // Create SOCKS6 CONNECT request.
        let request = Socks6Request::new(
            SOCKS_CMD_CONNECT,
            destination.try_into()?,
            initial_data_length,
            options,
            None,
        );

        // Send SOCKS request information.
        let request_bytes = request.into_socks_bytes();
        stream.write(&request_bytes).await?;

        // Wait for authentication and operation reply.
        let _ = socks6::read_no_authentication(stream).await?;
        let (binding, _) = socks6::read_reply(stream).await?;

        Ok(binding)
    }
}
