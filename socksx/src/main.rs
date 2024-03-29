#[macro_use]
extern crate human_panic;

use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use itertools::Itertools;
use log::LevelFilter;
use socksx::{self, Socks5Handler, Socks6Handler, SocksHandler};
use std::{convert::TryInto, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::Instant;

type Handler = Arc<dyn SocksHandler + Sync + Send>;

#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    /// Entry in the proxy chain, the order is preserved
    #[clap(short, long, env = "CHAIN", multiple_occurrences = true)]
    chain: Vec<String>,

    /// Prints debug information
    #[clap(short, long, env = "DEBUG", takes_value = false)]
    debug: bool,

    /// Host (IP) for the SOCKS server
    #[clap(short, long, env = "HOST", default_value = "0.0.0.0")]
    host: String,

    /// Concurrent connections limit (0=unlimted)
    #[clap(short, long, env = "LIMIT", default_value = "256")]
    limit: usize,

    /// Port for the SOCKS server
    #[clap(short, long, env = "PORT", default_value = "1080")]
    port: u16,

    /// SOCKS version
    #[clap(short, long, env = "SOCKS", default_value = "6", possible_values = &["5", "6"])]
    socks: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let args = Args::parse();

    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if args.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();

        setup_panic!(Metadata {
            name: "SOCKSX".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
        });
    }

    // TODO: validate host

    //
    //
    let chain = args.chain.iter().cloned().map(|c| c.try_into()).try_collect()?;

    //
    //
    let semaphore = if args.limit > 0 {
        Some(Arc::new(Semaphore::new(args.limit)))
    } else {
        None
    };

    //
    //
    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port)).await?;
    let handler: Handler = match args.socks {
        5 => Arc::new(Socks5Handler::new(chain)),
        6 => Arc::new(Socks6Handler::new(chain)),
        _ => unreachable!(),
    };

    loop {
        let (incoming, _) = listener.accept().await?;

        let handler = Arc::clone(&handler);
        let semaphore = semaphore.clone();

        tokio::spawn(process(incoming, handler, semaphore));
    }
}

///
///
///
async fn process(
    incoming: TcpStream,
    handler: Handler,
    semaphore: Option<Arc<Semaphore>>,
) -> Result<()> {
    let mut incoming = incoming;
    let start_time = Instant::now();

    if let Some(semaphore) = semaphore {
        let permit = semaphore.try_acquire();
        if permit.is_ok() {
            handler.accept_request(&mut incoming).await?;
        } else {
            handler.refuse_request(&mut incoming).await?;
        }
    } else {
        handler.accept_request(&mut incoming).await?;
    }

    println!("{}ms", Instant::now().saturating_duration_since(start_time).as_millis());

    Ok(())
}
