use crate::Address;
use anyhow::Result;
use std::os;

#[cfg(unix)]
pub fn get_original_dst<S: os::unix::io::AsRawFd>(socket: &S) -> Result<Address> {
    use nix::sys::socket::{self, sockopt, InetAddr};

    let orignal_dst = socket::getsockopt(socket.as_raw_fd(), sockopt::OriginalDst)?;
    let orignal_dst = InetAddr::V4(orignal_dst).to_std();

    Ok(Address::Ip {
        host: orignal_dst.ip(),
        port: orignal_dst.port(),
    })
}

#[cfg(windows)]
pub fn get_original_dst<S: os::windows::io::AsRawSocket>(socket: &S) -> Result<Address> {
    unimplemented!();
}
