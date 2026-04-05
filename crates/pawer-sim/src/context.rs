use std::collections::HashMap;

use pawer::types::Real;

/// Handle to a registered signal, used for fast indexed logging.
///
/// Obtained from [`SimContext::register_signal`] during [`Scenario::init`].
/// Pass to [`SimContext::log_id`] in the step function for allocation-free logging.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SignalId(pub(crate) usize);

/// Data collected from one simulation step (both registered and ad-hoc signals).
pub(crate) struct StepData {
    pub registered: Vec<Real>,
    pub adhoc: HashMap<String, Real>,
}

/// Simulation context passed to [`Scenario`](crate::Scenario) methods.
///
/// Provides access to:
/// - **Parameters**: user-settable values (via `/set` in the CLI)
/// - **Signal logging**: register signals upfront, then log by index for performance
/// - **Time info**: current simulation time and sampling period
pub struct SimContext {
    params: HashMap<String, f64>,

    // Registered signals (fast path)
    registered_names: Vec<String>,
    registered_buffer: Vec<Real>,

    // Ad-hoc signals (backward compat, slower)
    adhoc_buffer: HashMap<String, Real>,

    time: f64,
    dt: f64,
}

impl SimContext {
    pub(crate) fn new(dt: f64) -> Self {
        Self {
            params: HashMap::new(),
            registered_names: Vec::new(),
            registered_buffer: Vec::new(),
            adhoc_buffer: HashMap::new(),
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

    // ── Signal registration ───────────────────────────────────────────

    /// Register a named signal for fast indexed logging.
    ///
    /// Call during [`Scenario::init`] to declare every signal upfront.
    /// Returns a [`SignalId`] that can be passed to [`log_id`](Self::log_id)
    /// during each step — this avoids string allocation and HashMap lookups
    /// on the hot path.
    pub fn register_signal(&mut self, name: &str) -> SignalId {
        let id = SignalId(self.registered_names.len());
        self.registered_names.push(name.to_owned());
        self.registered_buffer.push(0.0);
        id
    }

    /// Log a registered signal by index. This is the fast path:
    /// a single indexed write into a flat `Vec`, no allocation.
    #[inline]
    pub fn log_id(&mut self, id: SignalId, value: Real) {
        self.registered_buffer[id.0] = value;
    }

    // ── Ad-hoc signal logging (backward compat) ───────────────────────

    /// Log a named signal value for the current step.
    ///
    /// This is the slower path kept for backward compatibility. Prefer
    /// [`register_signal`](Self::register_signal) + [`log_id`](Self::log_id)
    /// for scenarios with many signals.
    pub fn log(&mut self, name: &str, value: Real) {
        self.adhoc_buffer.insert(name.to_owned(), value);
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

    /// Take the current step's data (both registered and ad-hoc) and reset
    /// the ad-hoc buffer. The registered buffer is copied, not moved, so
    /// IDs remain valid. Values are reset to 0.0 for the next step.
    pub(crate) fn take_step_data(&mut self) -> StepData {
        let registered = self.registered_buffer.clone();
        // Reset registered buffer for the next step
        for v in &mut self.registered_buffer {
            *v = 0.0;
        }
        let adhoc = std::mem::take(&mut self.adhoc_buffer);
        StepData { registered, adhoc }
    }

    /// Names of all registered signals, in registration order.
    pub(crate) fn registered_signal_names(&self) -> &[String] {
        &self.registered_names
    }

    pub(crate) fn reset_time(&mut self) {
        self.time = 0.0;
    }

    pub(crate) fn clear_params(&mut self) {
        self.params.clear();
    }

    pub(crate) fn clear_signals(&mut self) {
        self.registered_names.clear();
        self.registered_buffer.clear();
        self.adhoc_buffer.clear();
    }
}
