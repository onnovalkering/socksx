use crate::{constants::*, Address, Credentials};
use anyhow::{bail, ensure, Result};
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct Socks5Client {
    proxy_addr: SocketAddr,
    credentials: Option<Credentials>,
}

impl Socks5Client {
    ///
    ///
    ///
    pub async fn new<A: Into<String>>(
        proxy_addr: A,
        credentials: Option<Credentials>,
    ) -> Result<Self> {
        let proxy_addr = crate::resolve_addr(proxy_addr).await?;

        Ok(Socks5Client {
            proxy_addr,
            credentials,
        })
    }

    /// ...
    /// ...
    /// ...
    ///
    /// [rfc1928] https://tools.ietf.org/html/rfc1928
    pub async fn connect<A: Into<Address>>(
        &self,
        dst_addr: A,
    ) -> Result<(TcpStream, Address)> {
        if let Some(Credentials { username, password }) = &self.credentials {
            ensure!(username.len() > 255, "Username can be no longer than 255 bytes.");
            ensure!(password.len() > 255, "Password can be no longer than 255 bytes.");
        }

        let mut stream = TcpStream::connect(&self.proxy_addr).await?;

        // Enter authentication negotiation.
        let auth_method = self.negotiate_auth_method(&mut stream).await?;
        if auth_method == SOCKS_AUTH_USERNAME_PASSWORD {
            if let Some(credentials) = &self.credentials {
                self.authenticate(&mut stream, credentials).await?;
            } else {
                unreachable!();
            }
        }

        // Send SOCKS request information.
        let mut request: Vec<u8> = vec![SOCKS_VER_5, SOCKS_CMD_CONNECT, SOCKS_RSV];
        request.extend(dst_addr.into().as_socks_bytes());

        stream.write(&request).await?;

        // Read the first few reply bytes to determine the status.
        let mut reply = [0; 4];
        stream.read_exact(&mut reply).await?;

        let rep = reply[1];
        if rep != SOCKS_REP_SUCCEEDED {
            bail!("CONNECT did not succeed: {}.", rep);
        }

        // On success, the remaining bytes contain the binding.
        let atyp = reply[3];
        let binding = match atyp {
            SOCKS_ATYP_IPV4 => {
                let mut bnd_addr = [0; 4];
                stream.read_exact(&mut bnd_addr).await?;

                let mut bnd_port = [0; 2];
                stream.read_exact(&mut bnd_port).await?;

                (bnd_addr, bnd_port).into()
            }
            SOCKS_ATYP_IPV6 => {
                let mut bnd_addr = [0; 16];
                stream.read_exact(&mut bnd_addr).await?;

                let mut bnd_port = [0; 2];
                stream.read_exact(&mut bnd_port).await?;

                (bnd_addr, bnd_port).into()
            }
            SOCKS_ATYP_DOMAINNAME => {
                let mut length = [0; 1];
                stream.read_exact(&mut length).await?;

                let mut bnd_addr = vec![0; length[0] as usize];
                stream.read_exact(&mut bnd_addr).await?;

                let mut bnd_port = [0; 2];
                stream.read_exact(&mut bnd_port).await?;

                (String::from_utf8(bnd_addr)?, bnd_port).into()
            }
            _ => unreachable!(),
        };

        Ok((stream, binding))
    }

    /// ...
    /// ...
    /// ...
    ///
    /// [rfc1928] https://tools.ietf.org/html/rfc1928
    async fn negotiate_auth_method(
        &self,
        stream: &mut TcpStream,
    ) -> Result<u8> {
        let mut request = vec![SOCKS_VER_5, 0x01, SOCKS_AUTH_NOT_REQUIRED];
        if self.credentials.is_some() {
            request[1] = 0x02;
            request.push(SOCKS_AUTH_USERNAME_PASSWORD);
        }

        stream.write(&request).await?;

        let mut reply = [0; 2];
        stream.read_exact(&mut reply).await?;

        let socks_version = reply[0];
        if socks_version != SOCKS_VER_5 {
            bail!("Proxy uses a different SOCKS version: {}.", socks_version);
        }

        let auth_method = reply[1];
        match auth_method {
            0x00 => Ok(auth_method),
            0x02 => {
                if self.credentials.is_none() {
                    bail!("Proxy demands authentication, but no credentials are provided.");
                } else {
                    Ok(auth_method)
                }
            }
            0xFF => bail!("Proxy did not accept authentication method."),
            _ => bail!("Proxy proposed unsupported authentication method: {}.", auth_method),
        }
    }

