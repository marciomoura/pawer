use std::collections::{BTreeSet, HashMap};

use pawer::types::Real;

/// A single recorded time step.
#[derive(Clone, Debug)]
pub struct LogRecord {
    pub time: f64,
    pub signals: HashMap<String, Real>,
}

/// Accumulates time-series data produced during simulation.
///
/// Each call to [`record`](Logger::record) captures one time step's worth
/// of signal values. The logged data can then be exported to CSV or plotted.
pub struct Logger {
    records: Vec<LogRecord>,
    known_signals: BTreeSet<String>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            known_signals: BTreeSet::new(),
        }
    }

    /// Record one time step. `signals` is the per-step buffer taken from
    /// [`SimContext`](crate::SimContext).
    pub fn record(&mut self, time: f64, signals: HashMap<String, Real>) {
        for name in signals.keys() {
            self.known_signals.insert(name.clone());
        }
        self.records.push(LogRecord { time, signals });
    }

    /// Remove all recorded data (but keep known signal names).
    pub fn clear(&mut self) {
        self.records.clear();
    }

    /// Full reset: clear records and forget signal names.
    pub fn reset(&mut self) {
        self.records.clear();
        self.known_signals.clear();
    }

    /// Sorted list of all signal names that have been logged at least once.
    pub fn signal_names(&self) -> Vec<String> {
        self.known_signals.iter().cloned().collect()
    }

    /// Number of recorded time steps.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Extract a single signal's time-series as `(time, value)` pairs.
    /// Steps where the signal was not logged are skipped.
    pub fn series(&self, name: &str) -> Vec<(f64, Real)> {
        self.records
            .iter()
            .filter_map(|r| r.signals.get(name).map(|&v| (r.time, v)))
            .collect()
    }

    /// Access all records.
    pub fn records(&self) -> &[LogRecord] {
        &self.records
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
