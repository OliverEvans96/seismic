use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::sync::oneshot;
use tracing::{info, instrument};

use crate::measurement::MeasurementSet;

/// Measures a counter periodically,
/// stopping when a signal is given.
pub struct Measurer {
    /// Measurement frequency
    freq: Duration,
    /// Counter for chunks sent
    sent: Arc<AtomicU64>,
    /// Counter for chunks received
    received: Arc<AtomicU64>,
    /// One-shot channel indicating
    /// measurement should end.
    stop: Pin<Box<oneshot::Receiver<()>>>,
}

pub struct MeasurerStopper(oneshot::Sender<()>);

impl MeasurerStopper {
    pub fn stop(self) {
        // Convert Result to Option via .ok()
        // because failure means that the other
        // end has already hung up, which means
        // that stopping has already occurred.
        self.0.send(()).ok();
        info!("MeasurerStopper::stop()");
    }
}

impl Measurer {
    pub fn new(
        freq: Duration,
        sent: Arc<AtomicU64>,
        received: Arc<AtomicU64>,
    ) -> (Self, MeasurerStopper) {
        let (stop_send, stop_recv) = oneshot::channel();
        let stopper = MeasurerStopper(stop_send);
        let stop = Box::pin(stop_recv);
        let measurer = Self {
            freq,
            sent,
            received,
            stop,
        };

        (measurer, stopper)
    }

    #[instrument(name = "Measurer::run", skip(self))]
    pub async fn run(&mut self) -> MeasurementSet {
        // Ticks once for each measurement
        let mut interval = tokio::time::interval(self.freq);

        let mut mset = MeasurementSet::new();

        loop {
            tokio::select! {
                _ = &mut self.stop => { break; }
                _ = interval.tick() => {
                    let sent = self.sent.load(Ordering::SeqCst);
                    let received = self.received.load(Ordering::SeqCst);
                    mset.record(sent, received);
                }
            }
        }

        info!("End Measurer::run");
        mset
    }
}
