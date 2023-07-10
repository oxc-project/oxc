use std::{
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    time::Duration,
};

#[derive(Debug)]
pub struct RuleTimer {
    pub secs: AtomicU64,
    pub nanos: AtomicU32,
}

impl RuleTimer {
    pub const fn new() -> Self {
        Self { secs: AtomicU64::new(0), nanos: AtomicU32::new(0) }
    }

    pub fn update(&mut self, duration: &Duration) {
        self.secs.fetch_add(duration.as_secs(), Ordering::SeqCst);
        self.nanos.fetch_add(duration.subsec_nanos(), Ordering::SeqCst);
    }

    pub fn duration(&self) -> Duration {
        let secs = self.secs.load(Ordering::SeqCst);
        let nanos = self.nanos.load(Ordering::SeqCst);
        Duration::new(secs, nanos)
    }
}
