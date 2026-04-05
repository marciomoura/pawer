//! Discrete derivative (backward difference).
//!
//! Discrete-time equation: `y[k] = (u[k] - u[k-1]) / Ts`

use crate::types::Real;

/// First-order backward-difference derivative.
pub struct Derivative {
    sampling_time: Real,
    previous_input: Real,
    output: Real,
}

impl Derivative {
    /// Create a new derivative block with zero initial previous-input.
    pub fn new(sampling_time: Real) -> Self {
        assert!(sampling_time > 0.0, "Sampling time must be positive");
        Self {
            sampling_time,
            previous_input: 0.0,
            output: 0.0,
        }
    }

    /// Create a new derivative block with a specified initial previous-input.
    pub fn with_initial(sampling_time: Real, initial_value: Real) -> Self {
        assert!(sampling_time > 0.0, "Sampling time must be positive");
        Self {
            sampling_time,
            previous_input: initial_value,
            output: 0.0,
        }
    }

    /// Reconfigure the sampling period.
    pub fn configure_sampling_time(&mut self, sampling_time: Real) {
        assert!(sampling_time > 0.0, "Sampling time must be positive");
        self.sampling_time = sampling_time;
    }

    /// Compute the derivative for the current sample.
    pub fn update(&mut self, input: Real) {
        self.output = (input - self.previous_input) / self.sampling_time;
        self.previous_input = input;
    }

    /// Reset the internal state: set previous input to `value` and output to 0.
    pub fn reset(&mut self, value: Real) {
        self.previous_input = value;
        self.output = 0.0;
    }

    /// Return the most recently computed derivative value.
    pub fn output(&self) -> Real {
        self.output
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.001;
    const EPS: Real = 1e-3;

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn step_input_derivative() {
        let mut d = Derivative::new(TS);
        // First sample: step from 0 to 5
        d.update(5.0);
        assert!(approx_eq(d.output(), 5.0 / TS));
    }

    #[test]
    fn constant_input_gives_zero_derivative() {
        let mut d = Derivative::with_initial(TS, 3.0);
        for _ in 0..10 {
            d.update(3.0);
        }
        assert!(approx_eq(d.output(), 0.0));
    }

    #[test]
    fn ramp_input_gives_constant_derivative() {
        let mut d = Derivative::new(TS);
        let slope: Real = 2.0;
        // Feed a ramp: u[k] = slope * k * Ts
        for k in 0..100 {
            let input = slope * (k as Real) * TS;
            d.update(input);
        }
        // After settling, derivative should be close to `slope`
        assert!(approx_eq(d.output(), slope));
    }

    #[test]
    fn reset_then_update() {
        let mut d = Derivative::new(TS);
        d.update(10.0);
        d.reset(10.0);
        assert!(approx_eq(d.output(), 0.0));
        // Next input same as reset value → derivative 0
        d.update(10.0);
        assert!(approx_eq(d.output(), 0.0));
        // Next input different
        d.update(20.0);
        assert!(approx_eq(d.output(), 10.0 / TS));
    }

    #[test]
    fn negative_slope() {
        let mut d = Derivative::new(TS);
        d.update(0.0);
        d.update(-5.0);
        assert!(approx_eq(d.output(), -5.0 / TS));
    }

    #[test]
    fn with_initial_sets_previous_input() {
        let mut d = Derivative::with_initial(TS, 100.0);
        d.update(100.0);
        assert!(approx_eq(d.output(), 0.0));
    }

    #[test]
    fn configure_sampling_time_works() {
        let mut d = Derivative::new(TS);
        d.configure_sampling_time(0.01);
        d.update(0.0);
        d.update(1.0);
        assert!(approx_eq(d.output(), 1.0 / 0.01));
    }

    #[test]
    #[should_panic]
    fn new_rejects_zero_sampling_time() {
        let _ = Derivative::new(0.0);
    }
}
