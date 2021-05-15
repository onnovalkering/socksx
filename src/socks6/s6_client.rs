use crate::socks6::options::{AuthMethodAdvertisementOption, SocksOption};
use crate::{constants::*, Address, Credentials};
use anyhow::{ensure, Result};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

    /// ...
    /// ...
    /// ...
    /// [socks6-draft11] https://tools.ietf.org/html/draft-olteanu-intarea-socks-6-11
    pub async fn connect<A: Into<Address>>(
        &self,
        dst_addr: A,
        initial_data: Option<Vec<u8>>,
        options: Option<Vec<SocksOption>>,
    ) -> Result<(TcpStream, Address)> {
        if let Some(Credentials { username, password }) = &self.credentials {
            ensure!(username.len() > 255, "Username can be no longer than 255 bytes.");
            ensure!(password.len() > 255, "Password can be no longer than 255 bytes.");
        }

        let dst_addr = dst_addr.into();
        let initial_data = initial_data.unwrap_or_default();

        // Prepare SOCKS options
        let mut auth_option_data = vec![];
        auth_option_data.extend((initial_data.len() as u16).to_be_bytes().iter());
        if self.credentials.is_some() {
            auth_option_data.push(SOCKS_AUTH_USERNAME_PASSWORD)
        }

        let auth_meth_adv_option = AuthMethodAdvertisementOption::new(initial_data.len() as u16, vec![]);

        let options = if let Some(mut options) = options.clone() {
            options.push(auth_meth_adv_option);
            options
        } else {
            vec![auth_meth_adv_option]
        };

        let options_bytes: Vec<u8> = options.iter().flat_map(|o| o.as_socks_bytes()).collect();

        // Prepare SOCKS request
        let mut request: Vec<u8> = vec![SOCKS_VER_6, SOCKS_CMD_CONNECT];
        request.extend(dst_addr.as_socks_bytes());
        request.push(SOCKS_PADDING);
        request.extend((options_bytes.len() as u16).to_be_bytes().iter());
        request.extend(options_bytes.iter());

        dbg!(&request);

        // Send SOCKS request information.
        let mut stream = TcpStream::connect(&self.proxy_addr).await?;
        stream.write(&request).await?;
        if !initial_data.is_empty() {
            stream.write(&initial_data).await?;
        }

        // check !

        // Wait for authentication reply.
        let mut reply = [0; 1];
        stream.read_exact(&mut reply).await?;

        let socks_version = reply[0];
        ensure!(
            socks_version == SOCKS_VER_6,
            "Proxy uses a different SOCKS version: {}",
            socks_version
        );

        let mut reply = [0; 3];
        stream.read_exact(&mut reply).await?;

        let status = reply[0];
        ensure!(
            status == SOCKS_AUTH_SUCCESS,
            "Authentication with proxy failed: {}",
            status
        );

        let options_length = ((reply[1] as u16) << 8) | reply[2] as u16;
        let mut reply_options = vec![0; options_length as usize];
        stream.read_exact(&mut reply_options).await?;

        // check !

        // Wait for operation reply.
        let mut operation_reply = [0; 6];
        stream.read_exact(&mut operation_reply).await?;

        let reply_code = operation_reply[1];
        ensure!(
            reply_code == SOCKS_REP_SUCCEEDED,
            "CONNECT operation failed: {}",
            reply_code
        );

        let bnd_port = [operation_reply[2], operation_reply[3]];

        let atyp = operation_reply[5];
        let binding = match atyp {
            SOCKS_ATYP_IPV4 => {
                let mut bnd_addr = [0; 4];
                stream.read_exact(&mut bnd_addr).await?;

                (bnd_addr, bnd_port).into()
            }
            SOCKS_ATYP_IPV6 => {
                let mut bnd_addr = [0; 16];
                stream.read_exact(&mut bnd_addr).await?;

                (bnd_addr, bnd_port).into()
            }
            SOCKS_ATYP_DOMAINNAME => {
                let mut length = [0; 1];
                stream.read_exact(&mut length).await?;

                let mut bnd_addr = vec![0; length[0] as usize];
                stream.read_exact(&mut bnd_addr).await?;

                (String::from_utf8(bnd_addr)?, bnd_port).into()
            }
            _ => unreachable!(),
        };

        let mut options_length = [0; 2];
        stream.read_exact(&mut options_length).await?;

        let options_length = ((options_length[0] as u16) << 8) | options_length[1] as u16;
        let mut reply_options = vec![0; options_length as usize];
        stream.read_exact(&mut reply_options).await?;

        Ok((stream, binding))
    }
}
