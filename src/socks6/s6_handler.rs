use crate::constants::*;
use anyhow::Result;
use std::net::IpAddr;
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
        stream: &mut TcpStream,
    ) -> Result<()> {
        // Read SOCKS request
        let mut request = [0; 3];
        stream.read_exact(&mut request).await?;

        let version = request[0];
        if version != SOCKS_VER_6 {
            stream.write_u8(SOCKS_VER_6).await?;

            // A mismatch is not an error.
            return Ok(());
        }

        let command = request[1];
        if command != SOCKS_CMD_CONNECT {
            unimplemented!();
        }

        let atype = request[2];
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

        // Read destination port and padding (ignored).
        let mut dst_port = [0; 3];
        stream.read_exact(&mut dst_port).await?;

        let dst_port = ((dst_port[0] as u16) << 8) | dst_port[1] as u16;
        let dst = format!("{}:{}", dst_addr, dst_port);

        // Read options
        let mut options_length = [0; 2];
        stream.read_exact(&mut options_length).await?;

        let options_length = ((options_length[0] as u16) << 8) | options_length[1] as u16;

        let mut reply_options = vec![0; options_length as usize];
        stream.read_exact(&mut reply_options).await?;

        let initial_data_len = ((reply_options[4] as u16) << 8) | reply_options[5] as u16;

        let mut initial_data = vec![0; initial_data_len as usize];
        stream.read_exact(&mut initial_data).await?;

        // Write auth reply
        let auth_reply = [SOCKS_VER_6, SOCKS_AUTH_SUCCESS, 0x00u8, 0x00u8];
        stream.write(&auth_reply).await?;

        // Open socket and send initial data
        let mut out = TcpStream::connect(dst).await?;

        out.write(&initial_data).await?;

        let reply = [
            SOCKS_VER_6,
            SOCKS_REP_SUCCEEDED,
            0x00,
            0x00,
            SOCKS_PADDING,
            SOCKS_ATYP_IPV4,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        stream.write(&reply).await?;
        stream.flush().await?;

        tokio::io::copy_bidirectional(stream, &mut out).await?;

        Ok(())
    }
}