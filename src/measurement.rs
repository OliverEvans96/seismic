use std::time::{Duration, Instant, SystemTime};

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

pub struct MeasurementSet {
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
        println!("Record {}", count);
    }

    pub fn print(&self) {
        println!("Measurements @ {:?}", self.start_time);
        for measurement in &self.measurements {
            let secs = measurement.dt.as_secs_f64();
            println!("{:.2}s: {}", secs, measurement.count);
        }
    }
}
