use tokio::net::TcpStream;

pub const ALPN: &'static [u8] = b"iroh_tcp_transport";

pub async fn copy_between_iroh_and_tcp_streams(mut tcp_stream: TcpStream, (send_stream, recv_stream): (iroh::endpoint::SendStream, iroh::endpoint::RecvStream)) {
    let mut iroh_stream = tokio::io::join(recv_stream, send_stream);
    if let Err(e) = tokio::io::copy_bidirectional(&mut iroh_stream, &mut tcp_stream).await {
        eprintln!("{e}");
    }
}

pub fn prepare_iroh_endpoint_builder() -> iroh::endpoint::Builder {
    let mut builder = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal);
    if let Ok(iroh_relay_url) = std::env::var("IROH_RELAY_URL") {
        let url: iroh::RelayUrl = iroh_relay_url.parse().unwrap();
        builder = builder.relay_mode(iroh::RelayMode::Custom(url.into()));
    }
    builder
}
