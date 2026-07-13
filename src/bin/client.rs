use iroh::{EndpointAddr, PublicKey};
use tokio::net::{TcpListener, TcpStream};

async fn try_begin_serving(iroh_conn: iroh::endpoint::Connection, tcp_stream: TcpStream) -> Result<impl std::future::Future<Output = impl Send>, (iroh::endpoint::ConnectionError, TcpStream)> {
    match iroh_conn.open_bi().await {
        Ok(iroh_stream) => Ok(iroh_tcp_transport::copy_between_iroh_and_tcp_streams(tcp_stream, iroh_stream)),
        Err(err) => Err((err, tcp_stream)),
    }
}

async fn run() {
    let iroh_pubkey: PublicKey = std::env::var("IROH_PUBKEY").expect("you should set IROH_PUBKEY env var").parse().unwrap();
    let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_err| String::from("127.0.0.1:1080"));
    let iroh_reconnect = async move || {
        iroh_tcp_transport::prepare_iroh_endpoint_builder().bind().await.unwrap().connect(EndpointAddr::new(iroh_pubkey), iroh_tcp_transport::ALPN).await
    };
    let mut iroh_conn = iroh_reconnect().await.unwrap();
    let listener = TcpListener::bind(&listen_addr).await.unwrap();
    println!("Listening on {listen_addr}");
    loop {
        match listener.accept().await {
            Ok((tcp_stream, _addr)) => {
                let mut opening_result = try_begin_serving(iroh_conn.clone(), tcp_stream).await;
                if let Err((err, tcp_stream)) = opening_result {
                    match iroh_reconnect().await {
                        Ok(new_iroh_conn) => iroh_conn = new_iroh_conn,
                        Err(e) => eprintln!("{e}"),
                    }
                    opening_result = try_begin_serving(iroh_conn.clone(), tcp_stream).await;
                }
                match opening_result {
                    Err((err, _tcp_stream)) => eprintln!("{err}"),
                    Ok(server) => {
                        tokio::spawn(server).await.unwrap();
                    },
                }
            },
            Err(e) => eprintln!("{e}"),
        }
    }
}

fn main() {
    tokio::runtime::Builder::new_multi_thread().build().unwrap().block_on(run());
}
