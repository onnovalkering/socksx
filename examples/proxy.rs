use anyhow::Result;
use socksx::{self, Socks5Guard, Socks5Handler};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:1080").await?;
    let guard = Socks5Guard::new(None);
    let handler = Socks5Handler::new();

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(process(socket, guard.clone(), handler.clone()));
    }
}

///
///
///
async fn process(
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
