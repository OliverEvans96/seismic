use std::{
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

use crate::measurement::MeasurementSet;
use crate::measurer::Measurer;

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
    /// Atomic chunk counter
    counter: Arc<AtomicU64>,
}

impl Sender {
    pub async fn new(config: SenderConfig) -> anyhow::Result<Self> {
        let counter = Arc::new(AtomicU64::new(0));
        let stream = TcpStream::connect(&config.addr).await?;
        let sender = Self {
            stream,
            config,
            counter,
        };

        Ok(sender)
    }

    /// TODO: Get rid of this method, probably.
    fn split(self) -> (Arc<AtomicU64>, Generator) {
        let (_reader, writer) = self.stream.into_split();
        let generator = Generator::new(self.config.length, writer, self.config.chunk_size);

        (self.counter, generator)
    }

    pub async fn run(self) -> anyhow::Result<MeasurementSet> {
        let freq = self.config.freq;
        let (counter, mut generator) = self.split();
        let (mut measurer, stopper) = Measurer::new(freq, counter.clone());

        // Start measuring
        let mfut = tokio::spawn(async move { measurer.run().await });

        // Start writing
        println!("Start writing");
        let write_res = generator.run(counter.clone()).await;
        // Stop measuring once reading is complete
        println!("STOP");
        stopper.stop();
        println!("STOPPED");

        // Get the measurements and return them
        // if writing was successful
        let mset = mfut.await?;
        println!("End Sender::run");
        write_res.and(Ok(mset))
    }
}

/// Generate data and send it over the wire
pub struct Generator {
    length: Duration,
    writer: OwnedWriteHalf,
    buf: Vec<u8>,
}

impl Generator {
    pub fn new(length: Duration, writer: OwnedWriteHalf, chunk_size: usize) -> Self {
        let mut buf = Vec::new();
        buf.resize(chunk_size, 0);

        Self {
            length,
            writer,
            buf,
        }
    }

    pub async fn run(&mut self, counter: Arc<AtomicU64>) -> anyhow::Result<()> {
        let start_time = Instant::now();
        let mut rng = thread_rng();
        loop {
            let now = Instant::now();
            let elapsed = now - start_time;
            println!("elapsed: {:.2}s", elapsed.as_secs_f64());
            if elapsed >= self.length {
                break;
            }

            // Generate random chunk of data
            rng.fill_bytes(&mut self.buf);
            println!("A");
            // Send it over the wire
            self.writer.write_all(&self.buf).await?;
            println!("B");
            // Increment counter
            counter.fetch_add(1, Ordering::SeqCst);
            println!("C");
        }

        println!("End Generator::run");
        Ok(())
    }
}
