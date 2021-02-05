# SOCKS toolkit

A SOCKS toolkit for Rust, based on Tokio.

# Capabilities

## Clients
The toolkit includes clients for SOCKS5 and SOCKS6.

```rust
let proxy_addr = Address::new("localhost", 1080);
let credentials = Credentials::new("username", "password");

let client = Socks5Client::new(proxy_addr, Some(credentials));

let dst_addr = Address::new("google.com", 80);
let socket = client.connect(dst_addr).await?;
```

```rust

```