//! Clock Service (Architecture Bible, Part 2A §18).
//!
//! "Entire system same clock use kare." No module should call
//! `std::time` or `chrono::Utc::now()` directly — everything routes
//! through this service so time can be mocked/frozen in tests and
//! so monotonic vs wall-clock time stays consistent system-wide.

use crate::registry::Service;
use chrono::{DateTime, Utc};
use std::time::Instant;

pub trait ClockService: Service {
    fn now(&self) -> DateTime<Utc>;
    fn monotonic(&self) -> Instant;
}

/// Real system clock, used in production.
pub struct SystemClock {
    boot_instant: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            boot_instant: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Service for SystemClock {
    fn name(&self) -> &str {
        "clock"
    }
}

impl ClockService for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn monotonic(&self) -> Instant {
        self.boot_instant
    }
}