    /// ...
    /// ...
    /// ...
    ///
    /// [rfc1929] https://tools.ietf.org/html/rfc1929
    async fn authenticate(
        &self,
        stream: &mut TcpStream,
        credentials: &Credentials,
    ) -> Result<()> {
        let mut request = vec![SOCKS_AUTH_VER];
        request.extend(credentials.as_socks_bytes());

        stream.write(&request).await?;

        let mut reply = [0; 2];
        stream.read_exact(&mut reply).await?;

        let auth_version = reply[0];
        if auth_version != SOCKS_AUTH_VER {
            bail!(
                "Proxy uses a different authentication method version: {}.",
                auth_version
            );
        }

        // Check if status indicates success. If not, bail to close the connection.
        let status = reply[1];
        if status != SOCKS_AUTH_SUCCESS {
            bail!("Authentication with the provided credentials failed.");
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct Socks5Guard {
    credentials: Option<Credentials>,
}

impl Socks5Guard {
    ///
    ///
    ///
    pub fn new(credentials: Option<Credentials>) -> Self {
        Socks5Guard { credentials }
    }

    ///
    ///
    ///
    pub async fn authenticate(
        &self,
        stream: &mut TcpStream,
    ) -> Result<()> {
        let mut request = [0; 2];
        stream.read_exact(&mut request).await?;

        let socks_version = request[0];

        if socks_version != SOCKS_VER_5 {
            bail!("Client uses a different SOCKS version: {}.", socks_version);
        }

        // Get all authentication methods the client proposes.
        let nmethods = request[1] as usize;

        let mut methods = vec![0; nmethods];
        stream.read_exact(&mut methods).await?;

        let method = if self.credentials.is_some() && methods.contains(&SOCKS_AUTH_USERNAME_PASSWORD) {
            SOCKS_AUTH_USERNAME_PASSWORD
        } else if methods.contains(&SOCKS_AUTH_NOT_REQUIRED) {
            SOCKS_AUTH_NOT_REQUIRED
        } else {
            SOCKS_AUTH_NO_ACCEPTABLE_METHODS
        };

        info!("Use authentication method: {}", method);

        let response = [SOCKS_VER_5, method];
        stream.write(&response).await?;

        // Enter method-specific sub-negotiation
        if method == SOCKS_AUTH_USERNAME_PASSWORD {
            let mut request = [0; 2];
            stream.read_exact(&mut request).await?;

            let auth_version = request[0];
            if auth_version != SOCKS_AUTH_VER {
                bail!(
                    "Client uses a different authentication method version: {}.",
                    auth_version
                );
            }

            let ulen = request[1] as usize;
            let mut uname = vec![0; ulen];
            stream.read_exact(&mut uname).await?;

            let plen = request[1] as usize;
            let mut passwd = vec![0; plen];
            stream.read_exact(&mut passwd).await?;

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
            stream.write(&response).await?;

            ensure!(status == SOCKS_AUTH_SUCCESS, "Username/password authentication failed.");
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct Socks5Handler {}

impl Socks5Handler {
    ///
    ///
    ///
    pub fn new() -> Self {
        Socks5Handler {}
    }

    pub async fn handle_request(
        &self,
        stream: &mut TcpStream,
    ) -> Result<()> {
        let mut request = [0; 4];
        stream.read_exact(&mut request).await?;

        let command = request[1];
        if command != SOCKS_CMD_CONNECT {
            unimplemented!();
        }

        let atype = request[3];
        let dst_addr = match atype {
            SOCKS_ATYP_IPV4 => {
                let mut dst_addr = [0; 4];
                stream.read_exact(&mut dst_addr).await?;

                IpAddr::from(dst_addr).to_string()
            }
            SOCKS_ATYP_IPV6 => {
                let mut dst_addr = [0; 16];
                stream.read_exact(&mut dst_addr).await?;

                IpAddr::from(dst_addr).to_string()
            }
            SOCKS_ATYP_DOMAINNAME => {
                let mut length = [0; 1];
                stream.read_exact(&mut length).await?;

                let mut dst_addr = vec![0; length[0] as usize];
                stream.read_exact(&mut dst_addr).await?;

                String::from_utf8(dst_addr.to_vec())?
            }
            _ => unreachable!(),
        };

        let mut dst_port = [0; 2];
        stream.read_exact(&mut dst_port).await?;

        let dst_port = ((dst_port[0] as u16) << 8) | dst_port[1] as u16;
        let dst = format!("{}:{}", dst_addr, dst_port);

        let mut out = TcpStream::connect(dst).await?;

        let mut reply = [
            SOCKS_VER_5,
            SOCKS_REP_SUCCEEDED,
            SOCKS_RSV,
            SOCKS_ATYP_IPV4,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        stream.write(&mut reply).await?;
        stream.flush().await?;

        tokio::io::copy_bidirectional(stream, &mut out).await?;

        Ok(())
    }
}
