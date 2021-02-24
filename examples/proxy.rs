use anyhow::Result;
use clap::{App, Arg};
use socksx::{self, Socks5Guard, Socks5Handler, Socks6Handler};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::new("Proxy")
        .arg(
            Arg::with_name("VERSION")
                .short("s")
                .long("socks")
                .help("The SOCKS version to use")
                .default_value("5"),
        )
        .get_matches();

    let listener = TcpListener::bind("0.0.0.0:1080").await?;

    match args.value_of("VERSION") {
        Some("5") => {
            let guard = Socks5Guard::new(None);
            let handler = Socks5Handler::new();

            loop {
                let (socket, _) = listener.accept().await?;
                tokio::spawn(process_v5(socket, guard.clone(), handler.clone()));
            }
        }
        Some("6") => {
            let handler = Arc::new(Socks6Handler::new());

            loop {
                let (mut incoming, _) = listener.accept().await?;
                let handler = Arc::clone(&handler);

                tokio::spawn(async move { handler.handle_request(&mut incoming).await });
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
    guard: Socks5Guard,
    handler: Socks5Handler,
) -> Result<()> {
    let mut incoming = incoming;
    let start_time = Instant::now();

    guard.authenticate(&mut incoming).await?;
    handler.handle_request(&mut incoming).await?;

    println!("{}ms", Instant::now().saturating_duration_since(start_time).as_millis());

    Ok(())
}
