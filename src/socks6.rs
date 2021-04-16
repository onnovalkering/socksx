use crate::{constants::*, Address, Credentials};
use anyhow::{ensure, Result};
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use num_traits::FromPrimitive;

#[derive(Clone, Debug)]
pub struct SocksOption {
    kind: u16,
    data: Vec<u8>,
}

impl SocksOption {
    ///
    ///
    ///
    pub fn new(
        kind: u16,
        data: Vec<u8>,
    ) -> Self {
        SocksOption { kind, data }
    }

    pub fn as_socks_bytes(&self) -> Vec<u8> {
        // The total length of the option is the combined number of bytes of
        // the kind, length, and data fields, plus the number of padding bytes.
        let option_length = self.data.len() + 2 + 2;
        let padding_bytes = vec![0; 4 - (option_length % 4)];
        let total_length: u16 = (option_length + padding_bytes.len()) as u16;

        let mut bytes = vec![];
        bytes.extend(self.kind.to_be_bytes().iter());
        bytes.extend(total_length.to_be_bytes().iter());
        bytes.extend(self.data.iter());
        bytes.extend(padding_bytes.iter());

        bytes
    }
}

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

        let auth_meth_adv_option = SocksOption::new(SOCKS_OKIND_AUTH_METH_ADV, auth_option_data);

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

#[derive(Clone)]
pub struct Socks6Handler {}

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

        let mut reply = [
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

        stream.write(&mut reply).await?;
        stream.flush().await?;

        tokio::io::copy_bidirectional(stream, &mut out).await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Socks6Request {
    pub command: SocksCommand,
    pub destination: Address,
    pub options: Vec<SocksOption>,
}

impl Socks6Request {
    ///
    ///
    ///
    pub fn new(
        command: u8,
        destination: Address,
        options: Vec<SocksOption>
    ) -> Self {
        Socks6Request {
            command: SocksCommand::from_u8(command).unwrap(),
            destination,
            options,
        }
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
pub async fn read_request<S>(   stream: &mut S) -> Result<Socks6Request>
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

    Ok(Socks6Request::new(command, destination, options))
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

    // Read destination port and padding (ignored).
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
    let mut options_length = [0; 2];
    stream.read_exact(&mut options_length).await?;

    let options_length = ((options_length[0] as u16) << 8) | options_length[1] as u16;
    let mut options_bytes_read = 0;

    dbg!(&options_length);

    let mut options = Vec::new();
    while options_bytes_read < options_length {
        let mut buffer = [0; 4];
        stream.read_exact(&mut buffer).await?;
        options_bytes_read = options_bytes_read + 4;

        dbg!(&buffer);
        
        let [kind_0, kind_1, length_0, length_1] = buffer;
        let kind = ((kind_0 as u16) << 8) | kind_1 as u16;
        let length = ((length_0 as u16) << 8) | length_1 as u16;

        // Read remaining bytes of this option.
        let mut data = vec![0; (length - 4)  as usize];
        stream.read_exact(&mut data).await?;
        options_bytes_read = options_bytes_read + length;

        dbg!(&data);

        options.push(SocksOption::new(kind, data.to_vec()));
    }

    Ok(options)
}


pub async fn no_authentication<S>(stream: &mut S) -> Result<()> 
where
    S: AsyncRead + AsyncWrite + Unpin,
{
        // Write auth reply
        let auth_reply = [SOCKS_VER_6, SOCKS_AUTH_SUCCESS, 0x00u8, 0x00u8];
        stream.write(&auth_reply).await?;

        Ok(())
}

pub async fn write_initial_data<S>(_stream: &mut S, _request: &Socks6Request) -> Result<()> 
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

pub async fn write_reply<S>(stream: &mut S, reply: SocksReply) -> Result<()> 
where
    S: AsyncWrite + Unpin,
{
    let mut reply = [
        SOCKS_VER_6,
        reply as u8,
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

    stream.write(&mut reply).await?;

    Ok(())
}