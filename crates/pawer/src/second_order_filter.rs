//! Second-order IIR filters (low-pass, band-reject / notch, high-pass).
//!
//! Difference equation:
//! ```text
//! y[n] = -a1*y[n-1] - a2*y[n-2] + b0*x[n] + b1*x[n-1] + b2*x[n-2]
//! ```
//!
//! Coefficients are computed from the RBJ Audio EQ Cookbook formulae.
//! Internal arithmetic uses `f64`; the public interface uses [`Real`] (`f32`).

use crate::types::Real;

const PI_F64: f64 = core::f64::consts::PI;

// ── Common second-order IIR state ────────────────────────────────────────────

/// Internal IIR state shared by all second-order filter variants.
struct SecondOrderState {
    // Coefficients
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    // Previous inputs
    x1: f64,
    x2: f64,
    // Previous outputs
    y1: f64,
    y2: f64,
    // Current output
    output: f64,
    // Sampling period
    ts: f64,
}

impl SecondOrderState {
    fn new(sampling_period: Real) -> Self {
        Self {
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            output: 0.0,
            ts: sampling_period as f64,
        }
    }

    fn update(&mut self, input: Real) -> Real {
        let inp = input as f64;
        let out = -self.a1 * self.y1 - self.a2 * self.y2
            + self.b0 * inp
            + self.b1 * self.x1
            + self.b2 * self.x2;
        self.x2 = self.x1;
        self.x1 = inp;
        self.y2 = self.y1;
        self.y1 = out;
        self.output = out;
        out as Real
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
        self.output = 0.0;
    }

    fn reset_to(&mut self, value: Real) {
        let v = value as f64;
        self.x1 = v;
        self.x2 = v;
        self.y1 = v;
        self.y2 = v;
        self.output = v;
    }

    fn output(&self) -> Real {
        self.output as Real
    }
}

// ── Second-order low-pass filter ─────────────────────────────────────────────

/// Second-order (bi-quad) low-pass filter using RBJ cookbook coefficients.
pub struct SecondOrderLowPassFilter {
    state: SecondOrderState,
}

impl SecondOrderLowPassFilter {
    /// Create an unconfigured LPF. Call [`configure`](Self::configure) before
    /// use.
    pub fn new(sampling_period: Real) -> Self {
        assert!(sampling_period > 0.0, "Sampling period must be positive");
        Self {
            state: SecondOrderState::new(sampling_period),
        }
    }

    /// Set up the filter for the given cutoff frequency (Hz) and damping ratio.
    ///
    /// `damping_ratio` maps to the RBJ `Q` parameter — for a Butterworth
    /// response use `1 / √2 ≈ 0.7071`.
    pub fn configure(&mut self, cutoff_freq_hz: f64, damping_ratio: f64) {
        let fs = 1.0 / self.state.ts;
        let omega = 2.0 * PI_F64 * cutoff_freq_hz / fs;
        let cos_omega = libm::cos(omega);
        let sin_omega = libm::sin(omega);
        let alpha = sin_omega / (2.0 * damping_ratio);

        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;

        self.state.b0 = b0 / a0;
        self.state.b1 = b1 / a0;
        self.state.b2 = b2 / a0;
        self.state.a1 = -2.0 * cos_omega / a0;
        self.state.a2 = (1.0 - alpha) / a0;
    }

    /// Process one input sample and return the filtered output.
    pub fn update(&mut self, input: Real) -> Real {
        self.state.update(input)
    }

    /// Return the most recent output without advancing.
    pub fn output(&self) -> Real {
        self.state.output()
    }

    /// Reset all internal state to zero.
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Reset all internal state so that input and output history equal `value`.
    pub fn reset_to(&mut self, value: Real) {
        self.state.reset_to(value);
    }
}

// ── Second-order band-reject (notch) filter ──────────────────────────────────

/// Second-order band-reject (notch) filter using RBJ cookbook coefficients.
pub struct SecondOrderBandRejectFilter {
    state: SecondOrderState,
}

impl SecondOrderBandRejectFilter {
    /// Create an unconfigured notch filter.
    pub fn new(sampling_period: Real) -> Self {
        assert!(sampling_period > 0.0, "Sampling period must be positive");
        Self {
            state: SecondOrderState::new(sampling_period),
        }
    }

