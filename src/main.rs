#[macro_use]
extern crate human_panic;

use anyhow::Result;
use clap::{App, Arg};
use dotenv::dotenv;
use log::LevelFilter;
use socksx::{self, ProxyAddress, Socks5Handler, Socks6Handler};
use tokio::sync::Semaphore;
use std::{convert::TryInto, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let args = App::new("socksx")
        .version("0.2.0")
        .about("https://github.com/onnovalkering/socksx")
        .arg(
            Arg::new("DEBUG")
                .short('d')
                .long("debug")
                .about("Prints debug information verbosely")
        )
        .arg(
            Arg::new("VERSION")
                .short('s')
                .long("socks")
                .about("SOCKS version to use")
                .possible_values(&["5", "6"])
                .default_value("6"),
        )
        .arg(
            Arg::new("CONN_LIMIT")
                .long("connections-limit")
                .about("Concurrent connections limit (0=unlimted)")
                .default_value("0"),
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

    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if args.is_present("DEBUG") {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();

        setup_panic!(Metadata {
            name: "socksx".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
        });
    }

    let port = args.value_of("PORT").unwrap();
    let chain: Option<Vec<ProxyAddress>> = args
        .values_of("CHAIN")
        .map(|c| c.into_iter().map(|c| c.to_string().try_into().unwrap()).collect());

    let conn_limit = args.value_of("CONN_LIMIT").unwrap();
    let semaphore = if conn_limit != "0" {
        Some(Arc::new(Semaphore::new(conn_limit.parse()?)))
    } else {
        None
    };

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    match args.value_of("VERSION") {
        Some("5") => {
            let handler = Arc::new(Socks5Handler::new(chain));

            loop {
                let (incoming, _) = listener.accept().await?;
                
                let handler = Arc::clone(&handler);
                let semaphore = semaphore.clone();

                tokio::spawn(process_v5(incoming, handler, semaphore));
            }
        }
        Some("6") => {
            let handler = Arc::new(Socks6Handler::new(chain));

            loop {
                let (incoming, _) = listener.accept().await?;
                let handler = Arc::clone(&handler);
                let semaphore = semaphore.clone();

                tokio::spawn(process_v6(incoming, handler, semaphore));
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
    semaphore: Option<Arc<Semaphore>>,
) -> Result<()> {
    let mut incoming = incoming;
    let start_time = Instant::now();

    if let Some(semaphore) = semaphore {
        let permit = semaphore.try_acquire();
        if permit.is_ok() {
            handler.handle_request(&mut incoming).await?;
        } else {
            handler.refuse_request(&mut incoming).await?;
        }
    } else {
        handler.handle_request(&mut incoming).await?;
    }

    println!("{}ms", Instant::now().saturating_duration_since(start_time).as_millis());

    Ok(())
}

///
///
///
async fn process_v6(
    incoming: TcpStream,
    handler: Arc<Socks6Handler>,
    semaphore: Option<Arc<Semaphore>>,
) -> Result<()> {
    let mut incoming = incoming;
    let start_time = Instant::now();

    if let Some(semaphore) = semaphore {
        let permit = semaphore.try_acquire();
        if permit.is_ok() {
            handler.handle_request(&mut incoming).await?;
        } else {
            handler.refuse_request(&mut incoming).await?;
        }
    } else {
        handler.handle_request(&mut incoming).await?;
    }

    println!("{}ms", Instant::now().saturating_duration_since(start_time).as_millis());

    Ok(())
}
