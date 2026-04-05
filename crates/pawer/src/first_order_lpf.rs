//! First-order low-pass filter using the bilinear (Tustin) transform.
//!
//! Continuous transfer function: `H(s) = ωc / (s + ωc)`
//!
//! Internal state is kept in `f64` for numerical stability; the public
//! interface uses [`Real`] (`f32`).

use crate::types::Real;

/// Bilinear-transform (Tustin) first-order low-pass filter.
pub struct FirstOrderLowPassFilter {
    // Difference-equation coefficients (f64 for precision)
    a1: f64,
    b0: f64,
    b1: f64,
    // State
    prev_input: f64,
    output_internal: f64,
    // Configuration
    ts: f64,
}

impl FirstOrderLowPassFilter {
    /// Create an unconfigured filter. Call [`configure`](Self::configure) to
    /// set the cutoff frequency before use.
    pub fn new(sampling_period: Real) -> Self {
        assert!(sampling_period > 0.0, "Sampling period must be positive");
        Self {
            a1: 0.0,
            b0: 0.0,
            b1: 0.0,
            prev_input: 0.0,
            output_internal: 0.0,
            ts: sampling_period as f64,
        }
    }

    /// Configure the cutoff frequency (in Hz).
    ///
    /// Internally pre-warps the analog frequency via the bilinear transform so
    /// the digital filter matches at the specified cutoff.
    pub fn configure(&mut self, cutoff_freq_hz: Real) {
        let fc = cutoff_freq_hz as f64;
        let omega_c = 2.0 * core::f64::consts::PI * fc;
        let omega_w = 2.0 / self.ts * libm::tan(omega_c * self.ts * 0.5);
        let denom = omega_w * self.ts + 2.0;
        self.b0 = omega_w * self.ts / denom;
        self.b1 = self.b0;
        self.a1 = (omega_w * self.ts - 2.0) / denom;
    }

    /// Process one input sample and return the filtered output.
    pub fn update(&mut self, input: Real) -> Real {
        let inp = input as f64;
        let out = -self.a1 * self.output_internal + self.b0 * inp + self.b1 * self.prev_input;
        self.prev_input = inp;
        self.output_internal = out;
        out as Real
    }

    /// Return the most recent output without advancing.
    pub fn output(&self) -> Real {
        self.output_internal as Real
    }

    /// Reset internal state so that both the stored input and output equal
    /// `value`. This avoids the transient that would otherwise occur after a
    /// step change in operating point.
    pub fn reset(&mut self, value: Real) {
        let v = value as f64;
        self.prev_input = v;
        self.output_internal = v;
    }

    /// Return the configured sampling period.
    pub fn sampling_time(&self) -> Real {
        self.ts as Real
    }

    /// Helper: convert a time-constant τ (seconds) to a cutoff frequency
    /// (Hz): `fc = 1 / (2π τ)`.
    pub fn cutoff_from_time_constant(time_constant: Real) -> Real {
        if time_constant <= 0.0 {
            return 0.0;
        }
        1.0 / (2.0 * core::f32::consts::PI * time_constant)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.0001; // 10 kHz sampling
    const EPS: Real = 1e-3;

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn dc_input_converges_to_input() {
        let mut f = FirstOrderLowPassFilter::new(TS);
        f.configure(100.0); // 100 Hz cutoff
        for _ in 0..10_000 {
            f.update(1.0);
        }
        assert!(
            approx_eq(f.output(), 1.0),
            "Expected ~1.0, got {}",
            f.output()
        );
    }

    #[test]
    fn step_response_gradually_approaches() {
        let mut f = FirstOrderLowPassFilter::new(TS);
        f.configure(100.0);
        f.update(1.0);
        let first = f.output();
        // After one sample the output should be between 0 and 1
        assert!(first > 0.0 && first < 1.0, "first = {}", first);

        for _ in 0..500 {
            f.update(1.0);
        }
        let mid = f.output();
        // Should be closer to 1 than the first sample
        assert!(mid > first, "mid = {}", mid);
    }

    #[test]
    fn reset_sets_output() {
        let mut f = FirstOrderLowPassFilter::new(TS);
        f.configure(50.0);
        f.reset(5.0);
        assert!(approx_eq(f.output(), 5.0));
    }

    #[test]
    fn cutoff_from_time_constant_10hz() {
        // τ = 1 / (2π * 10) ≈ 0.015915
        let tau = 1.0 / (2.0 * core::f32::consts::PI * 10.0);
        let fc = FirstOrderLowPassFilter::cutoff_from_time_constant(tau);
        assert!(approx_eq(fc, 10.0), "Expected ~10 Hz, got {}", fc);
    }

    #[test]
    fn cutoff_from_zero_time_constant_returns_zero() {
        assert!(approx_eq(
            FirstOrderLowPassFilter::cutoff_from_time_constant(0.0),
            0.0
        ));
    }

    #[test]
    fn cutoff_from_negative_time_constant_returns_zero() {
        assert!(approx_eq(
            FirstOrderLowPassFilter::cutoff_from_time_constant(-1.0),
            0.0
        ));
    }

    #[test]
    fn filter_attenuates_high_frequency() {
        let mut f = FirstOrderLowPassFilter::new(TS);
        f.configure(10.0); // very low cutoff
        // Feed alternating +1 / -1 (high-frequency content)
        for i in 0..2000 {
            let input = if i % 2 == 0 { 1.0 } else { -1.0 };
            f.update(input);
        }
        assert!(
            f.output().abs() < 0.05,
            "Expected near-zero, got {}",
            f.output()
        );
    }

    #[test]
    fn sampling_time_accessor() {
        let f = FirstOrderLowPassFilter::new(0.001);
        assert!(approx_eq(f.sampling_time(), 0.001));
    }

    #[test]
    #[should_panic]
    fn new_rejects_zero_sampling_period() {
        let _ = FirstOrderLowPassFilter::new(0.0);
    }
}
