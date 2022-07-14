use std::time::{Duration, Instant, SystemTime};

use ansi_rgb::Foreground;
use rgb::RGB8;
use textplots::{Chart, ColorPlot, Shape};
use tracing::debug;

#[derive(Debug)]
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
        let measurement = Self { dt, sent, received };
        debug!("{:?}", measurement);
        measurement
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
        // TODO: Format SystemTime
        println!("Measurements @ {:?}", self.start_time);
        for measurement in &self.measurements {
            let secs = measurement.dt.as_secs_f64();
            println!(
                "{:.2}s: {:10} sent / {:10} received",
                secs, measurement.sent, measurement.received
            );
        }
        println!("");
    }

    pub fn time(&self) -> Vec<f64> {
        self.measurements
            .iter()
            .map(|m| m.dt.as_secs_f64())
            .collect()
    }

    pub fn sent(&self) -> Vec<u64> {
        self.measurements.iter().map(|m| m.sent).collect()
    }

    pub fn received(&self) -> Vec<u64> {
        self.measurements.iter().map(|m| m.received).collect()
    }

    pub fn plot(&self) {
        let t: Vec<f32> = self.time().into_iter().map(|x| x as f32).collect();
        let s: Vec<f32> = self.sent().into_iter().map(|x| x as f32).collect();
        let r: Vec<f32> = self.received().into_iter().map(|x| x as f32).collect();

        let ts: Vec<_> = t.iter().cloned().zip(s).collect();
        let tr: Vec<_> = t.iter().cloned().zip(r).collect();

        let sent_shape = Shape::Lines(&ts);
        let received_shape = Shape::Lines(&tr);

        let width = 120;
        let height = 60;
        let xmin = *t.first().expect("cannot plot with no first t value");
        let xmax = *t.last().expect("cannot plot with no last t value");

        let red: RGB8 = [255, 0, 0].into();
        let green: RGB8 = [0, 255, 0].into();

        Chart::new(width, height, xmin, xmax)
            .linecolorplot(&sent_shape, red.clone())
            .linecolorplot(&received_shape, green.clone())
            .nice();

        println!("{} {}", "sent".fg(red), "received".fg(green));
    }
}
