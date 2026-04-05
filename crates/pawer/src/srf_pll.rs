//! Synchronous-Reference Frame Phase-Locked Loop (SRF-PLL).
//!
//! Estimates the frequency and phase of a three-phase sinusoidal signal
//! supplied as an αβ (stationary frame) vector.
//!
//! # Algorithm
//!
//! 1. Park-transform the input [`AlphaBeta`] using the current estimated angle.
//! 2. The q-axis component is the phase error.
//! 3. A PI controller drives q → 0 by adjusting the estimated frequency.
//! 4. The adjusted angular frequency is integrated to produce the next angle.
//! 5. A 2nd-order LPF smooths the instantaneous frequency for reporting.
//!
//! The PLL's internal angle is cosine-aligned.  Add π/2 via
//! [`SrfPll::estimated_angle_phase_a`] to obtain the phase-A sine angle.

use crate::angle::AngleWrapped;
use crate::constants::{PI, TWO_PI};
use crate::frames::{AlphaBeta, Dq};
use crate::pi_controller::PiController;
use crate::reciprocal::Reciprocal;
use crate::second_order_filter::SecondOrderLowPassFilter;
use crate::types::Real;

// ── Default tuning constants ──────────────────────────────────────────────────

/// Default cutoff frequency for the frequency-estimation 2nd-order LPF (Hz).
pub const DEFAULT_FILTER_CUTOFF_HZ: f64 = 100.0;

/// PI gains targeting a 50 Hz loop-crossover frequency.
pub const GAINS_50HZ_CROSSOVER: PiGains = PiGains {
    kp: 0.866,
    ti: 0.005_51,
};

/// PI gains targeting a 30 Hz loop-crossover frequency.
pub const GAINS_30HZ_CROSSOVER: PiGains = PiGains {
    kp: 0.519_6,
    ti: 0.009_19,
};

// ── PiGains ───────────────────────────────────────────────────────────────────

/// Proportional gain and integral time constant for the SRF-PLL PI controller.
#[derive(Clone, Copy, Debug)]
pub struct PiGains {
    pub kp: Real,
    /// Integral time constant in seconds.
    pub ti: Real,
}

// ── SrfPll ────────────────────────────────────────────────────────────────────

/// Synchronous-Reference Frame Phase-Locked Loop.
///
/// # Example
///
/// ```
/// use pawer::srf_pll::{SrfPll, GAINS_50HZ_CROSSOVER};
/// use pawer::frames::AlphaBeta;
///
/// let ts = 100e-6_f32; // 10 kHz
/// let mut pll = SrfPll::new(ts);
/// pll.configure_nominal_frequency(50.0);
/// pll.reset_with_frequency(1.0);
///
/// // Feed one zero sample
/// pll.update(AlphaBeta::default());
/// let _freq_hz = pll.estimated_frequency_hz();
/// ```
pub struct SrfPll {
    frequency_pi: PiController,
    frequency_filter: SecondOrderLowPassFilter,
    /// Reciprocal of the nominal frequency (Hz).  Used for pu ↔ Hz conversions.
    nominal_frequency: Reciprocal,
    /// Per-unit feed-forward frequency (1.0 = nominal).
    initial_frequency_pu: Real,
    /// Filtered estimated frequency in Hz.
    estimated_frequency_hz: Real,
    /// Accumulated phase angle (wraps naturally via u32 arithmetic).
    phase_angle: AngleWrapped,
    /// Park-transformed dq vector from the last [`update`](Self::update).
    dq: Dq<Real>,
    /// Sampling period in seconds.
    sampling_time: Real,
}

