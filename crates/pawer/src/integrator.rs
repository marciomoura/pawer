//! Simple rectangular (forward Euler) integrator.
//!
//! Discrete-time equation: `y[k] = y[k-1] + x[k] * Ts`

use crate::types::Real;

/// Rectangular integrator that accumulates its input scaled by the sampling
/// period.
pub struct Integrator {
    sampling_time: Real,
    value: Real,
}

impl Integrator {
    /// Create a new integrator with the given sampling period (must be > 0).
    pub fn new(sampling_time: Real) -> Self {
        assert!(sampling_time > 0.0, "Sampling time must be positive");
        Self {
            sampling_time,
            value: 0.0,
        }
    }

    /// Reconfigure the sampling period.
    pub fn configure(&mut self, sampling_time: Real) {
        assert!(sampling_time > 0.0, "Sampling time must be positive");
        self.sampling_time = sampling_time;
    }

    /// Reset the integrator state to an arbitrary value.
    pub fn reset(&mut self, value: Real) {
        self.value = value;
    }

    /// Reset the integrator state to zero.
    pub fn reset_to_zero(&mut self) {
        self.value = 0.0;
    }

    /// Advance one sample: `y[k] = y[k-1] + input * Ts`.
    pub fn update(&mut self, input: Real) -> Real {
        self.value += self.sampling_time * input;
        self.value
    }

    /// Return the current accumulated value without advancing.
    pub fn output(&self) -> Real {
        self.value
    }

    /// Return the configured sampling period.
    pub fn sampling_time(&self) -> Real {
        self.sampling_time
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.001; // 1 kHz
    const EPS: Real = 1e-5;

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn default_reset_output_is_zero() {
        let integrator = Integrator::new(TS);
        assert!(approx_eq(integrator.output(), 0.0));
    }

    #[test]
    fn constant_input_produces_linear_ramp() {
        let mut integrator = Integrator::new(TS);
        let input = 10.0;
        let steps = 100;
        for _ in 0..steps {
            integrator.update(input);
        }
        let expected = input * TS * steps as Real;
        assert!(approx_eq(integrator.output(), expected));
    }

    #[test]
    fn reset_to_value() {
        let mut integrator = Integrator::new(TS);
        integrator.update(5.0);
        integrator.reset(42.0);
        assert!(approx_eq(integrator.output(), 42.0));
    }

    #[test]
    fn zero_input_holds_value() {
        let mut integrator = Integrator::new(TS);
        integrator.reset(7.5);
        for _ in 0..50 {
            integrator.update(0.0);
        }
        assert!(approx_eq(integrator.output(), 7.5));
    }

    #[test]
    fn negative_input_decreases_output() {
        let mut integrator = Integrator::new(TS);
        for _ in 0..100 {
            integrator.update(-5.0);
        }
        let expected = -5.0 * TS * 100.0;
        assert!(approx_eq(integrator.output(), expected));
    }

    #[test]
    fn reset_to_zero_clears_state() {
        let mut integrator = Integrator::new(TS);
        integrator.update(100.0);
        integrator.reset_to_zero();
        assert!(approx_eq(integrator.output(), 0.0));
    }

    #[test]
    fn configure_changes_sampling_time() {
        let mut integrator = Integrator::new(TS);
        let new_ts: Real = 0.01;
        integrator.configure(new_ts);
        assert!(approx_eq(integrator.sampling_time(), new_ts));
        integrator.update(1.0);
        assert!(approx_eq(integrator.output(), new_ts));
    }

    #[test]
    #[should_panic]
    fn new_rejects_zero_sampling_time() {
        let _ = Integrator::new(0.0);
    }

    #[test]
    #[should_panic]
    fn new_rejects_negative_sampling_time() {
        let _ = Integrator::new(-1.0);
    }
}
