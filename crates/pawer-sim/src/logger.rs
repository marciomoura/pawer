use std::collections::{BTreeSet, HashMap};

use pawer::types::Real;

/// A single recorded time step.
#[derive(Clone, Debug)]
pub struct LogRecord {
    pub time: f64,
    /// Dense, indexed by [`SignalId`](crate::context::SignalId).
    pub registered: Vec<Real>,
    /// Sparse, for ad-hoc signals logged via [`SimContext::log`](crate::context::SimContext::log).
    pub adhoc: HashMap<String, Real>,
}

/// Accumulates time-series data produced during simulation.
///
/// Registered signals are stored in a dense `Vec<Real>` per record for
/// optimal cache performance — no string lookups or HashMap operations on
/// the hot path. Ad-hoc signals (`ctx.log("name", v)`) are still supported
/// via a per-record HashMap for backward compatibility.
pub struct Logger {
    registered_names: Vec<String>,
    adhoc_names: BTreeSet<String>,
    records: Vec<LogRecord>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            registered_names: Vec::new(),
            adhoc_names: BTreeSet::new(),
            records: Vec::new(),
        }
    }

    /// Set the registered signal names. Called once after `Scenario::init`.
    pub(crate) fn set_registered_names(&mut self, names: Vec<String>) {
        self.registered_names = names;
    }

    /// Record one time step.
    pub fn record(&mut self, time: f64, registered: Vec<Real>, adhoc: HashMap<String, Real>) {
        for name in adhoc.keys() {
            self.adhoc_names.insert(name.clone());
        }
        self.records.push(LogRecord {
            time,
            registered,
            adhoc,
        });
    }

    /// Remove all recorded data (but keep known signal names).
    pub fn clear(&mut self) {
        self.records.clear();
    }

    /// Full reset: clear records and forget signal names.
    pub fn reset(&mut self) {
        self.records.clear();
        self.registered_names.clear();
        self.adhoc_names.clear();
    }

    /// Sorted list of all signal names (registered + ad-hoc).
    pub fn signal_names(&self) -> Vec<String> {
        let mut names: BTreeSet<String> = self.registered_names.iter().cloned().collect();
        names.extend(self.adhoc_names.iter().cloned());
        names.into_iter().collect()
    }

    /// Names of registered signals, in registration order.
    pub fn registered_names(&self) -> &[String] {
        &self.registered_names
    }

    /// Number of recorded time steps.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Extract a single signal's time-series as `(time, value)` pairs.
    /// Checks registered signals first (by name → index), then ad-hoc.
    pub fn series(&self, name: &str) -> Vec<(f64, Real)> {
        // Check if it's a registered signal
        if let Some(idx) = self.registered_names.iter().position(|n| n == name) {
            return self
                .records
                .iter()
                .filter_map(|r| r.registered.get(idx).map(|&v| (r.time, v)))
                .collect();
        }
        // Fall back to ad-hoc
        self.records
            .iter()
            .filter_map(|r| r.adhoc.get(name).map(|&v| (r.time, v)))
            .collect()
    }

    /// Access all records.
    pub fn records(&self) -> &[LogRecord] {
        &self.records
    }

    /// Return the latest recorded value for each of the requested signal
    /// names. If `names` is empty, returns all known signals.
    /// Missing signals yield `None`.
    pub fn snapshot(&self, names: &[String]) -> Vec<(String, Option<Real>)> {
        let last = self.records.last();
        let query: Vec<String> = if names.is_empty() {
            self.signal_names()
        } else {
            names.to_vec()
        };

        query
            .into_iter()
            .map(|name| {
                let value = last.and_then(|r| {
                    // Registered first
                    if let Some(idx) = self.registered_names.iter().position(|n| n == &name) {
                        r.registered.get(idx).copied()
                    } else {
                        r.adhoc.get(&name).copied()
                    }
                });
                (name, value)
            })
            .collect()
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
