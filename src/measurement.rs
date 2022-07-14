use std::time::{Duration, Instant, SystemTime};

pub struct Measurement {
    /// Time offset start beginning of measurement set
    pub dt: Duration,
    /// Number of chunks sent
    pub sent: u64,
    /// Number of chunks received
    pub received: u64,
}

impl Measurement {
    pub fn new(start: Instant, sent: u64, received: u64) -> Self {
        let now = Instant::now();
        let dt = now - start;
        Self { dt, sent, received }
    }
}

pub struct MeasurementSet {
    start: Instant,
    start_time: SystemTime,
    pub measurements: Vec<Measurement>,
}

impl Default for MeasurementSet {
    fn default() -> Self {
        Self::new()
    }
}

impl MeasurementSet {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            start_time: SystemTime::now(),
            measurements: Vec::new(),
        }
    }

    pub fn record(&mut self, sent: u64, received: u64) {
        let measurement = Measurement::new(self.start, sent, received);
        self.measurements.push(measurement);
    }

    pub fn print(&self) {
        println!("Measurements @ {:?}", self.start_time);
        for measurement in &self.measurements {
            let secs = measurement.dt.as_secs_f64();
            println!(
                "{:.2}s: {:10} sent / {:10} received",
                secs, measurement.sent, measurement.received
            );
        }
    }
}
