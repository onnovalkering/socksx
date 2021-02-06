use std::net::{IpAddr, SocketAddr};

mod socks5;

pub use socks5::Socks5Client;

pub mod constants {
    pub const SOCKS_VER_5: u8 = 0x05u8;
    pub const SOCKS_AUTH_VER: u8 = 0x01u8;
    pub const SOCKS_AUTH_NOT_REQUIRED: u8 = 0x00u8;
    pub const SOCKS_AUTH_USERNAME_PASSWORD: u8 = 0x02u8;
    pub const SOCKS_AUTH_SUCCESS: u8 = 0x00u8;
    pub const SOCKS_CMD_CONNECT: u8 = 0x01u8;
    pub const SOCKS_CMD_BIND: u8 = 0x02u8;
    pub const SOCKS_CMD_ASSOCIATE: u8 = 0x03u8;
    pub const SOCKS_RSV: u8 = 0x00u8;
    pub const SOCKS_ATYP_IPV4: u8 = 0x01u8;
    pub const SOCKS_ATYP_DOMAINNAME: u8 = 0x03u8;
    pub const SOCKS_ATYP_IPV6: u8 = 0x04u8;

    pub const SOCKS_REP_SUCCEEDED: u8 = 0x00u8;
}

pub enum Address {
    Domainname { host: String, port: u16 },
    Ip { host: IpAddr, port: u16 },
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
            Address::Ip { host, port }
        } else {
            Address::Domainname { host, port }
        }
    }

    ///
    ///
    ///
    pub fn as_socket_addr(&self) -> SocketAddr {
        match self.clone() {
            Address::Ip { host, port } => SocketAddr::new(*host, *port),
            _ => unimplemented!(),
        }
    }
}

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

        Credentials {
            username,
            password
        }
    }
}
