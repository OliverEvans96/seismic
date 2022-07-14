use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use rand::{thread_rng, RngCore};
use tokio::{
    io::AsyncWriteExt,
    net::{tcp::OwnedWriteHalf, TcpStream},
};
use tracing::{debug, info, instrument};

use crate::{measurement::MeasurementSet, measurer::MeasurerStopper};
use crate::{measurer::Measurer, reader::SimpleReader};

#[derive(Debug)]
pub struct SenderConfig {
    pub addr: String,
    pub freq: Duration,
    pub length: Duration,
    pub chunk_size: usize,
}

pub struct Sender {
    /// TCP Stream to read from
    stream: TcpStream,
    /// Configuration values
    config: SenderConfig,
    /// Counter for chunks sent
    sent: Arc<AtomicU64>,
    /// Counter for chunks received
    received: Arc<AtomicU64>,
}

impl Sender {
    pub async fn new(config: SenderConfig) -> anyhow::Result<Self> {
        let sent = Arc::new(AtomicU64::new(0));
        let received = Arc::new(AtomicU64::new(0));
        let stream = TcpStream::connect(&config.addr).await?;
        let sender = Self {
            stream,
            config,
            sent,
            received,
        };

        Ok(sender)
    }

    /// TODO: Get rid of this method, probably.
    fn split(self) -> (Generator, SimpleReader, Measurer, MeasurerStopper) {
        let freq = self.config.freq;
        let (read_half, write_half) = self.stream.into_split();

        let generator = Generator::new(
            self.config.length,
            write_half,
            self.config.chunk_size,
            self.sent.clone(),
        );

        let reader = SimpleReader::new(read_half, self.config.chunk_size, self.received.clone());

        let (measurer, stopper) = Measurer::new(freq, self.sent, self.received);

        (generator, reader, measurer, stopper)
    }

    #[instrument(name = "Sender::run", skip(self))]
    pub async fn run(self) -> anyhow::Result<MeasurementSet> {
        let (generator, mut reader, mut measurer, stopper) = self.split();

        // Start measuring
        info!("Start measuring");
        let mfut = tokio::spawn(async move { measurer.run().await });

        // Start reading
        info!("Start reading");
        let read_fut = tokio::spawn(async move { reader.run().await });

        // Start writing
        info!("Start writing");
        let write_fut = tokio::spawn(async move { generator.run().await });

        // Wait for writing to complete
        info!("Wait for writing to complete");
        let write_res = write_fut.await?;

        // Wait for reading to complete
        info!("Wait for reading to complete");
        let read_res = read_fut.await?;

        // Stop measuring once reading is complete
        info!("Stop measuring");
        stopper.stop();

        // Get the measurements and return them
        // if reading and writing were successful
        let mset = mfut.await?;
        info!("End Sender::run");
        write_res.and(read_res).and(Ok(mset))
    }
}

/// Generate data and send it over the wire
pub struct Generator {
    length: Duration,
    write_half: OwnedWriteHalf,
    buf: Vec<u8>,
    /// Counter for chunks sent
    sent: Arc<AtomicU64>,
}

impl Generator {
    pub fn new(
        length: Duration,
        write_half: OwnedWriteHalf,
        chunk_size: usize,
        sent: Arc<AtomicU64>,
    ) -> Self {
        let mut buf = Vec::new();
        buf.resize(chunk_size, 0);

        Self {
            length,
            write_half,
            buf,
            sent,
        }
    }

    #[instrument(name = "Generator::run", skip(self))]
    pub async fn run(mut self) -> anyhow::Result<()> {
        let start_time = Instant::now();
        loop {
            let now = Instant::now();
            let elapsed = now - start_time;
            debug!("elapsed: {:.2}s", elapsed.as_secs_f64());
            if elapsed >= self.length {
                break;
            }

            // Generate random chunk of data
            {
                let mut rng = thread_rng();
                rng.fill_bytes(&mut self.buf);
            }
            debug!("A");
            // Send it over the wire
            self.write_half.write_all(&self.buf).await?;
            self.write_half.flush().await?;
            debug!("B");
            // Increment counter
            self.sent.fetch_add(1, Ordering::SeqCst);
            debug!("C");
        }

        info!("End Generator::run");
        Ok(())
    }
}
