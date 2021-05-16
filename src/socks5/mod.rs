use crate::address::{self, Address};
use crate::constants::*;
use anyhow::Result;
use num_traits::FromPrimitive;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

mod s5_client;
mod s5_handler;

pub use s5_client::Socks5Client;
pub use s5_handler::Socks5Handler;

#[repr(u8)]
#[derive(Clone, Debug, FromPrimitive, PartialEq)]
pub enum Socks5Command {
    Connect = 0x01,
    Bind = 0x02,
    UdpAssociate = 0x03,
}

#[derive(Clone, Debug)]
pub struct Socks5Request {
    pub command: Socks5Command,
    pub destination: Address,
}

impl Socks5Request {
    ///
    ///
    ///
    pub fn new(
        command: u8,
        destination: Address,
    ) -> Self {
        Socks5Request {
            command: Socks5Command::from_u8(command).unwrap(),
            destination,
        }
    }

    ///
    ///
    ///
    pub fn into_socks_bytes(self) -> Vec<u8> {
        let mut data = vec![SOCKS_VER_5, SOCKS_CMD_CONNECT, SOCKS_RSV];
        data.extend(self.destination.as_socks_bytes());

        data
    }
}

#[repr(u8)]
#[derive(Clone, Debug, FromPrimitive, PartialEq)]
pub enum Socks5Reply {
    Success = 0x00,
    GeneralFailure = 0x01,
    ConnectionNotAllowed = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TTLExpired = 0x06,
    CommandNotSupported = 0x07,
    AddressTypeNotSupported = 0x08,
    ConnectionAttemptTimeOut = 0x09,
}

///
///
///
pub async fn write_reply<S>(
    stream: &mut S,
    reply: Socks5Reply,
) -> Result<()>
where
    S: AsyncWrite + Unpin,
{
    let reply = [
        SOCKS_VER_5,
        reply as u8,
        SOCKS_RSV,
        SOCKS_ATYP_IPV4,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
    ];

    stream.write(&reply).await?;

    Ok(())
}

///
///
///
pub async fn read_reply<S>(stream: &mut S) -> Result<Address>
where
    S: AsyncRead + Unpin,
{
    let mut operation_reply = [0; 3];
    stream.read_exact(&mut operation_reply).await?;

    let reply_code = operation_reply[1];
    ensure!(
        reply_code == SOCKS_REP_SUCCEEDED,
        "CONNECT operation failed: {}",
        reply_code
    );

    let binding = address::read_address(stream).await?;

    Ok(binding)
}
