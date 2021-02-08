use crate::{constants::*, Address, Credentials};
use anyhow::{bail, ensure, Result};
use std::net::IpAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct Socks5Client {
    proxy_addr: String,
    credentials: Option<Credentials>,
}

impl Socks5Client {
    ///
    ///
    ///
    pub fn new<A: Into<String>>(
        proxy_addr: A,
        credentials: Option<Credentials>,
    ) -> Self {
        Socks5Client {
            proxy_addr: proxy_addr.into(),
            credentials,
        }
    }

    /// ...
    /// ...
    /// ...
    ///
    /// [rfc1928] https://tools.ietf.org/html/rfc1928
    pub async fn connect(
        &self,
        dst_addr: Address,
    ) -> Result<TcpStream> {
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

        // Gather and send SOCKS request information.
        let mut request: Vec<u8> = vec![SOCKS_VER_5, SOCKS_CMD_CONNECT, SOCKS_RSV];
        match dst_addr {
            Address::Ip { host, port } => {
                match host {
                    IpAddr::V4(host) => {
                        request.push(SOCKS_ATYP_IPV4);
                        request.extend(host.octets().iter());
                    }
                    IpAddr::V6(host) => {
                        request.push(SOCKS_ATYP_IPV6);
                        request.extend(host.octets().iter());
                    }
                }

                request.extend(port.to_be_bytes().iter())
            }
            Address::Domainname { host, port } => {
                request.push(SOCKS_ATYP_DOMAINNAME);

                let host = host.as_bytes();
                request.push(host.len() as u8);
                request.extend(host);

                request.extend(port.to_be_bytes().iter());
            }
        }

        stream.write(&request).await?;

        // We're only interrested in the reply's first few bytes.
        let mut response = [0; 4];
        stream.read_exact(&mut response).await?;

        let reply = response[1];
        if reply != SOCKS_REP_SUCCEEDED {
            bail!("CONNECT did not succeed: {}.", reply);
        }

        let atype = response[3];
        match atype {
            SOCKS_ATYP_IPV4 => {
                let mut bnd_addr = [0; 4];
                stream.read_exact(&mut bnd_addr).await?;
            }
            SOCKS_ATYP_IPV6 => {
                let mut bnd_addr = [0; 16];
                stream.read_exact(&mut bnd_addr).await?;
            }
            SOCKS_ATYP_DOMAINNAME => {
                let mut length = [0; 1];
                stream.read_exact(&mut length).await?;

                let mut bnd_addr = vec![0; length[0] as usize];
                stream.read_exact(&mut bnd_addr).await?;
            }
            _ => unreachable!(),
        }

        let mut response = [0; 2];
        stream.read_exact(&mut response).await?;

        Ok(stream)
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

        let mut response = [0; 2];
        stream.read_exact(&mut response).await?;

        let socks_version = response[0];
        if socks_version != SOCKS_VER_5 {
            bail!("Proxy uses a different SOCKS version: {}.", socks_version);
        }

        let auth_method = response[1];
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
        let username = credentials.username.clone();
        let password = credentials.password.clone();

        let mut request = vec![SOCKS_AUTH_VER];

        // Append username to request.
        request.push(username.len() as u8);
        request.extend(username);

        // Append password to request.
        request.push(password.len() as u8);
        request.extend(password);

        stream.write(&request).await?;

        let mut response = [0; 2];
        stream.read_exact(&mut response).await?;

        let auth_version = response[0];
        if auth_version != SOCKS_AUTH_VER {
            bail!(
                "Proxy uses a different authentication method version: {}.",
                auth_version
            );
        }

        // Check if status indicates success. If not, bail to close the connection.
        let status = response[1];
        if status != SOCKS_AUTH_SUCCESS {
            bail!("Authentication with the provided credentials failed.");
        }

        Ok(())
    }
}
