//! Discrete-time filters for power electronic control applications.
//!
//! Implements a **first-order low-pass filter** and a
//! **second-order low-pass filter**, both discretised with the
//! bilinear (Tustin) transform.

/// First-order low-pass filter discretised via the bilinear transform.
///
/// Continuous-time transfer function:
/// ```text
/// H(s) = ωc / (s + ωc)
/// ```
///
/// Bilinear-transform difference equation:
/// ```text
/// y[k] = b0·x[k] + b1·x[k-1] − a1·y[k-1]
/// ```
///
/// where
/// ```text
/// K  = ωc · Ts / 2
/// b0 = b1 = K / (1 + K)
/// a1 = (K − 1) / (K + 1)
/// ```
///
/// # Example
/// ```
/// use pawer::filters::LowPassFilter;
///
/// let mut lpf = LowPassFilter::new(100.0, 1e-4); // 100 rad/s, Ts = 100 µs
/// let output = lpf.update(1.0);
/// assert!(output >= 0.0 && output <= 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct LowPassFilter {
    b0: f64,
    b1: f64,
    a1: f64,
    x_prev: f64,
    y_prev: f64,
}

impl LowPassFilter {
    /// Create a new first-order low-pass filter.
    ///
    /// # Arguments
    /// * `cutoff_freq_rad` – Cut-off frequency in **rad/s** (ωc > 0).
    /// * `sample_time`     – Sampling period in **seconds** (Ts > 0).
    ///
    /// # Panics
    /// Panics when `cutoff_freq_rad` or `sample_time` are not strictly positive.
    pub fn new(cutoff_freq_rad: f64, sample_time: f64) -> Self {
        assert!(cutoff_freq_rad > 0.0, "cutoff_freq_rad must be > 0");
        assert!(sample_time > 0.0, "sample_time must be > 0");

        let k = cutoff_freq_rad * sample_time / 2.0;
        let b0 = k / (1.0 + k);
        let b1 = b0;
        let a1 = (k - 1.0) / (k + 1.0);

        Self {
            b0,
            b1,
            a1,
            x_prev: 0.0,
            y_prev: 0.0,
        }
    }

    /// Process one sample and return the filtered output.
    pub fn update(&mut self, input: f64) -> f64 {
        let output = self.b0 * input + self.b1 * self.x_prev - self.a1 * self.y_prev;
        self.x_prev = input;
        self.y_prev = output;
        output
    }

    /// Reset the filter state to zero.
    pub fn reset(&mut self) {
        self.x_prev = 0.0;
        self.y_prev = 0.0;
    }
}

