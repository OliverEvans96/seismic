use std::{
    net::SocketAddr,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use anyhow::bail;
use seismic::{CONTROL_PORT, DATA_PORT};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

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

async fn listen_data() -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", DATA_PORT);
    let listener = TcpListener::bind(&addr).await?;

    println!("Listening on data port {}", addr);
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_data(stream, addr));
    }

    Ok(())
}

async fn handle_data(stream: TcpStream, addr: SocketAddr) {
    println!("Handling data connection from {}", addr);

    if let Err(err) = recv_stream(stream).await {
        eprintln!("data error: {}", err);
    }
}

async fn read_bytes<const CHUNK_SIZE: usize>(
    mut stream: TcpStream,
    counter: &AtomicU64,
) -> anyhow::Result<()> {
    let mut buf: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

    loop {
        match stream.read_exact(&mut buf).await {
            Ok(nbytes) => {
                // TODO: Which ordering?
                counter.fetch_add(1, Ordering::SeqCst);

                if nbytes != CHUNK_SIZE {
                    bail!(
                        "incorrect number of bytes read. nbytes = {}, CHUNK_SIZE = {}",
                        nbytes,
                        CHUNK_SIZE
                    );
                }

                // TODO: Not returning error
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }
}

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
    measurements: Vec<Measurement>,
}

impl MeasurementSet {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            measurements: Vec::new(),
        }
    }

    pub fn record(&mut self, count: u64) {
        let measurement = Measurement::new(self.start, count);
        self.measurements.push(measurement);
    }
}

async fn take_measurements(
    freq: Duration,
    length: Duration,
    counter: &AtomicU64,
) -> MeasurementSet {
    // Ticks once for each measurement
    let mut interval = tokio::time::interval(freq);
    // Ticks once at the end of the set
    let timer = tokio::time::sleep(length);

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

fn print_measurements(mset: &MeasurementSet) {
    for measurement in &mset.measurements {
        let secs = measurement.dt.as_secs_f64();
        println!("{:.2}s: {}", secs, measurement.count);
    }
}

async fn recv_stream(stream: TcpStream) -> anyhow::Result<()> {
    // TODO: Refactor w/ Receiver struct

    const CHUNK_SIZE: usize = 1024;
    let counter: AtomicU64 = AtomicU64::new(0);

    // TODO: Set via config / CLI args
    let freq = Duration::from_millis(200);
    let length = Duration::from_secs(5);

    let read_fut = read_bytes::<CHUNK_SIZE>(stream, &counter);
    let measure_fut = take_measurements(freq, length, &counter);

    let (read_res, mset) = tokio::join!(read_fut, measure_fut);

    // TODO: DO something else with mset?
    print_measurements(&mset);

    read_res
}

#[tokio::main]
async fn main() {
    println!("Hello, server!");

    let data_fut = listen_data();
    let control_fut = listen_control();

    let (data_res, control_res) = tokio::join!(data_fut, control_fut);

    if let Err(err) = data_res {
        eprintln!("Data error: {}", err)
    }

    if let Err(err) = control_res {
        eprintln!("Control error: {}", err)
    }
}
