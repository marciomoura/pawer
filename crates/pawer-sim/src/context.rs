use std::collections::HashMap;

use pawer::types::Real;

/// Simulation context passed to [`Scenario`](crate::Scenario) methods.
///
/// Provides access to:
/// - **Parameters**: user-settable values (via `/set` in the CLI)
/// - **Signal logging**: record named signals each step
/// - **Time info**: current simulation time and sampling period
pub struct SimContext {
    params: HashMap<String, f64>,
    step_buffer: HashMap<String, Real>,
    time: f64,
    dt: f64,
}

impl SimContext {
    pub(crate) fn new(dt: f64) -> Self {
        Self {
            params: HashMap::new(),
            step_buffer: HashMap::new(),
            time: 0.0,
            dt,
        }
    }

    // ── Parameter access ──────────────────────────────────────────────

    /// Set a named parameter. Called by the scenario in `init()` to declare
    /// defaults, or by the CLI via `/set`.
    pub fn set_param(&mut self, name: &str, value: f64) {
        self.params.insert(name.to_owned(), value);
    }

    /// Read a parameter as `Real`. Returns `0.0` if the parameter is not set.
    pub fn param(&self, name: &str) -> Real {
        self.params.get(name).copied().unwrap_or(0.0) as Real
    }

    /// Read a parameter as `f64`. Returns `0.0` if the parameter is not set.
    pub fn param_f64(&self, name: &str) -> f64 {
        self.params.get(name).copied().unwrap_or(0.0)
    }

    /// List all parameter names.
    pub fn param_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.params.keys().cloned().collect();
        names.sort();
        names
    }

    // ── Signal logging ────────────────────────────────────────────────

    /// Log a named signal value for the current step.
    pub fn log(&mut self, name: &str, value: Real) {
        self.step_buffer.insert(name.to_owned(), value);
    }

    // ── Time access ───────────────────────────────────────────────────

    /// Current simulation time in seconds.
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Sampling period (Δt) in seconds.
    pub fn dt(&self) -> Real {
        self.dt as Real
    }

    /// Sampling period as `f64`.
    pub fn dt_f64(&self) -> f64 {
        self.dt
    }

    // ── Internal helpers ──────────────────────────────────────────────

    pub(crate) fn advance_time(&mut self) {
        self.time += self.dt;
    }

    pub(crate) fn take_step_buffer(&mut self) -> HashMap<String, Real> {
        std::mem::take(&mut self.step_buffer)
    }

    pub(crate) fn reset_time(&mut self) {
        self.time = 0.0;
    }

    pub(crate) fn clear_params(&mut self) {
        self.params.clear();
    }
}
