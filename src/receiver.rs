use std::{
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use tokio::net::TcpStream;
use tracing::{info, instrument};

use crate::{
    measurement::MeasurementSet,
    measurer::{Measurer, MeasurerStopper},
    reader::{EchoingReader, Reader, SimpleReader},
};

#[derive(Clone, Copy)]
pub struct ReceiverConfig {
    /// Measurement frequency
    pub freq: Duration,
    /// Bytes per chunk
    pub chunk_size: usize,
    /// Whether to echo data back to stream
    pub echo: bool,
}

pub struct Receiver {
    /// TCP Stream to read from
    stream: TcpStream,
    /// Configuration values
    config: ReceiverConfig,
    /// Counter for chunks sent
    sent: Arc<AtomicU64>,
    /// Counter for chunks received
    received: Arc<AtomicU64>,
}

impl Receiver {
    pub fn new(stream: TcpStream, config: ReceiverConfig) -> Self {
        let sent = Arc::new(AtomicU64::new(0));
        let received = Arc::new(AtomicU64::new(0));
        Self {
            stream,
            config,
            sent,
            received,
        }
    }

    /// TODO: Get rid of this method, probably.
    fn split(self) -> (Reader, Measurer, MeasurerStopper) {
        let freq = self.config.freq;

        let reader = if self.config.echo {
            let inner = EchoingReader::new(
                self.stream,
                self.config.chunk_size,
                self.sent.clone(),
                self.received.clone(),
            );
            Reader::Echoing(inner)
        } else {
            let (read_half, _write_half) = self.stream.into_split();
            let inner = SimpleReader::new(read_half, self.config.chunk_size, self.received.clone());
            Reader::Simple(inner)
        };

        let (measurer, stopper) = Measurer::new(freq, self.sent, self.received);

        (reader, measurer, stopper)
    }

    #[instrument(name = "Receiver::run", skip(self))]
    pub async fn run(self) -> anyhow::Result<MeasurementSet> {
        let (mut reader, mut measurer, stopper) = self.split();

        // Start measuring
        let mfut = tokio::spawn(async move { measurer.run().await });

        // Start reading
        let read_res = reader.run().await;
        // Stop measuring once reading is complete
        stopper.stop();

        // Get the measurements and return them
        // if reading was successful
        let mset = mfut.await?;
        info!("End Receiver::run");
        read_res.and(Ok(mset))
    }
}