    /// Configure for a given center frequency (Hz) and 3 dB bandwidth (Hz).
    pub fn configure(&mut self, center_freq_hz: f64, bandwidth_hz: f64) {
        let fs = 1.0 / self.state.ts;
        let omega = 2.0 * PI_F64 * center_freq_hz / fs;
        let q = center_freq_hz / bandwidth_hz;
        let sin_omega = libm::sin(omega);
        let cos_omega = libm::cos(omega);
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0;
        let a0 = 1.0 + alpha;

        self.state.b0 = b0 / a0;
        self.state.b1 = b1 / a0;
        self.state.b2 = b2 / a0;
        self.state.a1 = b1 / a0; // same as b1/a0 for notch
        self.state.a2 = (1.0 - alpha) / a0;
    }

    /// Process one input sample and return the filtered output.
    pub fn update(&mut self, input: Real) -> Real {
        self.state.update(input)
    }

    /// Return the most recent output without advancing.
    pub fn output(&self) -> Real {
        self.state.output()
    }

    /// Reset all internal state to zero.
    pub fn reset(&mut self) {
        self.state.reset();
    }
}

// ── Second-order high-pass filter ────────────────────────────────────────────

/// Second-order (bi-quad) high-pass filter using RBJ cookbook coefficients.
pub struct SecondOrderHighPassFilter {
    state: SecondOrderState,
}

impl SecondOrderHighPassFilter {
    /// Create an unconfigured HPF.
    pub fn new(sampling_period: Real) -> Self {
        assert!(sampling_period > 0.0, "Sampling period must be positive");
        Self {
            state: SecondOrderState::new(sampling_period),
        }
    }

    /// Set up the filter for the given cutoff frequency (Hz) and damping ratio.
    pub fn configure(&mut self, cutoff_freq_hz: f64, damping_ratio: f64) {
        let fs = 1.0 / self.state.ts;
        let omega = 2.0 * PI_F64 * cutoff_freq_hz / fs;
        let cos_omega = libm::cos(omega);
        let sin_omega = libm::sin(omega);
        let alpha = sin_omega / (2.0 * damping_ratio);

        let b0 = (1.0 + cos_omega) / 2.0;
        let b1 = -(1.0 + cos_omega);
        let b2 = (1.0 + cos_omega) / 2.0;
        let a0 = 1.0 + alpha;

        self.state.b0 = b0 / a0;
        self.state.b1 = b1 / a0;
        self.state.b2 = b2 / a0;
        self.state.a1 = -2.0 * cos_omega / a0;
        self.state.a2 = (1.0 - alpha) / a0;
    }

    /// Process one input sample and return the filtered output.
    pub fn update(&mut self, input: Real) -> Real {
        self.state.update(input)
    }

    /// Return the most recent output without advancing.
    pub fn output(&self) -> Real {
        self.state.output()
    }

    /// Reset all internal state to zero.
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Reset all internal state so that input and output history equal `value`.
    pub fn reset_to(&mut self, value: Real) {
        self.state.reset_to(value);
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.0001; // 10 kHz sampling
    const EPS: Real = 1e-2;

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < EPS
    }

    // ── Low-pass filter ──────────────────────────────────────────────────

    #[test]
    fn lpf_dc_passes_through() {
        let mut f = SecondOrderLowPassFilter::new(TS);
        f.configure(500.0, 0.7071);
        for _ in 0..20_000 {
            f.update(1.0);
        }
        assert!(
            approx_eq(f.output(), 1.0),
            "Expected ~1.0, got {}",
            f.output()
        );
    }

