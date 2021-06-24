use crate::{constants::*, Credentials};
use anyhow::Result;
use std::convert::{TryFrom, TryInto};
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt};
use url::Url;

#[derive(Clone, Debug)]
pub struct ProxyAddress {
    pub socks_version: u8,
    pub host: String,
    pub port: u16,
    pub credentials: Option<Credentials>,
}

impl ProxyAddress {
    pub fn new(
        socks_version: u8,
        host: String,
        port: u16,
        credentials: Option<Credentials>,
    ) -> Self {
        Self {
            socks_version,
            host,
            port,
            credentials,
        }
    }

    pub fn root() -> Self {
        ProxyAddress::new(6, String::from("root"), 1080, None)
    }
}

impl ToString for ProxyAddress {
    fn to_string(&self) -> String {
        format!("socks{}://{}:{}", self.socks_version, self.host, self.port)
    }
}

impl TryFrom<String> for ProxyAddress {
    type Error = anyhow::Error;

    fn try_from(proxy_addr: String) -> Result<Self> {
        let proxy_addr = Url::parse(&proxy_addr)?;

        ensure!(
            proxy_addr.host().is_some(),
            "Missing explicit IP/host in proxy address."
        );
        ensure!(proxy_addr.port().is_some(), "Missing explicit port in proxy address.");

        let socks_version = match proxy_addr.scheme() {
            "socks5" => SOCKS_VER_5,
            "socks6" => SOCKS_VER_6,
            scheme => bail!("Unrecognized SOCKS scheme: {}", scheme),
        };

        let username = proxy_addr.username();
        let credentials = if username.is_empty() {
            None
        } else {
            let password = proxy_addr.password().unwrap_or_default();
            Some(Credentials::new(username, password))
        };

        Ok(Self::new(
            socks_version,
            proxy_addr.host().map(|h| h.to_string()).unwrap(),
            proxy_addr.port().unwrap(),
            credentials,
        ))
    }
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

impl TryFrom<SocketAddr> for Address {
    type Error = anyhow::Error;

    fn try_from(addr: SocketAddr) -> Result<Self, Self::Error> {
        addr.to_string().try_into()
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

impl TryFrom<&ProxyAddress> for Address {
    type Error = anyhow::Error;

    fn try_from(addr: &ProxyAddress) -> Result<Self> {
        format!("{}:{}", addr.host, addr.port).try_into()
    }
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
