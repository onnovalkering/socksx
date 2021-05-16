use crate::socks6::options::{
    AuthMethodAdvertisementOption, AuthMethodSelectionOption, MetadataOption, SocksOption, UnrecognizedOption,
};
use crate::{constants::*, Address};
use anyhow::{ensure, Result};
use num_traits::FromPrimitive;
use std::{collections::HashMap, net::IpAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

mod options;
mod s6_client;
mod s6_handler;

pub use s6_client::Socks6Client;
pub use s6_handler::Socks6Handler;

#[repr(u8)]
#[derive(Clone, Debug, FromPrimitive)]
pub enum AuthMethod {
    NoAuthentication = 0x00,
    Gssapi = 0x01,
    UsernamePassword = 0x02,
    NoAcceptableMethods = 0xFF,
}

#[derive(Clone, Debug)]
pub struct Socks6Request {
    pub command: SocksCommand,
    pub destination: Address,
    pub initial_data_length: u16,
    pub options: Vec<SocksOption>,
    pub metadata: HashMap<u16, String>,
}

impl Socks6Request {
    ///
    ///
    ///
    pub fn new(
        command: u8,
        destination: Address,
        initial_data_length: u16,
        options: Vec<SocksOption>,
        metadata: Option<HashMap<u16, String>>,
    ) -> Self {
        Socks6Request {
            command: SocksCommand::from_u8(command).unwrap(),
            destination,
            initial_data_length,
            options,
            metadata: metadata.unwrap_or_default(),
        }
    }

    ///
    ///
    ///
    pub fn into_socks_bytes(self) -> Vec<u8> {
        let mut data = vec![SOCKS_VER_6, SOCKS_CMD_CONNECT];
        data.extend(self.destination.as_socks_bytes());
        data.push(SOCKS_PADDING);

        let options_bytes: Vec<_> = self.options.into_iter().flat_map(|o| o.as_socks_bytes()).collect();
        let options_bytes_length = (options_bytes.len() as u16).to_be_bytes();

        data.extend(options_bytes_length.iter());
        data.extend(options_bytes.iter());

        data
    }
}

#[repr(u8)]
#[derive(Clone, Debug, FromPrimitive, PartialEq)]
pub enum SocksCommand {
    NoOp = 0x00,
    Connect = 0x01,
    Bind = 0x02,
    UdpAssociate = 0x03,
}

///
///
///
pub async fn read_request<S>(stream: &mut S) -> Result<Socks6Request>
where
    S: AsyncRead + Unpin,
{
    // Read SOCKS version and command type.
    let mut request = [0; 2];
    stream.read_exact(&mut request).await?;

    let [version, command] = request;

    // Validate the request.
    ensure!(version == SOCKS_VER_6, "Version mismatch!");
    ensure!(command == SOCKS_CMD_CONNECT, "Only COMMAND is supported!");

    let destination = read_address(stream).await?;

    let mut padding = [0; 1];
    stream.read_exact(&mut padding).await?;

    let options = read_options(stream).await?;

    let mut initial_data_length = 0;
    let mut metadata = HashMap::new();
    for option in &options {
        match option {
            SocksOption::AuthMethodAdvertisement(advertisement) => {
                // Make note of initial data length for convience.
                initial_data_length = advertisement.initial_data_length;
            }
            SocksOption::Metadata(key_value) => {
                metadata.insert(key_value.key, key_value.value.clone());
            }
            _ => {}
        }

        if let SocksOption::Metadata(key_value) = option {
            metadata.insert(key_value.key, key_value.value.clone());
        }
    }

    Ok(Socks6Request::new(
        command,
        destination,
        initial_data_length,
        options,
        Some(metadata),
    ))
}

///
///
///
pub async fn read_address<S>(stream: &mut S) -> Result<Address>
where
    S: AsyncRead + Unpin,
{
    // Read address type.
    let mut address_type = [0; 1];
    stream.read_exact(&mut address_type).await?;

    let dst_addr = match address_type[0] {
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

            String::from_utf8_lossy(&dst_addr[..]).to_string()
        }
        _ => unreachable!(),
    };

    // Read destination port.
    let mut dst_port = [0; 2];
    stream.read_exact(&mut dst_port).await?;

    let dst_port = ((dst_port[0] as u16) << 8) | dst_port[1] as u16;

    Ok(Address::new(dst_addr, dst_port))
}

///
///
///
pub async fn read_options<S>(stream: &mut S) -> Result<Vec<SocksOption>>
where
    S: AsyncRead + Unpin,
{
    let mut options = Vec::new();

    let mut options_length = [0; 2];
    stream.read_exact(&mut options_length).await?;

    let options_length = ((options_length[0] as u16) << 8) | options_length[1] as u16;
    let mut options_bytes_read = 0;

    while options_bytes_read < options_length {
        let mut buffer = [0; 4];
        stream.read_exact(&mut buffer).await?;

        let [kind_0, kind_1, length_0, length_1] = buffer;
        let kind = ((kind_0 as u16) << 8) | kind_1 as u16;
        let length = ((length_0 as u16) << 8) | length_1 as u16;

        // Read remaining bytes of this option.
        let mut options_data = vec![0; (length - 4) as usize];
        stream.read_exact(&mut options_data).await?;

        let option = match kind {
            0x0002 => AuthMethodAdvertisementOption::from_socks_bytes(options_data)?,
            0x0003 => AuthMethodSelectionOption::from_socks_bytes(options_data)?,
            0xFDE8 => MetadataOption::from_socks_bytes(options_data)?,
            _ => UnrecognizedOption::new(kind, options_data.to_vec()).wrap(),
        };

        options.push(option);
        options_bytes_read += length;
    }

    Ok(options)
}

pub async fn read_no_authentication<S>(stream: &mut S) -> Result<Vec<SocksOption>>
where
    S: AsyncRead + Unpin,
{
    // Read auth reply
    let mut reply = [0; 1];
    stream.read_exact(&mut reply).await?;

    let socks_version = reply[0];
    ensure!(
        socks_version == SOCKS_VER_6,
        "Proxy uses a different SOCKS version: {}",
        socks_version
    );

    let mut reply = [0; 1];
    stream.read_exact(&mut reply).await?;

    let status = reply[0];
    ensure!(
        status == SOCKS_AUTH_SUCCESS,
        "Authentication with proxy failed: {}",
        status
    );

    let options = read_options(stream).await?;
        
    Ok(options)
}

pub async fn write_no_authentication<S>(stream: &mut S) -> Result<()>
where
    S: AsyncWrite + Unpin,
{
    // Write auth reply
    let auth_reply = [SOCKS_VER_6, SOCKS_AUTH_SUCCESS, 0x00u8, 0x00u8];
    stream.write(&auth_reply).await?;

    Ok(())
}

pub async fn write_initial_data<S>(
    _stream: &mut S,
    _request: &Socks6Request,
) -> Result<()>
where
    S: AsyncWrite + Unpin,
{
    // Not yet implemented.
    Ok(())
}

#[repr(u8)]
#[derive(Clone, Debug, FromPrimitive, PartialEq)]
pub enum SocksReply {
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
    reply: SocksReply,
) -> Result<()>
where
    S: AsyncWrite + Unpin,
{
    let reply = [
        SOCKS_VER_6,
        reply as u8,
        SOCKS_PADDING,
        SOCKS_ATYP_IPV4,
        0x00,
        0x00,
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
pub async fn read_reply<S>(
    stream: &mut S
) -> Result<(Address, Vec<SocksOption>)>
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

    let binding = read_address(stream).await?;
    let options = read_options(stream).await?;

    Ok((binding, options))
}