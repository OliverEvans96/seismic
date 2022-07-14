use std::{net::SocketAddr, time::Duration};

use clap::Parser;
use tokio::net::{TcpListener, TcpStream};

use seismic::receiver::{Receiver, ReceiverConfig};

#[derive(Parser)]
struct Opts {
    /// TCP port for control commands.
    #[clap(long, default_value = "7224")]
    control_port: u16,
    /// TCP port for data transfer.
    #[clap(long, default_value = "7225")]
    data_port: u16,
    #[clap(short, default_value = "1024")]
    chunk_size: u16,
    #[clap(short, default_value = "200")]
    freq_ms: u16,
}

async fn listen_control(port: u16) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
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

async fn listen_data(port: u16, config: ReceiverConfig) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
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
    // console_subscriber::init();
    let opts = Opts::parse();

    println!("Hello, server!");

    let config = ReceiverConfig {
        freq: Duration::from_millis(opts.freq_ms.into()),
        chunk_size: opts.chunk_size.into(),
        echo: true,
    };

    let data_fut = listen_data(opts.data_port, config);
    let control_fut = listen_control(opts.control_port);

    let (data_res, control_res) = tokio::join!(data_fut, control_fut);

    if let Err(err) = data_res {
        eprintln!("Data error: {}", err)
    }

    if let Err(err) = control_res {
        eprintln!("Control error: {}", err)
    }
}
