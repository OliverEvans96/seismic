use std::{
    io::ErrorKind,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{measurement::MeasurementSet, measurer::Measurer};

pub struct Receiver {
    /// TCP Stream to read from
    stream: TcpStream,
    /// Configuration values
    config: ReceiverConfig,
    /// Atomic chunk counter
    counter: Arc<AtomicU64>,
}

#[derive(Clone, Copy)]
pub struct ReceiverConfig {
    /// Measurement frequency
    pub freq: Duration,
    /// Bytes per chunk
    pub chunk_size: usize,
    /// Whether to echo data back to stream
    pub echo: bool,
}

impl Receiver {
    pub fn new(stream: TcpStream, config: ReceiverConfig) -> Self {
        let counter = Arc::new(AtomicU64::new(0));
        Self {
            stream,
            config,
            counter,
        }
    }

    /// TODO: Get rid of this method, probably.
    fn split(self) -> (Arc<AtomicU64>, Reader) {
        let stream = self.stream;
        let config = self.config;

        let reader = Reader::new(stream, config);

        (self.counter, reader)
    }

    pub async fn run(self) -> anyhow::Result<MeasurementSet> {
        let freq = self.config.freq;
        let (counter, mut reader) = self.split();
        let (mut measurer, stopper) = Measurer::new(freq, counter.clone());

        // Start measuring
        let mfut = tokio::spawn(async move { measurer.run().await });

        // Start reading
        let read_res = reader.run(counter).await;
        // Stop measuring once reading is complete
        stopper.stop();

        // Get the measurements and return them
        // if reading was successful
        let mset = mfut.await?;
        println!("End Receiver::run");
        read_res.and(Ok(mset))
    }
}

struct Reader {
    stream: TcpStream,
    config: ReceiverConfig,
    buf: Vec<u8>,
}

impl Reader {
    pub fn new(stream: TcpStream, config: ReceiverConfig) -> Self {
        let mut buf = Vec::new();
        buf.resize(config.chunk_size, 0);

        Self {
            stream,
            config,
            buf,
        }
    }

    async fn read_chunk(&mut self, counter: &AtomicU64) -> std::io::Result<()> {
        println!("Read chunk ({}): {:?}", self.buf.len(), &self.buf[..5]);
        match self.stream.read_exact(&mut self.buf).await {
            Ok(nbytes) => {
                println!("Read {} bytes", nbytes);
                counter.fetch_add(1, Ordering::SeqCst);

                // Optionally reply on stream
                if self.config.echo {
                    self.stream.write_all(&self.buf).await?;
                }

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    async fn run(&mut self, counter: Arc<AtomicU64>) -> anyhow::Result<()> {
        loop {
            if let Err(err) = self.read_chunk(&counter).await {
                println!("End Reader::run");

                // EOF _is_ expected here.
                if err.kind() == ErrorKind::UnexpectedEof {
                    return Ok(());
                } else {
                    return Err(err.into());
                }
            }
        }
    }
}
