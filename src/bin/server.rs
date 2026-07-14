use iroh_tcp_transport::reduce_err;

#[tokio::main]
async fn main() {
    // Collecting params
    let send_addr = std::sync::Arc::new(std::env::var("SEND_ADDR").expect("SEND_ADDR env var should be provided"));
    let iroh_relay_url = std::env::var("IROH_RELAY_URL").ok();
    let secret_key = match std::env::var("SECRET_KEY") {
        Err(_e) => {
            let secret_key = iroh::SecretKey::generate();
            println!("secret key: {secret_key:?}");
            secret_key
        },
        Ok(secret_key_string) => secret_key_string.parse::<iroh::SecretKey>().unwrap(),
    };
    println!("public key: {}", secret_key.public());
    // Building tools
    let mut builder = iroh::Endpoint::builder(iroh::endpoint::presets::N0).alpns(vec![iroh_tcp_transport::ALPN.to_vec()]).secret_key(secret_key);
    if let Some(iroh_relay_url) = iroh_relay_url {
        let iroh_relay_url: iroh::RelayUrl = iroh_relay_url.parse().unwrap();
        builder = builder.relay_mode(iroh::RelayMode::Custom(iroh_relay_url.into()));
    }
    let iroh_endpoint = builder.bind().await.expect("couldn't bind own iroh endpoint");
    // Starting the server
    println!("Listening on iroh");
    loop {
        let incoming_listener = iroh_endpoint.accept().await.unwrap();
        let send_addr = send_addr.clone();
        tokio::spawn(async move {
            let Some(conn) = reduce_err(incoming_listener.await, "couldn't accept incoming connection") else { return; };
            loop {
                let Some(listen_stream) = reduce_err(conn.accept_bi().await, "couldn't accept iroh conn") else { return; };
                let Some(send_stream) = reduce_err(tokio::net::TcpStream::connect(&*send_addr).await, "couldn't connect to iroh conn") else { return; };
                tokio::spawn(async move {
                    _ = reduce_err(iroh_tcp_transport::copy_bidir(listen_stream, send_stream).await, "couldn't copy bidir");
                });
            }
        });
    }
}
