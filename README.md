# SOCKS toolkit

A SOCKS toolkit for Rust, based on Tokio. SOCKS5 ([rfc1928](https://tools.ietf.org/html/rfc1928)) and SOCKS6 ([draft-11](https://tools.ietf.org/html/draft-olteanu-intarea-socks-6-11))  are supported.

## Clients
All client commands are supported: `CONNECT`, `BIND`, and `UDP ASSOCIATE`.

```rust
use socksx::{self, Address, Credentials, Socks6Client};

let proxy_server = Address::new("127.0.0.1", 1080);
let credentials = Credentials::new("myuser", "mypass");
let client = Socks6Client::new(proxy_server, Some(credentials));

let destination = Address::new("github.com", 80);
let socket = client.connect(destination).await?;
```