/// Second-order low-pass filter discretised via the bilinear transform.
///
/// Continuous-time transfer function (Butterworth-style, ζ = √2/2):
/// ```text
/// H(s) = ωn² / (s² + √2·ωn·s + ωn²)
/// ```
///
/// # Example
/// ```
/// use pawer::filters::SecondOrderLowPassFilter;
///
/// let mut lpf2 = SecondOrderLowPassFilter::new(100.0, 1e-4);
/// let output = lpf2.update(1.0);
/// assert!(output >= 0.0 && output <= 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct SecondOrderLowPassFilter {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl SecondOrderLowPassFilter {
    /// Create a new second-order Butterworth low-pass filter.
    ///
    /// # Arguments
    /// * `natural_freq_rad` – Natural (cut-off) frequency in **rad/s** (ωn > 0).
    /// * `sample_time`      – Sampling period in **seconds** (Ts > 0).
    ///
    /// # Panics
    /// Panics when `natural_freq_rad` or `sample_time` are not strictly positive.
    pub fn new(natural_freq_rad: f64, sample_time: f64) -> Self {
        assert!(natural_freq_rad > 0.0, "natural_freq_rad must be > 0");
        assert!(sample_time > 0.0, "sample_time must be > 0");

        // Pre-warp the analogue cut-off frequency for the bilinear transform.
        let wd = 2.0 / sample_time * (natural_freq_rad * sample_time / 2.0).tan();
        let zeta = 2.0_f64.sqrt() / 2.0; // Butterworth damping ratio

        let k = wd * sample_time / 2.0;
        let k2 = k * k;
        let denom = 1.0 + 2.0 * zeta * k + k2;

        let b0 = k2 / denom;
        let b1 = 2.0 * b0;
        let b2 = b0;
        let a1 = 2.0 * (k2 - 1.0) / denom;
        let a2 = (1.0 - 2.0 * zeta * k + k2) / denom;

        Self {
            b0,
            b1,
            b2,
            a1,
            a2,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Process one sample and return the filtered output.
    pub fn update(&mut self, input: f64) -> f64 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }

    /// Reset the filter state to zero.
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit step applied to the LPF must converge to 1.0.
    #[test]
    fn lpf_step_response_converges_to_one() {
        let mut lpf = LowPassFilter::new(100.0, 1e-4);
        let mut out = 0.0;
        for _ in 0..10_000 {
            out = lpf.update(1.0);
        }
        assert!((out - 1.0).abs() < 1e-6, "expected ≈1.0, got {out}");
    }

    /// Output must start below the input and never overshoot for a first-order filter.
    #[test]
    fn lpf_no_overshoot() {
        let mut lpf = LowPassFilter::new(200.0, 1e-4);
        for _ in 0..5_000 {
            let out = lpf.update(1.0);
            assert!(
                out <= 1.0 + 1e-10,
                "first-order LPF must not overshoot, got {out}"
            );
        }
    }

    /// `reset()` must bring the state back to zero.
    #[test]
    fn lpf_reset() {
        let mut lpf = LowPassFilter::new(100.0, 1e-4);
        for _ in 0..100 {
            lpf.update(1.0);
        }
        lpf.reset();
        let out = lpf.update(0.0);
        assert_eq!(out, 0.0, "after reset the output of a zero input must be 0");
    }

    /// A unit step applied to the second-order LPF must converge to 1.0.
    #[test]
    fn lpf2_step_response_converges_to_one() {
        let mut lpf2 = SecondOrderLowPassFilter::new(100.0, 1e-4);
        let mut out = 0.0;
        for _ in 0..10_000 {
            out = lpf2.update(1.0);
        }
        assert!((out - 1.0).abs() < 1e-6, "expected ≈1.0, got {out}");
    }

    /// `reset()` must return the second-order filter state to zero.
    #[test]
    fn lpf2_reset() {
        let mut lpf2 = SecondOrderLowPassFilter::new(100.0, 1e-4);
        for _ in 0..100 {
            lpf2.update(1.0);
        }
        lpf2.reset();
        let out = lpf2.update(0.0);
        assert_eq!(out, 0.0, "after reset the output of a zero input must be 0");
    }

    /// The first-order filter should attenuate a high-frequency sinusoid.
    #[test]
    fn lpf_attenuates_high_frequency() {
        // Filter: ωc = 100 rad/s
        // Input: sin(2000·t), well above the cut-off.
        let mut lpf = LowPassFilter::new(100.0, 1e-5);
        let ts = 1e-5_f64;
        let freq = 2000.0_f64; // rad/s

        // Let transients die out.
        for i in 0..20_000 {
            lpf.update((freq * ts * i as f64).sin());
        }

        // Measure peak output over the next full cycle.
        let samples_per_cycle = (2.0 * std::f64::consts::PI / (freq * ts)).ceil() as usize;
        let mut peak = 0.0_f64;
        for i in 20_000..20_000 + samples_per_cycle {
            let out = lpf.update((freq * ts * i as f64).sin());
            peak = peak.max(out.abs());
        }

        // Theoretical gain ≈ ωc / ω = 100 / 2000 = 0.05
        assert!(peak < 0.1, "expected attenuation, peak was {peak}");
    }
}
