#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;

#[path = "./common/addresses.rs"]
pub mod addresses;
#[path = "./common/chain.rs"]
pub mod chain;
#[path = "./common/constants.rs"]
pub mod constants;
#[path = "./common/credentials.rs"]
pub mod credentials;
#[path = "./common/interface.rs"]
pub mod interface;
pub mod socks5;
pub mod socks6;
#[path = "./common/util.rs"]
pub mod util;

pub use addresses::{Address, ProxyAddress};
pub use credentials::Credentials;
pub use interface::SocksHandler;
pub use socks5::{Socks5Client, Socks5Handler};
pub use socks6::{Socks6Client, Socks6Handler};
pub use tokio::io::copy_bidirectional;
pub use util::{get_original_dst, resolve_addr, try_read_initial_data};
