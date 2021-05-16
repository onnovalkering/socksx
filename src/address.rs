use crate::constants::*;
use anyhow::Result;
use std::convert::TryFrom;
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt};

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

#[derive(Clone, Debug)]
pub enum Address {
    Domainname { host: String, port: u16 },
    Ip(SocketAddr),
}

impl Address {
    ///
    ///
    ///
    pub fn new<S: Into<String>>(
        host: S,
        port: u16,
    ) -> Self {
        let host = host.into();

        if let Ok(host) = host.parse::<IpAddr>() {
            Address::Ip(SocketAddr::new(host, port))
        } else {
            Address::Domainname { host, port }
        }
    }

    ///
    ///
    ///
    pub fn as_socks_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        match self {
            Address::Ip(dst_addr) => {
                match dst_addr.ip() {
                    IpAddr::V4(host) => {
                        bytes.push(SOCKS_ATYP_IPV4);
                        bytes.extend(host.octets().iter());
                    }
                    IpAddr::V6(host) => {
                        bytes.push(SOCKS_ATYP_IPV6);
                        bytes.extend(host.octets().iter());
                    }
                }

                bytes.extend(dst_addr.port().to_be_bytes().iter())
            }
            Address::Domainname { host, port } => {
                bytes.push(SOCKS_ATYP_DOMAINNAME);

                let host = host.as_bytes();
                bytes.push(host.len() as u8);
                bytes.extend(host);

                bytes.extend(port.to_be_bytes().iter());
            }
        }

        bytes
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        match self {
            Address::Domainname { host, port } => format!("{}:{}", host, port),
            Address::Ip(socket_addr) => socket_addr.to_string(),
        }
    }
}

impl TryFrom<String> for Address {
    type Error = anyhow::Error;

    fn try_from(addr: String) -> Result<Self> {
        if let Some((host, port)) = addr.split_once(':') {
            Ok(Address::new(host, port.parse()?))
        } else {
            bail!("Address doesn't seperate host and port by ':'.")
        }
    }
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Self {
        Address::Ip(addr)
    }
}

impl From<([u8; 4], [u8; 2])> for Address {
    ///
    ///
    ///
    fn from(addr: ([u8; 4], [u8; 2])) -> Address {
        let host = IpAddr::from(addr.0);
        let port = ((addr.1[0] as u16) << 8) | addr.1[1] as u16;
        Address::Ip(SocketAddr::new(host, port))
    }
}

impl From<([u8; 16], [u8; 2])> for Address {
    ///
    ///
    ///
    fn from(addr: ([u8; 16], [u8; 2])) -> Address {
        let host = IpAddr::from(addr.0);
        let port = ((addr.1[0] as u16) << 8) | addr.1[1] as u16;
        Address::Ip(SocketAddr::new(host, port))
    }
}

impl From<(String, [u8; 2])> for Address {
    ///
    ///
    ///
    fn from(addr: (String, [u8; 2])) -> Address {
        let port = ((addr.1[0] as u16) << 8) | addr.1[1] as u16;
        Address::Domainname { host: addr.0, port }
    }
}
