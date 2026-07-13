use iroh_tcp_transport::reduce_err;

#[tokio::main]
async fn main() {
    // Collecting params
    let listen_addr = std::env::var("LISTEN_ADDR").expect("LISTEN_ADDR env var should be provided");
    let iroh_endpoint_id: iroh::EndpointId = std::env::var("IROH_PUBKEY").expect("IROH_PUBKEY env var should be provided").parse().unwrap();
    let (builder, relay_url) = iroh_tcp_transport::build_endpoint();
    let mut iroh_endpoint_addr = iroh::EndpointAddr::new(iroh_endpoint_id);
    if let Some(relay_url) = relay_url {
        iroh_endpoint_addr = iroh_endpoint_addr.with_relay_url(relay_url);
    }
    let endpoint = builder.bind().await.expect("couldn't bind own iroh endpoint");
    // Building tools
    let get_send_stream = {
        let connect_iroh = async move || reduce_err(endpoint.connect(iroh_endpoint_addr.clone(), iroh_tcp_transport::ALPN).await, "couldn't send iroh conn").map(move |conn| async move || conn.open_bi().await);
        let open_bi = tokio::sync::Mutex::new(connect_iroh().await.unwrap());
        std::sync::Arc::new(async move || {
            let mut open_bi = open_bi.lock().await;
            if let Some(bi) = reduce_err(open_bi().await, "couldn't bind iroh") {
                return Some(bi);
            }
            *open_bi = connect_iroh().await?;
            reduce_err(open_bi().await, "couldn't open bi again")
        })
    };
    let listener = tokio::net::TcpListener::bind(&listen_addr).await.unwrap();
    // Starting the client
    println!("Listening on {listen_addr}");
    loop {
        let Some((listen_stream, _addr)) = reduce_err(listener.accept().await, "couldn't accept listen stream") else { continue };
        let get_send_stream = get_send_stream.clone();
        tokio::spawn(async move {
            if let Some(send_stream) = get_send_stream().await {
                _ = reduce_err(iroh_tcp_transport::copy_bidir(send_stream, listen_stream).await, "couldn't copy bidir");
            }
        });
    }
}
