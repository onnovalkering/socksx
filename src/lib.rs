#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;

use std::net::{IpAddr, SocketAddr};

mod socks5;
pub mod socks6;
mod util;

pub use socks5::{Socks5Client, Socks5Guard, Socks5Handler};
pub use socks6::{Socks6Client, Socks6Handler};
pub use util::{get_original_dst, resolve_addr, try_read_initial_data};

// Re-Export
pub use tokio::io::copy_bidirectional;

pub mod constants {
    pub const SOCKS_VER_5: u8 = 0x05u8;
    pub const SOCKS_VER_6: u8 = 0x06u8;

    pub const SOCKS_AUTH_VER: u8 = 0x01u8;
    pub const SOCKS_AUTH_NOT_REQUIRED: u8 = 0x00u8;
    pub const SOCKS_AUTH_USERNAME_PASSWORD: u8 = 0x02u8;
    pub const SOCKS_AUTH_NO_ACCEPTABLE_METHODS: u8 = 0xFFu8;
    pub const SOCKS_AUTH_SUCCESS: u8 = 0x00u8;
    pub const SOCKS_AUTH_FAILED: u8 = 0x01u8;

    pub const SOCKS_OKIND_STACK: u16 = 0x01u16;
    pub const SOCKS_OKIND_AUTH_METH_ADV: u16 = 0x02u16;
    pub const SOCKS_OKIND_AUTH_METH_SEL: u16 = 0x03u16;
    pub const SOCKS_OKIND_AUTH_DATA: u16 = 0x04u16;

    pub const SOCKS_CMD_NOOP: u8 = 0x00u8;
    pub const SOCKS_CMD_CONNECT: u8 = 0x01u8;
    pub const SOCKS_CMD_BIND: u8 = 0x02u8;
    pub const SOCKS_CMD_UDP_ASSOCIATE: u8 = 0x03u8;

    pub const SOCKS_PADDING: u8 = 0x00u8;
    pub const SOCKS_RSV: u8 = 0x00u8;

    pub const SOCKS_ATYP_IPV4: u8 = 0x01u8;
    pub const SOCKS_ATYP_DOMAINNAME: u8 = 0x03u8;
    pub const SOCKS_ATYP_IPV6: u8 = 0x04u8;

    pub const SOCKS_REP_SUCCEEDED: u8 = 0x00u8;
}

#[derive(Clone, Debug)]
pub enum Address {
    Domainname { host: String, port: u16 },
    Ip(SocketAddr),
}

impl ToString for Address {
    fn to_string(&self) -> String {
        match self {
            Address::Domainname { host, port } => format!("{}{}", host, port),
            Address::Ip(socket_addr) => socket_addr.to_string(),
        }
    }
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
        use constants::*;
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

impl From<SocketAddr> for Address {
    ///
    ///
    ///
    fn from(addr: SocketAddr) -> Address {
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

#[derive(Clone)]
pub struct Credentials {
    username: Vec<u8>,
    password: Vec<u8>,
}

impl Credentials {
    ///
    ///
    ///
    pub fn new<S: Into<Vec<u8>>>(
        username: S,
        password: S,
    ) -> Self {
        let username = username.into();
        let password = password.into();

        Credentials { username, password }
    }

    ///
    ///
    ///
    pub fn as_socks_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        // Append username
        bytes.push(self.username.len() as u8);
        bytes.extend(self.username.clone());

        // Append password
        bytes.push(self.password.len() as u8);
        bytes.extend(self.password.clone());

        bytes
    }
}
