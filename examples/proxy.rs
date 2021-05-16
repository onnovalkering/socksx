use anyhow::Result;
use clap::{App, Arg};
use socksx::{self, Socks5Handler, Socks6Handler};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::new("Proxy")
        .arg(
            Arg::new("VERSION")
                .short('s')
                .long("socks")
                .about("The SOCKS version to use")
                .possible_values(&["5", "6"])
                .default_value("5"),
        )
        .get_matches();

    let listener = TcpListener::bind("0.0.0.0:1080").await?;
    match args.value_of("VERSION") {
        Some("5") => {
            let handler = Arc::new(Socks5Handler::new(None));

            loop {
                let (incoming, _) = listener.accept().await?;
                let handler = Arc::clone(&handler);

                tokio::spawn(process_v5(incoming, handler));
            }
        }
        Some("6") => {
            let handler = Arc::new(Socks6Handler::new(None));

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
