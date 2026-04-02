//! Discrete-time integrator for power electronic control applications.
//!
//! Implements a simple **forward-Euler** integrator with optional
//! output clamping (saturation limits).
//!
//! ```text
//! y[k] = y[k-1] + Ts · x[k-1]   (forward Euler)
//! ```

/// Discrete-time forward-Euler integrator with optional saturation.
///
/// # Example
/// ```
/// use pawer::integrator::Integrator;
///
/// let mut integ = Integrator::new(1e-4, Some(-10.0), Some(10.0));
/// let out = integ.update(1.0); // integrates a constant 1.0
/// assert!(out >= 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct Integrator {
    sample_time: f64,
    min: Option<f64>,
    max: Option<f64>,
    state: f64,
}

impl Integrator {
    /// Create a new integrator.
    ///
    /// # Arguments
    /// * `sample_time` – Sampling period in **seconds** (Ts > 0).
    /// * `min`         – Optional lower saturation limit.
    /// * `max`         – Optional upper saturation limit.
    ///
    /// # Panics
    /// Panics when `sample_time` is not strictly positive, or when `min >= max`
    /// (if both limits are provided).
    pub fn new(sample_time: f64, min: Option<f64>, max: Option<f64>) -> Self {
        assert!(sample_time > 0.0, "sample_time must be > 0");
        if let (Some(lo), Some(hi)) = (min, max) {
            assert!(lo < hi, "min must be less than max");
        }
        Self {
            sample_time,
            min,
            max,
            state: 0.0,
        }
    }

    /// Process one sample and return the integrated output.
    pub fn update(&mut self, input: f64) -> f64 {
        self.state += self.sample_time * input;
        self.state = self.clamp(self.state);
        self.state
    }

    /// Reset the integrator state to zero.
    pub fn reset(&mut self) {
        self.state = 0.0;
    }

    /// Set the integrator state to an arbitrary initial value (after clamping).
    pub fn set_state(&mut self, value: f64) {
        self.state = self.clamp(value);
    }

    fn clamp(&self, value: f64) -> f64 {
        let mut v = value;
        if let Some(lo) = self.min {
            if v < lo {
                v = lo;
            }
        }
        if let Some(hi) = self.max {
            if v > hi {
                v = hi;
            }
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Integrating a constant for N steps must yield N·Ts·input.
    #[test]
    fn integrator_constant_input() {
        let ts = 1e-3;
        let steps = 100;
        let input = 5.0;
        let mut integ = Integrator::new(ts, None, None);
        let mut out = 0.0;
        for _ in 0..steps {
            out = integ.update(input);
        }
        let expected = ts * input * steps as f64;
        assert!(
            (out - expected).abs() < 1e-10,
            "expected {expected}, got {out}"
        );
    }

    /// The output must not exceed the maximum saturation limit.
    #[test]
    fn integrator_saturation_max() {
        let mut integ = Integrator::new(1e-3, None, Some(1.0));
        let mut out = 0.0;
        for _ in 0..10_000 {
            out = integ.update(1.0);
        }
        assert!(
            (out - 1.0).abs() < 1e-10,
            "saturated output should be 1.0, got {out}"
        );
    }

    /// The output must not fall below the minimum saturation limit.
    #[test]
    fn integrator_saturation_min() {
        let mut integ = Integrator::new(1e-3, Some(-1.0), None);
        let mut out = 0.0;
        for _ in 0..10_000 {
            out = integ.update(-1.0);
        }
        assert!(
            (out + 1.0).abs() < 1e-10,
            "saturated output should be -1.0, got {out}"
        );
    }

    /// `reset()` must return the state to zero.
    #[test]
    fn integrator_reset() {
        let mut integ = Integrator::new(1e-3, None, None);
        for _ in 0..50 {
            integ.update(1.0);
        }
        integ.reset();
        let out = integ.update(0.0);
        assert_eq!(out, 0.0, "state after reset must be 0");
    }

    /// `set_state` must pre-load the integrator with the given value.
    #[test]
    fn integrator_set_state() {
        let mut integ = Integrator::new(1e-3, None, None);
        integ.set_state(42.0);
        let out = integ.update(0.0);
        assert_eq!(out, 42.0, "state should be pre-loaded to 42.0");
    }
}
