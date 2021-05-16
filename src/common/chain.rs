use crate::addresses::{Address, ProxyAddress};
use crate::constants::*;
use crate::{Socks5Client, Socks6Client};
use anyhow::Result;
use tokio::net::TcpStream;

///
///
///
pub async fn setup(
    chain: &[ProxyAddress],
    destination: Address,
) -> Result<TcpStream> {
    if chain.len() == 1 {
        single_hop(chain.first().unwrap(), destination).await
    } else {
        multi_hop(chain, destination).await
    }
}

///
///
///
async fn single_hop(
    proxy: &ProxyAddress,
    destination: Address,
) -> Result<TcpStream> {
    let proxy_addr = format!("{}:{}", proxy.host, proxy.port);
    let credentials = proxy.credentials.clone();
    let destination = destination.to_string();

    match proxy.socks_version {
        SOCKS_VER_5 => {
            let client = Socks5Client::new(proxy_addr, credentials).await?;
            let (outgoing, _) = client.connect(destination).await?;

            Ok(outgoing)
        }
        SOCKS_VER_6 => {
            let client = Socks6Client::new(proxy_addr, credentials).await?;
            let (outgoing, _) = client.connect(destination, None, None).await?;

            Ok(outgoing)
        }
        _ => unreachable!(),
    }
}

///
///
///
async fn multi_hop(
    _chain: &[ProxyAddress],
    _destination: Address,
) -> Result<TcpStream> {
    todo!()
}
