use anyhow::Result;
use clap::{App, Arg};
use socksx::{self, Socks5Client};
use tokio::net::{TcpListener, TcpStream};

// iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("Redirector")
        .arg(
            Arg::with_name("PROXY")
                .help("The IP or hostname of the proxy")
                .required(true)
                .index(1),
        )
        .get_matches();

    // Setup SOCKS client
    let proxy_host = matches.value_of("PROXY").unwrap_or("127.0.0.1");
    let client = Socks5Client::new(format!("{}:1080", proxy_host), None).await?;

    // Start redirecting
    let listener = TcpListener::bind("127.0.0.1:42000").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(redirect(stream, client.clone()));
    }
}

/// Redirect an incoming TCP stream through the
/// proxy. The original destination of the stream
/// is preserved, by iptables, as an socket option.
async fn redirect(
    incoming: TcpStream,
    client: Socks5Client,
) -> Result<()> {
    let mut incoming = incoming;

    let dst_addr = socksx::get_original_dst(&incoming)?;
    let (mut outgoing, _) = client.connect(dst_addr).await?;

    socksx::bidirectional_copy(&mut incoming, &mut outgoing).await?;

    Ok(())
}
