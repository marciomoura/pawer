//! Discrete-time PI controller with anti-windup for power electronic
//! control applications.
//!
//! The controller output is:
//! ```text
//! u[k] = Kp·e[k] + Ki·Ts·Σe[i]
//! ```
//!
//! Anti-windup is implemented by clamping the integral state to
//! `[output_min, output_max]` whenever the total output is saturated.

/// Discrete-time PI controller with output saturation and anti-windup.
///
/// # Example
/// ```
/// use pawer::controllers::PiController;
///
/// let mut pi = PiController::new(1.0, 10.0, 1e-4, -100.0, 100.0);
/// let output = pi.update(1.0); // step reference error = 1.0
/// assert!(output > 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct PiController {
    kp: f64,
    ki: f64,
    sample_time: f64,
    output_min: f64,
    output_max: f64,
    integral: f64,
}

impl PiController {
    /// Create a new PI controller.
    ///
    /// # Arguments
    /// * `kp`          – Proportional gain (Kp).
    /// * `ki`          – Integral gain (Ki, in rad/s).
    /// * `sample_time` – Sampling period in **seconds** (Ts > 0).
    /// * `output_min`  – Lower saturation limit on the controller output.
    /// * `output_max`  – Upper saturation limit on the controller output.
    ///
    /// # Panics
    /// Panics when `sample_time` is not strictly positive, or when
    /// `output_min >= output_max`.
    pub fn new(kp: f64, ki: f64, sample_time: f64, output_min: f64, output_max: f64) -> Self {
        assert!(sample_time > 0.0, "sample_time must be > 0");
        assert!(
            output_min < output_max,
            "output_min must be less than output_max"
        );

        Self {
            kp,
            ki,
            sample_time,
            output_min,
            output_max,
            integral: 0.0,
        }
    }

    /// Compute the controller output for the current error sample.
    ///
    /// The integral state is updated **before** clamping (forward Euler), then
    /// the integral is back-calculated so that the unsaturated proportional term
    /// plus the integral stays within `[output_min, output_max]` (anti-windup).
    pub fn update(&mut self, error: f64) -> f64 {
        // Update integral (forward Euler).
        self.integral += self.ki * self.sample_time * error;

        // Compute unsaturated output.
        let output_unsat = self.kp * error + self.integral;

        // Saturate and apply anti-windup by back-calculating the integral.
        let output = output_unsat.clamp(self.output_min, self.output_max);
        if (output - output_unsat).abs() > f64::EPSILON {
            self.integral = output - self.kp * error;
        }

        output
    }

    /// Reset the integral state to zero.
    pub fn reset(&mut self) {
        self.integral = 0.0;
    }

    /// Pre-load the integral state (useful for bumpless transfer).
    pub fn set_integral(&mut self, value: f64) {
        self.integral = value.clamp(self.output_min, self.output_max);
    }

    /// Return the current integral state (read-only).
    pub fn integral(&self) -> f64 {
        self.integral
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With a constant error and no saturation, the output must grow
    /// proportionally to time (PI response to step error).
    #[test]
    fn pi_step_response_increases_over_time() {
        let mut pi = PiController::new(1.0, 10.0, 1e-4, -1000.0, 1000.0);
        let out1 = pi.update(1.0);
        let out2 = pi.update(1.0);
        assert!(out2 > out1, "output should increase with integral action");
    }

    /// The output must never exceed the saturation limits.
    #[test]
    fn pi_saturation() {
        let mut pi = PiController::new(1.0, 100.0, 1e-4, -10.0, 10.0);
        for _ in 0..100_000 {
            let out = pi.update(1.0);
            assert!(
                (-10.0..=10.0).contains(&out),
                "output {out} outside [-10, 10]"
            );
        }
    }

    /// Anti-windup: after saturation, the integral must not grow unboundedly.
    #[test]
    fn pi_anti_windup() {
        let mut pi = PiController::new(0.0, 100.0, 1e-4, -5.0, 5.0);
        // Drive into saturation.
        for _ in 0..100_000 {
            pi.update(1.0);
        }
        // Reverse the error – the output should recover quickly.
        let out = pi.update(-1.0);
        assert!(
            out < 5.0,
            "anti-windup should limit integral wind-up, got {out}"
        );
    }

    /// A zero error must keep the integral unchanged.
    #[test]
    fn pi_zero_error_keeps_integral() {
        let mut pi = PiController::new(1.0, 10.0, 1e-4, -100.0, 100.0);
        pi.update(1.0); // build up some integral
        let integral_before = pi.integral();
        pi.update(0.0);
        assert_eq!(
            pi.integral(),
            integral_before,
            "zero error must not change the integral"
        );
    }

    /// `reset()` must bring the integral back to zero.
    #[test]
    fn pi_reset() {
        let mut pi = PiController::new(1.0, 10.0, 1e-4, -100.0, 100.0);
        for _ in 0..100 {
            pi.update(1.0);
        }
        pi.reset();
        assert_eq!(pi.integral(), 0.0, "integral must be zero after reset");
    }

    /// `set_integral` pre-loads the integral state.
    #[test]
    fn pi_set_integral() {
        let mut pi = PiController::new(0.0, 1.0, 1e-4, -100.0, 100.0);
        pi.set_integral(7.5);
        let out = pi.update(0.0);
        assert!(
            (out - 7.5).abs() < 1e-10,
            "output should reflect pre-loaded integral"
        );
    }
}
