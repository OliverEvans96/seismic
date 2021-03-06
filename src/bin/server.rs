use std::{net::SocketAddr, time::Duration};

use clap::Parser;
use tokio::net::{TcpListener, TcpStream};

use seismic::{
    receiver::{Receiver, ReceiverConfig},
    tracing::init_tracing,
};
use tracing::{error, info, instrument};

#[derive(Parser)]
struct Opts {
    /// TCP port for control commands.
    #[clap(long, default_value = "7224")]
    control_port: u16,
    /// TCP port for data transfer.
    #[clap(long, default_value = "7225")]
    data_port: u16,
    /// Bytes per chunk
    #[clap(short, default_value = "1024")]
    chunk_size: u16,
    /// Measurement frequency
    #[clap(short, default_value = "200")]
    freq_ms: u16,
    /// Don't print measurements as they're recorded
    #[clap(short)]
    quiet: bool,
    /// Print INFO statements (default is WARN+)
    #[clap(short)]
    verbose: bool,
    //// Enable tracing to Jaeger
    #[clap(short)]
    jaeger: bool,
}

#[instrument]
async fn listen_control(port: u16) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on control port {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_control(stream, addr));
    }

    Ok(())
}

#[instrument(skip(_stream))]
async fn handle_control(_stream: TcpStream, addr: SocketAddr) {
    info!("Handling control connection from {}", addr);
}

#[instrument(skip(config))]
async fn listen_data(port: u16, config: ReceiverConfig) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Listening on data port {}", addr);
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_data(stream, addr, config));
    }

    Ok(())
}

#[instrument(skip(stream, config))]
async fn handle_data(stream: TcpStream, addr: SocketAddr, config: ReceiverConfig) {
    info!("Handling data connection from {}", addr);

    let receiver = Receiver::new(stream, config);

    match receiver.run().await {
        Ok(mset) => {
            mset.print();
            mset.plot();
        }
        Err(err) => {
            error!("data error: {}", err);
        }
    }
}

impl From<Opts> for ReceiverConfig {
    fn from(opts: Opts) -> Self {
        Self {
            freq: Duration::from_millis(opts.freq_ms.into()),
            chunk_size: opts.chunk_size.into(),
            echo: true,
            print_live: !opts.quiet,
        }
    }
}

#[instrument]
#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let level = if opts.verbose {
        tracing::Level::INFO
    } else {
        tracing::Level::WARN
    };

    init_tracing("seismic_server", level, opts.jaeger).expect("failed to init tracing");

    info!("Hello, server!");

    let control_fut = listen_control(opts.control_port);
    let data_fut = listen_data(opts.data_port, opts.into());

    let (data_res, control_res) = tokio::join!(data_fut, control_fut);

    if let Err(err) = data_res {
        error!("Data error: {}", err)
    }

    if let Err(err) = control_res {
        error!("Control error: {}", err)
    }
}
