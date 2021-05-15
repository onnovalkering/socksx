use anyhow::Result;
use std::{net::SocketAddr, os};
use tokio::net::{self, TcpStream};

///
///
///
#[cfg(unix)]
pub fn get_original_dst<S: os::unix::io::AsRawFd>(socket: &S) -> Result<SocketAddr> {
    use nix::sys::socket::{self, sockopt, InetAddr};

    let orignal_dst = socket::getsockopt(socket.as_raw_fd(), sockopt::OriginalDst)?;
    let orignal_dst = InetAddr::V4(orignal_dst).to_std();

    Ok(orignal_dst)
}

///
///
///
pub async fn resolve_addr<S: Into<String>>(addr: S) -> Result<SocketAddr> {
    let addr: String = addr.into();

    // First, try to parse address as socket address.
    if let Ok(addr) = addr.parse() {
        return Ok(addr);
    }

    // Otherwise, address is probably a domain name.
    let addresses: Vec<SocketAddr> = net::lookup_host(addr).await?.collect();
    match addresses[..] {
        [first, ..] => Ok(first),
        [] => bail!("Domain name didn't resolve to an IP address."),
    }
}

///
///
///
pub async fn try_read_initial_data(stream: &mut TcpStream) -> Result<Option<Vec<u8>>> {
    let mut initial_data = Vec::with_capacity(2usize.pow(14)); // 16KB is the max

    stream.readable().await?;
    match stream.try_read_buf(&mut initial_data) {
        Ok(0) => Ok(None),
        Ok(_) => Ok(Some(initial_data)),
        Err(e) => {
            return Err(e.into());
        }
    }
}
