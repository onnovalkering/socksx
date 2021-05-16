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