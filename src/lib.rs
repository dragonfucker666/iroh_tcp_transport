use tokio::net::TcpStream;

pub const ALPN: &'static [u8] = b"iroh_tcp_transport";

pub fn reduce_err<T, E: std::fmt::Debug>(r: Result<T, E>, message: &'static str) -> Option<T> {
    r.inspect_err(|e| eprintln!("{}: {:?}", message, e)).ok()
}

pub async fn copy_bidir((send_stream, recv_stream): (iroh::endpoint::SendStream, iroh::endpoint::RecvStream), mut tcp_stream: TcpStream) -> Result<(u64, u64), std::io::Error> {
    let mut iroh_stream = tokio::io::join(recv_stream, send_stream);
    tokio::io::copy_bidirectional(&mut iroh_stream, &mut tcp_stream).await
}

pub fn build_endpoint() -> iroh::endpoint::Builder {
    let mut builder = iroh::Endpoint::builder(iroh::endpoint::presets::N0);
    if let Ok(iroh_relay_url) = std::env::var("IROH_RELAY_URL") {
        let url: iroh::RelayUrl = iroh_relay_url.parse().unwrap();
        builder = builder.relay_mode(iroh::RelayMode::Custom(url.into()));
    }
    builder
}
