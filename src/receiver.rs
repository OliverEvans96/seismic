use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use anyhow::bail;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::measurement::MeasurementSet;

pub struct Receiver {
    /// TCP Stream to read from
    stream: TcpStream,
    /// Configuration values
    config: ReceiverConfig,
    /// Atomic chunk counter
    counter: AtomicU64,
}

#[derive(Clone, Copy)]
pub struct ReceiverConfig {
    /// Total duration of measurement set
    pub length: Duration,
    /// Measurement frequency
    pub freq: Duration,
    /// Bytes per chunk
    pub chunk_size: usize,
    /// Whether to echo data back to stream
    pub echo: bool,
}

impl Receiver {
    pub fn new(stream: TcpStream, config: ReceiverConfig) -> Self {
        let counter = AtomicU64::new(0);
        Self {
            stream,
            config,
            counter,
        }
    }

    fn split(self) -> (AtomicU64, Reader, Measurer) {
        let stream = self.stream;
        let config = self.config;

        let reader = Reader::new(stream, config);

        let measurer = Measurer::new(config);

        (self.counter, reader, measurer)
    }

    pub async fn run(self) -> anyhow::Result<MeasurementSet> {
        let (counter, mut reader, measurer) = self.split();

        let read_fut = reader.run(&counter);
        let measure_fut = measurer.run(&counter);
        let (read_res, mset) = tokio::join!(read_fut, measure_fut);

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

    fn check_size(&self, nbytes: usize) -> anyhow::Result<()> {
        if nbytes != self.buf.len() {
            bail!(
                "incorrect number of bytes read. nbytes = {}, CHUNK_SIZE = {}",
                nbytes,
                self.buf.len(),
            );
        }
        Ok(())
    }

    async fn read_chunk(&mut self, counter: &AtomicU64) -> anyhow::Result<()> {
        println!("Read chunk ({})", self.buf.len());
        match self.stream.read_exact(&mut self.buf).await {
            Ok(nbytes) => {
                println!("Read {} bytes", nbytes);
                // TODO: Which ordering?
                counter.fetch_add(1, Ordering::SeqCst);
                // TODO: Parse, don't validate?
                self.check_size(nbytes)?;

                // Optionally reply on stream
                if self.config.echo {
                    self.stream.write_all(&self.buf).await?;
                }

                Ok(())
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    async fn run(&mut self, counter: &AtomicU64) -> anyhow::Result<()> {
        // Ticks once at the end of the set
        let timer = tokio::time::sleep(self.config.length);
        // Necessary according to the docs
        // since we poll it more than once
        tokio::pin!(timer);

        loop {
            tokio::select! {
                _ = self.read_chunk(counter) => {}
                _ = &mut timer => { break; }
            }
        }

        Ok(())
    }
}

struct Measurer {
    config: ReceiverConfig,
}

impl Measurer {
    fn new(config: ReceiverConfig) -> Self {
        Self { config }
    }

    async fn run(&self, counter: &AtomicU64) -> MeasurementSet {
        // Ticks once for each measurement
        let mut interval = tokio::time::interval(self.config.freq);
        // Ticks once at the end of the set
        let timer = tokio::time::sleep(self.config.length);

        // Necessary according to the docs
        // since we poll it more than once
        tokio::pin!(timer);

        let mut mset = MeasurementSet::new();

        loop {
            tokio::select! {
                _ = &mut timer => { break; }
                _ = interval.tick() => {
                    // TODO: Which ordering?
                    let count = counter.load(Ordering::SeqCst);
                    mset.record(count);
                }
            };
        }

        mset
    }
}