impl SrfPll {
    /// Create a new SRF-PLL.
    ///
    /// Defaults applied automatically:
    /// - Nominal frequency: 50 Hz
    /// - PI gains: [`GAINS_50HZ_CROSSOVER`]
    /// - Estimation filter cutoff: [`DEFAULT_FILTER_CUTOFF_HZ`]
    /// - Initial frequency: 0 pu (call [`reset_with_frequency`](Self::reset_with_frequency) to change)
    pub fn new(sampling_time: Real) -> Self {
        debug_assert!(sampling_time > 0.0, "Sampling time must be positive");

        let mut pll = Self {
            frequency_pi: PiController::new(sampling_time),
            frequency_filter: SecondOrderLowPassFilter::new(sampling_time),
            nominal_frequency: Reciprocal::new(50.0),
            initial_frequency_pu: 0.0,
            estimated_frequency_hz: 0.0,
            phase_angle: AngleWrapped::default(),
            dq: Dq::default(),
            sampling_time,
        };

        pll.configure_frequency_estimation_filter(DEFAULT_FILTER_CUTOFF_HZ);
        pll.configure_pi_controller(GAINS_50HZ_CROSSOVER.kp, GAINS_50HZ_CROSSOVER.ti);
        pll.reset();
        pll
    }

    // ── Configuration ─────────────────────────────────────────────────────

    /// Set the nominal (center) frequency in Hz.
    pub fn configure_nominal_frequency(&mut self, hz: Real) {
        debug_assert!(hz >= 0.0);
        self.nominal_frequency = Reciprocal::new(hz);
    }

    /// Configure the PI controller with proportional gain `kp` and integral
    /// time constant `ti` (seconds).
    pub fn configure_pi_controller(&mut self, kp: Real, ti: Real) {
        self.frequency_pi.configure_with_ti(kp, ti);
    }

    /// Configure the 2nd-order LPF used for frequency estimation.
    ///
    /// `cutoff_hz` is the −3 dB frequency in Hz.  A Butterworth response
    /// is used (`damping = 1/√2 ≈ 0.707`).
    pub fn configure_frequency_estimation_filter(&mut self, cutoff_hz: f64) {
        self.frequency_filter.configure(cutoff_hz, 0.707);
    }

    // ── Reset / preset ────────────────────────────────────────────────────

    /// Reset to zero angle and zero estimated frequency.
    pub fn reset(&mut self) {
        self.reset_with_frequency(0.0);
    }

    /// Reset to zero angle and a specific per-unit initial frequency.
    ///
    /// `initial_frequency_pu = 1.0` sets the feed-forward to the nominal
    /// frequency (e.g. 50 Hz).
    pub fn reset_with_frequency(&mut self, initial_frequency_pu: Real) {
        self.initial_frequency_pu = initial_frequency_pu;
        self.estimated_frequency_hz = self.nominal_frequency.value() * initial_frequency_pu;
        self.dq = Dq::default();
        self.frequency_pi.reset_to_zero();
        self.phase_angle = AngleWrapped::default();
    }

    /// Preset the PLL to a specific angle and per-unit frequency.
    ///
    /// Resets the PI integrator and the frequency estimation filter so the
    /// PLL starts from a clean, consistent state at the given operating point.
    /// Useful for bumpless hand-off from a virtual angle source.
    pub fn preset(&mut self, angle: AngleWrapped, frequency_pu: Real) {
        self.initial_frequency_pu = frequency_pu;
        self.estimated_frequency_hz = self.nominal_frequency.value() * frequency_pu;
        self.frequency_pi.reset_to_zero();
        self.frequency_filter.reset_to(self.estimated_frequency_hz);
        self.phase_angle = angle;
    }

    /// Preset for bumpless transfer by pre-loading the PI integrator.
    ///
    /// Sets the PI integrator so that the first output is zero frequency
    /// deviation for the current phase error, avoiding a transient on
    /// hand-off from an external angle source.
    pub fn preset_for_bumpless_transfer(
        &mut self,
        angle: AngleWrapped,
        frequency_pu: Real,
        input: AlphaBeta<Real>,
    ) {
        self.initial_frequency_pu = frequency_pu;
        self.estimated_frequency_hz = self.nominal_frequency.value() * frequency_pu;
        self.phase_angle = angle;
        self.dq = input.to_dq(angle);
        let current_error = self.dq.q();
        self.frequency_pi
            .preset_for_bumpless_transfer(0.0, current_error);
        self.frequency_filter.reset_to(self.estimated_frequency_hz);
    }

