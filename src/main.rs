use anyhow::Result;
use clap::{App, Arg};
use socksx::{self, ProxyAddress, Socks5Handler, Socks6Handler};
use std::{convert::TryInto, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::new("socksx")
        .version("0.2.0")
        .about("https://github.com/onnovalkering/socksx")
        .arg(
            Arg::new("VERSION")
                .short('s')
                .long("socks")
                .about("SOCKS version to use")
                .possible_values(&["5", "6"])
                .default_value("6"),
        )
        .arg(
            Arg::new("PORT")
                .short('p')
                .long("port")
                .about("Port to use")
                .default_value("1080"),
        )
        .arg(
            Arg::new("CHAIN")
                .short('c')
                .long("chain")
                .about("Entry in the proxy chain, the order is preserved")
                .multiple(true)
                .takes_value(true),
        )
        .get_matches();

    let port = args.value_of("PORT").unwrap();
    let chain: Option<Vec<ProxyAddress>> = args
        .values_of("CHAIN")
        .map(|c| c.into_iter().map(|c| c.to_string().try_into().unwrap()).collect());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    match args.value_of("VERSION") {
        Some("5") => {
            let handler = Arc::new(Socks5Handler::new(chain));

            loop {
                let (incoming, _) = listener.accept().await?;
                let handler = Arc::clone(&handler);

                tokio::spawn(process_v5(incoming, handler));
            }
        }
        Some("6") => {
            let handler = Arc::new(Socks6Handler::new(chain));

            loop {
                let (incoming, _) = listener.accept().await?;
                let handler = Arc::clone(&handler);

                tokio::spawn(process_v6(incoming, handler));
            }
        }
        Some(version) => panic!("Unsupported version: {}", version),
        None => unreachable!(),
    }
}

///
///
///
async fn process_v5(
    incoming: TcpStream,
    handler: Arc<Socks5Handler>,
) -> Result<()> {
    let mut incoming = incoming;
    let start_time = Instant::now();

    handler.handle_request(&mut incoming).await?;

    println!("{}ms", Instant::now().saturating_duration_since(start_time).as_millis());

    Ok(())
}

///
///
///
async fn process_v6(
    incoming: TcpStream,
    handler: Arc<Socks6Handler>,
) -> Result<()> {
    let mut incoming = incoming;
    let start_time = Instant::now();

    handler.handle_request(&mut incoming).await?;

    println!("{}ms", Instant::now().saturating_duration_since(start_time).as_millis());

    Ok(())
}
