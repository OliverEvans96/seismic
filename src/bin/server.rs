use std::{
    net::SocketAddr,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime},
};

use anyhow::bail;
use seismic::{CONTROL_PORT, DATA_PORT};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

pub struct Measurement {
    /// Time offset start beginning of measurement set
    pub dt: Duration,
    /// Number of chunks sent/received
    pub count: u64,
}

impl Measurement {
    pub fn new(start: Instant, count: u64) -> Self {
        let now = Instant::now();
        let dt = now - start;
        Self { dt, count }
    }
}

struct MeasurementSet {
    start: Instant,
    start_time: SystemTime,
    pub measurements: Vec<Measurement>,
}

impl MeasurementSet {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            start_time: SystemTime::now(),
            measurements: Vec::new(),
        }
    }

    pub fn record(&mut self, count: u64) {
        let measurement = Measurement::new(self.start, count);
        self.measurements.push(measurement);
    }

    pub fn print(&self) {
        println!("Measurements @ {:?}", self.start_time);
        for measurement in &self.measurements {
            let secs = measurement.dt.as_secs_f64();
            println!("{:.2}s: {}", secs, measurement.count);
        }
    }
}

struct Receiver {
    stream: TcpStream,
    config: ReceiverConfig,
    counter: AtomicU64,
}

#[derive(Clone, Copy)]
struct ReceiverConfig {
    pub length: Duration,
    pub freq: Duration,
    pub chunk_size: usize,
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

    pub fn split(self) -> (AtomicU64, Reader, Measurer) {
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
    length: Duration,
    buf: Vec<u8>,
}

impl Reader {
    pub fn new(stream: TcpStream, config: ReceiverConfig) -> Self {
        let buf = Vec::with_capacity(config.chunk_size);
        let length = config.length;
        Self {
            stream,
            length,
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
        match self.stream.read_exact(&mut self.buf).await {
            Ok(nbytes) => {
                // TODO: Which ordering?
                counter.fetch_add(1, Ordering::SeqCst);
                // TODO: Parse, don't validate?
                self.check_size(nbytes)
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    async fn run(&mut self, counter: &AtomicU64) -> anyhow::Result<()> {
        // Ticks once at the end of the set
        let timer = tokio::time::sleep(self.length);
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
