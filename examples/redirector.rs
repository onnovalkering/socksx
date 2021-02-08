use anyhow::Result;
use socksx::{self, Socks5Client};
use tokio::net::{TcpListener, TcpStream};

// iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 46666

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:46666").await?;
    let client = Socks5Client::new("127.0.0.1:1080", None);

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(redirect(socket, client.clone()));
    }
}

/// Redirect an incoming TCP socket through the
/// proxy. The original destination of the socket
/// is preserved, by iptables, as an socket option.
async fn redirect(
    incoming: TcpStream,
    client: Socks5Client,
) -> Result<()> {
    let mut incoming = incoming;

    let dst_addr = socksx::get_original_dst(&incoming)?;
    let mut outgoing = client.connect(dst_addr).await?;

    socksx::bidirectional_copy(&mut incoming, &mut outgoing).await?;

    Ok(())
}