    // ── Update ────────────────────────────────────────────────────────────

    /// Advance the PLL by one sample.
    ///
    /// `input` is the per-unit αβ voltage vector (e.g. from a Clarke
    /// transform of the measured three-phase voltages).
    pub fn update(&mut self, input: AlphaBeta<Real>) {
        // Park transform using the angle from the start of this sample
        self.dq = input.to_dq(self.phase_angle);

        // q-axis is the phase error; PI drives it to zero
        let frequency_deviation = self.frequency_pi.update(self.dq.q());

        // Raw frequency estimate (Hz): nominal * (pu_feedforward + deviation)
        let raw_freq =
            self.nominal_frequency.value() * (self.initial_frequency_pu + frequency_deviation);

        // Integrate angular frequency to advance the phase
        self.phase_angle += AngleWrapped::new(TWO_PI * raw_freq * self.sampling_time);

        // Low-pass filter the raw estimate for the smooth reported frequency
        self.frequency_filter.update(raw_freq);
        self.estimated_frequency_hz = self.frequency_filter.output();
    }

    // ── Outputs ───────────────────────────────────────────────────────────

    /// Estimated phase angle aligned with the **cosine** waveform.
    ///
    /// Includes compensation for the one-sample computational delay so the
    /// returned angle corresponds to the midpoint of the current sample.
    pub fn estimated_angle(&self) -> AngleWrapped {
        let compensation = 0.25 * self.estimated_angular_frequency() * self.sampling_time;
        self.phase_angle - AngleWrapped::new(compensation)
    }

    /// Estimated phase angle aligned with the **phase-A sine** waveform.
    ///
    /// Equal to [`estimated_angle`](Self::estimated_angle) + π/2.
    pub fn estimated_angle_phase_a(&self) -> AngleWrapped {
        self.estimated_angle() + AngleWrapped::new(PI / 2.0)
    }

    /// Estimated frequency in Hz.
    pub fn estimated_frequency_hz(&self) -> Real {
        self.estimated_frequency_hz
    }

    /// Estimated frequency in per-unit relative to the nominal frequency.
    pub fn estimated_frequency_pu(&self) -> Real {
        self.nominal_frequency.divide(self.estimated_frequency_hz)
    }

    /// Estimated angular frequency in rad/s.
    pub fn estimated_angular_frequency(&self) -> Real {
        TWO_PI * self.estimated_frequency_hz
    }

    /// Park-transformed dq vector from the last call to [`update`](Self::update).
    ///
    /// When locked, `d ≈ signal_amplitude` and `q ≈ 0`.
    pub fn dq(&self) -> Dq<Real> {
        self.dq
    }

