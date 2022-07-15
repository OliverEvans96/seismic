use std::time::Duration;

use clap::Parser;

use seismic::{
    sender::{Sender, SenderConfig},
    tracing::init_tracing,
};
use tracing::{error, info, instrument};

#[derive(Parser)]
struct Opts {
    /// Target IP / host
    target: String,
    /// Duration (in seconds) of transmission
    #[clap(short, default_value = "5")]
    length_secs: u16,
    /// Measurement frequency
    #[clap(short, default_value = "200")]
    freq_ms: u16,
    /// Bytes per chunk
    #[clap(short, default_value = "1024")]
    chunk_size: usize,
    /// TCP port for data transfer.
    #[clap(short = 'p', default_value = "7225")]
    data_port: u16,
    /// Don't print measurements as they're recorded
    quiet: bool,
}

impl From<Opts> for SenderConfig {
    fn from(opts: Opts) -> Self {
        Self {
            addr: format!("{}:{}", opts.target, opts.data_port),
            freq: Duration::from_millis(opts.freq_ms as u64),
            length: Duration::from_secs(opts.length_secs as u64),
            chunk_size: opts.chunk_size,
            print_live: !opts.quiet,
        }
    }
}

#[instrument(skip(config))]
async fn send_stream(config: SenderConfig) -> anyhow::Result<()> {
    let sender = Sender::new(config).await?;
    match sender.run().await {
        Ok(mset) => {
            mset.print();
            mset.plot();
        }
        Err(err) => {
            error!("send error: {}", err);
        }
    }

    Ok(())
}

#[instrument]
#[tokio::main]
async fn main() {
    init_tracing("seismic_client", tracing::Level::INFO).expect("failed to init tracing");

    let opts = Opts::parse();

    info!("Hello, client!");

    if let Err(err) = send_stream(opts.into()).await {
        error!("send_stream error: {}", err);
    }
}
