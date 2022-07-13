use std::{net::SocketAddr, time::Duration};

use tokio::net::{TcpListener, TcpStream};

use seismic::receiver::{Receiver, ReceiverConfig};
use seismic::{CONTROL_PORT, DATA_PORT};

async fn listen_control() -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", CONTROL_PORT);
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on control port {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_control(stream, addr));
    }

    Ok(())
}

async fn handle_control(_stream: TcpStream, addr: SocketAddr) {
    println!("Handling control connection from {}", addr);
}

async fn listen_data(config: ReceiverConfig) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", DATA_PORT);
    let listener = TcpListener::bind(&addr).await?;

    println!("Listening on data port {}", addr);
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_data(stream, addr, config));
    }

    Ok(())
}

async fn handle_data(stream: TcpStream, addr: SocketAddr, config: ReceiverConfig) {
    println!("Handling data connection from {}", addr);

    let receiver = Receiver::new(stream, config);

    match receiver.run().await {
        Ok(mset) => mset.print(),
        Err(err) => {
            eprintln!("data error: {}", err);
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, server!");

    // TODO: Set via config / CLI args
    let config = ReceiverConfig {
        freq: Duration::from_millis(200),
        length: Duration::from_secs(5),
        chunk_size: 1024,
    };

    let data_fut = listen_data(config);
    let control_fut = listen_control();

    let (data_res, control_res) = tokio::join!(data_fut, control_fut);

    if let Err(err) = data_res {
        eprintln!("Data error: {}", err)
    }

    if let Err(err) = control_res {
        eprintln!("Control error: {}", err)
    }
}
