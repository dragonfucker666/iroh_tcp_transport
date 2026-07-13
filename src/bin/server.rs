use iroh_tcp_transport::reduce_err;

#[tokio::main]
async fn main() {
    // Collecting params
    let send_addr = std::sync::Arc::new(std::env::var("SEND_ADDR").expect("SEND_ADDR env var should be provided"));
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
    let iroh_endpoint = iroh_tcp_transport::build_endpoint().alpns(vec![iroh_tcp_transport::ALPN.to_vec()]).secret_key(secret_key).bind().await.expect("couldn't bind own iroh endpoint");
    // Listening
    println!("Listening on iroh");
    loop {
        let incoming_listener = iroh_endpoint.accept().await.unwrap();
        let send_addr = send_addr.clone();
        tokio::spawn(async move {
            let Some(conn) = reduce_err(incoming_listener.await, "couldn't accept incoming connection") else { return; };
            loop {
                let Some(listen_stream) = reduce_err(conn.accept_bi().await, "couldn't accept iroh conn") else { return; };
                let Some(send_stream) = reduce_err(tokio::net::TcpStream::connect(&*send_addr).await, "couldn't connect to iroh conn") else { return; };
                tokio::spawn(iroh_tcp_transport::copy_bidir(listen_stream, send_stream));
            }
        });
    }
}