    #[test]
    fn lpf_high_freq_attenuation() {
        let mut f = SecondOrderLowPassFilter::new(TS);
        f.configure(50.0, 0.7071); // low cutoff
        // Alternating +1/-1 is at Nyquist — should be heavily attenuated
        for i in 0..5000 {
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
    fn lpf_reset_to_value() {
        let mut f = SecondOrderLowPassFilter::new(TS);
        f.configure(100.0, 0.7071);
        f.update(5.0);
        f.reset_to(3.0);
        assert!(approx_eq(f.output(), 3.0));
    }

    #[test]
    fn lpf_reset_to_zero() {
        let mut f = SecondOrderLowPassFilter::new(TS);
        f.configure(100.0, 0.7071);
        for _ in 0..100 {
            f.update(10.0);
        }
        f.reset();
        assert!(approx_eq(f.output(), 0.0));
    }

    #[test]
    fn lpf_step_response_rises_gradually() {
        let mut f = SecondOrderLowPassFilter::new(TS);
        f.configure(100.0, 0.7071);
        f.update(1.0);
        let first = f.output();
        assert!(first > 0.0 && first < 1.0);

        for _ in 0..200 {
            f.update(1.0);
        }
        let later = f.output();
        assert!(later > first);
    }

    // ── Band-reject (notch) filter ───────────────────────────────────────

    #[test]
    fn notch_dc_passes_through() {
        let mut f = SecondOrderBandRejectFilter::new(TS);
        f.configure(1000.0, 100.0); // notch at 1 kHz
        for _ in 0..20_000 {
            f.update(1.0);
        }
        assert!(
            approx_eq(f.output(), 1.0),
            "Expected ~1.0, got {}",
            f.output()
        );
    }

    #[test]
    fn notch_attenuates_center_frequency() {
        let mut f = SecondOrderBandRejectFilter::new(TS);
        let center_hz = 1000.0;
        f.configure(center_hz, 50.0); // narrow notch

        // Feed a sine at the center frequency
        let omega = 2.0 * core::f64::consts::PI * center_hz;
        let mut max_out: Real = 0.0;
        // Let transient settle first
        for k in 0..20_000 {
            let t = k as f64 * TS as f64;
            let input = libm::sin(omega * t) as Real;
            f.update(input);
            if k > 15_000 {
                let abs = if f.output() < 0.0 {
                    -f.output()
                } else {
                    f.output()
                };
                if abs > max_out {
                    max_out = abs;
                }
            }
        }
        assert!(
            max_out < 0.15,
            "Expected strong attenuation at center freq, peak = {}",
            max_out
        );
    }

    #[test]
    fn notch_reset() {
        let mut f = SecondOrderBandRejectFilter::new(TS);
        f.configure(500.0, 100.0);
        for _ in 0..100 {
            f.update(5.0);
        }
        f.reset();
        assert!(approx_eq(f.output(), 0.0));
    }

    // ── High-pass filter ─────────────────────────────────────────────────

    #[test]
    fn hpf_dc_blocked() {
        let mut f = SecondOrderHighPassFilter::new(TS);
        f.configure(500.0, 0.7071);
        for _ in 0..20_000 {
            f.update(1.0);
        }
        assert!(f.output().abs() < 0.01, "Expected ~0.0, got {}", f.output());
    }

    #[test]
    fn hpf_passes_high_frequency() {
        let mut f = SecondOrderHighPassFilter::new(TS);
        f.configure(50.0, 0.7071); // low cutoff — passes most frequencies

        // Feed a high-frequency sine well above cutoff
        let freq_hz = 2000.0;
        let omega = 2.0 * core::f64::consts::PI * freq_hz;
        let mut max_out: Real = 0.0;
        for k in 0..10_000 {
            let t = k as f64 * TS as f64;
            let input = libm::sin(omega * t) as Real;
            f.update(input);
            if k > 5_000 {
                let abs = if f.output() < 0.0 {
                    -f.output()
                } else {
                    f.output()
                };
                if abs > max_out {
                    max_out = abs;
                }
            }
        }
        assert!(
            max_out > 0.8,
            "Expected high-freq passthrough, peak = {}",
            max_out
        );
    }

    #[test]
    fn hpf_reset_works() {
        let mut f = SecondOrderHighPassFilter::new(TS);
        f.configure(100.0, 0.7071);
        for _ in 0..100 {
            f.update(5.0);
        }
        f.reset();
        assert!(approx_eq(f.output(), 0.0));
    }

    #[test]
    fn hpf_reset_to_value() {
        let mut f = SecondOrderHighPassFilter::new(TS);
        f.configure(100.0, 0.7071);
        f.reset_to(7.0);
        assert!(approx_eq(f.output(), 7.0));
    }

    // ── Constructor panics ───────────────────────────────────────────────

    #[test]
    #[should_panic]
    fn lpf_rejects_zero_ts() {
        let _ = SecondOrderLowPassFilter::new(0.0);
    }

    #[test]
    #[should_panic]
    fn notch_rejects_zero_ts() {
        let _ = SecondOrderBandRejectFilter::new(0.0);
    }

    #[test]
    #[should_panic]
    fn hpf_rejects_zero_ts() {
        let _ = SecondOrderHighPassFilter::new(0.0);
    }
}