    /// Reconstructed αβ vector from the dq vector and estimated angle.
    pub fn alphabeta(&self) -> AlphaBeta<Real> {
        self.dq.to_alphabeta(self.estimated_angle())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frames::Abc;

    const TS: Real = 100e-6; // 100 µs — 10 kHz
    const NOMINAL_HZ: Real = 50.0;
    const AMPLITUDE: Real = 1.0;
    const FREQ_TOL_RAD: Real = TWO_PI * 0.1; // 0.1 Hz tolerance in rad/s
    const ANGLE_TOL: Real = 0.05; // ~3 degrees

    fn make_pll() -> SrfPll {
        let mut pll = SrfPll::new(TS);
        pll.configure_nominal_frequency(NOMINAL_HZ);
        pll.configure_pi_controller(GAINS_50HZ_CROSSOVER.kp, GAINS_50HZ_CROSSOVER.ti);
        pll.reset_with_frequency(1.0);
        pll
    }

    /// Generate one αβ sample for a balanced positive-sequence sine signal.
    fn signal_at(angle: AngleWrapped, amplitude: Real) -> AlphaBeta<Real> {
        let theta = angle.radians();
        let a = amplitude * libm::sinf(theta);
        let b = amplitude * libm::sinf(theta - TWO_PI / 3.0);
        let c = amplitude * libm::sinf(theta + TWO_PI / 3.0);
        Abc::new(a, b, c).to_alphabeta()
    }

    /// Run both the signal source and PLL for `steps` samples.
    /// Returns (signal_angle_at_end, pll_state).
    fn simulate(
        pll: &mut SrfPll,
        initial_signal_angle: AngleWrapped,
        freq_hz: Real,
        amplitude: Real,
        steps: usize,
    ) -> AngleWrapped {
        let mut sig_angle = initial_signal_angle;
        let angle_inc = AngleWrapped::new(TWO_PI * freq_hz * TS);
        for _ in 0..steps {
            let input = signal_at(sig_angle, amplitude);
            pll.update(input);
            sig_angle += angle_inc;
        }
        sig_angle
    }

    #[test]
    fn initializes_to_expected_state() {
        let mut pll = SrfPll::new(TS);
        pll.configure_nominal_frequency(NOMINAL_HZ);

        // Default reset → zero frequency
        pll.reset();
        assert!(
            pll.estimated_frequency_hz().abs() < 0.01,
            "Expected 0 Hz, got {}",
            pll.estimated_frequency_hz()
        );
        assert!(
            pll.estimated_frequency_pu().abs() < 0.01,
            "Expected 0 pu, got {}",
            pll.estimated_frequency_pu()
        );
        assert!(
            pll.estimated_angle().radians().abs() < 0.01,
            "Expected zero angle, got {}",
            pll.estimated_angle().radians()
        );

        // Reset with 1 pu → nominal frequency
        pll.reset_with_frequency(1.0);
        assert!(
            (pll.estimated_frequency_hz() - NOMINAL_HZ).abs() < 0.01,
            "Expected {} Hz, got {}",
            NOMINAL_HZ,
            pll.estimated_frequency_hz()
        );
        assert!(
            (pll.estimated_frequency_pu() - 1.0).abs() < 0.01,
            "Expected 1.0 pu, got {}",
            pll.estimated_frequency_pu()
        );

        // dq should be zero before any update
        assert!(
            pll.dq().d().abs() < 0.01,
            "d should be 0, got {}",
            pll.dq().d()
        );
        assert!(
            pll.dq().q().abs() < 0.01,
            "q should be 0, got {}",
            pll.dq().q()
        );
    }

    #[test]
    fn locks_to_nominal_frequency() {
        let mut pll = make_pll();
        let settle_steps = (0.5 / TS as f64) as usize; // 0.5 s

        simulate(
            &mut pll,
            AngleWrapped::default(),
            NOMINAL_HZ,
            AMPLITUDE,
            settle_steps,
        );

        let target_omega = TWO_PI * NOMINAL_HZ;
        assert!(
            (pll.estimated_angular_frequency() - target_omega).abs() < FREQ_TOL_RAD,
            "Frequency not locked: {:.3} rad/s (expected {:.3})",
            pll.estimated_angular_frequency(),
            target_omega
        );
        assert!(
            pll.dq().q().abs() < 0.01,
            "q-axis not zero: {:.4}",
            pll.dq().q()
        );
        assert!(
            (pll.dq().d() - AMPLITUDE).abs() < 0.01,
            "d-axis not at amplitude: {:.4}",
            pll.dq().d()
        );
    }

    #[test]
    fn handles_frequency_step() {
        let mut pll = make_pll();
        let step_hz: Real = 60.0;

        // Settle at 50 Hz
        let sig_angle = simulate(
            &mut pll,
            AngleWrapped::default(),
            NOMINAL_HZ,
            AMPLITUDE,
            (0.4 / TS as f64) as usize,
        );

        // Step to 60 Hz
        simulate(
            &mut pll,
            sig_angle,
            step_hz,
            AMPLITUDE,
            (0.6 / TS as f64) as usize,
        );

        let target_omega = TWO_PI * step_hz;
        assert!(
            (pll.estimated_angular_frequency() - target_omega).abs() < FREQ_TOL_RAD,
            "Did not lock to {step_hz} Hz after step: {:.3} rad/s",
            pll.estimated_angular_frequency()
        );
        assert!(
            pll.dq().q().abs() < 0.01,
            "q-axis not zero after step: {:.4}",
            pll.dq().q()
        );
    }

    #[test]
    fn handles_phase_step() {
        let mut pll = make_pll();

        // Settle
        let mut sig_angle = simulate(
            &mut pll,
            AngleWrapped::default(),
            NOMINAL_HZ,
            AMPLITUDE,
            (0.5 / TS as f64) as usize,
        );

        // Apply 45° phase jump
        sig_angle += AngleWrapped::new(PI / 4.0);

        // Allow re-lock
        simulate(
            &mut pll,
            sig_angle,
            NOMINAL_HZ,
            AMPLITUDE,
            (0.5 / TS as f64) as usize,
        );

        let target_omega = TWO_PI * NOMINAL_HZ;
        assert!(
            (pll.estimated_angular_frequency() - target_omega).abs() < FREQ_TOL_RAD,
            "Frequency deviated after phase step: {:.3} rad/s",
            pll.estimated_angular_frequency()
        );
        assert!(
            pll.dq().q().abs() < 0.01,
            "q-axis not zero after phase step: {:.4}",
            pll.dq().q()
        );
    }

    #[test]
    fn handles_amplitude_step() {
        let mut pll = make_pll();
        let new_amplitude: Real = 1.5;

        // Settle at unit amplitude
        let sig_angle = simulate(
            &mut pll,
            AngleWrapped::default(),
            NOMINAL_HZ,
            AMPLITUDE,
            (0.5 / TS as f64) as usize,
        );

        // Continue with new amplitude
        simulate(
            &mut pll,
            sig_angle,
            NOMINAL_HZ,
            new_amplitude,
            (0.5 / TS as f64) as usize,
        );

        let target_omega = TWO_PI * NOMINAL_HZ;
        assert!(
            (pll.estimated_angular_frequency() - target_omega).abs() < FREQ_TOL_RAD,
            "Frequency destabilised by amplitude step: {:.3} rad/s",
            pll.estimated_angular_frequency()
        );
        assert!(
            (pll.dq().d() - new_amplitude).abs() < 0.01,
            "d-axis not at new amplitude {new_amplitude}: {:.4}",
            pll.dq().d()
        );
        assert!(
            pll.dq().q().abs() < 0.01,
            "q-axis not zero after amplitude step: {:.4}",
            pll.dq().q()
        );
    }

    #[test]
    fn preset_restores_tracking() {
        let mut pll = make_pll();

        // Settle
        simulate(
            &mut pll,
            AngleWrapped::default(),
            NOMINAL_HZ,
            AMPLITUDE,
            (0.3 / TS as f64) as usize,
        );

        // Intentionally preset to a 90° offset (introduces a phase error)
        let offset_angle = pll.estimated_angle() + AngleWrapped::new(PI / 2.0);
        pll.preset(offset_angle, 1.0);

        // Allow correction
        simulate(
            &mut pll,
            AngleWrapped::new(0.0), // signal continues from wherever it is
            NOMINAL_HZ,
            AMPLITUDE,
            (0.7 / TS as f64) as usize,
        );

        let target_omega = TWO_PI * NOMINAL_HZ;
        assert!(
            (pll.estimated_angular_frequency() - target_omega).abs() < FREQ_TOL_RAD,
            "Frequency not recovered after preset: {:.3} rad/s",
            pll.estimated_angular_frequency()
        );
    }

    #[test]
    fn angle_phase_a_offset_from_estimated_angle() {
        let mut pll = make_pll();
        simulate(
            &mut pll,
            AngleWrapped::default(),
            NOMINAL_HZ,
            AMPLITUDE,
            (0.5 / TS as f64) as usize,
        );

        let delta = (pll.estimated_angle_phase_a() - pll.estimated_angle()).radians();
        // Difference should be ≈ π/2 (wrapped, so ∈ [0, 2π))
        let expected = PI / 2.0;
        assert!(
            (delta - expected).abs() < ANGLE_TOL,
            "Phase-A offset unexpected: {:.4} rad (expected {:.4})",
            delta,
            expected
        );
    }
}
