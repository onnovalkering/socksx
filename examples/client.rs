use anyhow::Result;
use clap::{App, Arg};
use socksx::{Socks5Client, Socks6Client};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::new("Client")
        .arg(
            Arg::with_name("VERSION")
                .short("s")
                .long("socks")
                .help("The SOCKS version to use")
                .possible_values(&["5", "6"])
                .default_value("5"),
        )
        .arg(
            Arg::with_name("PROXY_HOST")
                .help("The IP/hostname of the proxy")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::with_name("PROXY_PORT")
                .help("The port of the proxy server")
                .default_value("1080"),
        )
        .arg(
            Arg::with_name("DEST_HOST")
                .help("The IP/hostname of the destination")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::with_name("DEST_PORT")
                .help("The port of the destination server")
                .default_value("12345"),
        )
        .get_matches();

    let proxy_host = args.value_of("PROXY_HOST").unwrap();
    let proxy_port = args.value_of("PROXY_PORT").unwrap();
    let proxy_addr = format!("{}:{}", proxy_host, proxy_port);

    let dest_host = args.value_of("DEST_HOST").unwrap();
    let dest_port = args.value_of("DEST_PORT").unwrap();
    let dest_addr = format!("{}:{}", dest_host, dest_port);

    match args.value_of("VERSION") {
        Some("5") => connect_v5(proxy_addr, dest_addr).await,
        Some("6") => connect_v6(proxy_addr, dest_addr).await,
        Some(version) => panic!("Unsupported version: {}", version),
        None => unreachable!(),
    }
}

///
///
///
async fn connect_v5(
    proxy_addr: String,
    dest_addr: String,
) -> Result<()> {
    let client = Socks5Client::new(proxy_addr, None).await?;
    let (mut outgoing, _) = client.connect(dest_addr).await?;

    outgoing.write(String::from("Hello, world!\n").as_bytes()).await?;

    Ok(())
}

///
///
///
async fn connect_v6(
    proxy_addr: String,
    dest_addr: String,
) -> Result<()> {
    let client = Socks6Client::new(proxy_addr, None).await?;
    let (mut outgoing, _) = client.connect(dest_addr, None, None).await?;

    outgoing.write(String::from("Hello, world!\n").as_bytes()).await?;

    Ok(())
}
