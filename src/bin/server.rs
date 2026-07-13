use iroh::{Endpoint, EndpointAddr, RelayUrl, SecretKey, endpoint::presets};
use tokio::net::{TcpListener, TcpStream};

async fn serve(iroh_conn: iroh::endpoint::Connection, tcp_stream: TcpStream) {
    match iroh_conn.open_bi().await {
        Err(e) => eprintln!("{e}"),
        Ok(iroh_stream) => iroh_tcp_transport::copy_between_iroh_and_tcp_streams(tcp_stream, iroh_stream).await,
    }
}

async fn run() {
    let iroh_seckey: SecretKey = match std::env::var("IROH_SECKEY") {
        Ok(seckey_string) => seckey_string.parse().unwrap(),
        Err(_err) => {
            let seckey = SecretKey::generate();
            println!("Secret key: {seckey:?}");
            println!("Public key: {}", seckey.public());
            seckey
        },
    };
    let send_addr = std::env::var("SEND_ADDR").expect("you should set SEND_ADDR env var");
    let iroh_listener = iroh_tcp_transport::prepare_iroh_endpoint_builder().secret_key(iroh_seckey).alpns(vec![iroh_tcp_transport::ALPN.to_vec()]).bind().await;
    let mut iroh_conn = iroh_reconnect().await.unwrap();
    let listener = TcpListener::bind(&listen_addr).await.unwrap();
    println!("Listening on {listen_addr}");
    loop {
        match listener.accept().await {
            Ok((tcp_stream, _addr)) => {
                if iroh_conn.close_reason().is_some() {
                    match iroh_reconnect().await {
                        Ok(new_iroh_conn) => iroh_conn = new_iroh_conn,
                        Err(e) => eprintln!("{e}"),
                    }
                }
                tokio::spawn(serve(iroh_conn.clone(), tcp_stream));
            },
            Err(e) => eprintln!("{e}"),
        }
    }
}

fn main() {
    tokio::runtime::Builder::new_multi_thread().build().unwrap().block_on(run());
}
