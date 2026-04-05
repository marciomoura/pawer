//! Simplified three-phase sinusoidal waveform generator for SIL simulation.
//!
//! Generates a balanced positive-sequence fundamental signal:
//!
//! ```text
//! Phase A: A · sin(θ)
//! Phase B: A · sin(θ − 2π/3)
//! Phase C: A · sin(θ + 2π/3)
//! ```
//!
//! The angle θ is advanced by `2π · f · Ts` every call to [`update`](ThreePhaseGenerator::update).
//!
//! Optional one-shot events (steps, ramps) let the user schedule disturbances
//! for testing PLL and other control-block responses.

use std::f64::consts::PI;

use pawer::angle::AngleWrapped;
use pawer::frames::Abc;
use pawer::types::Real;

const TWO_PI_F64: f64 = 2.0 * PI;

// ── Internal event types ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct Step<T> {
    /// Delta applied at `time`.
    delta: T,
    time: f64,
}

#[derive(Clone, Copy)]
struct Ramp<T> {
    /// Total change accumulated over [`start`..`end`].
    delta: T,
    start: f64,
    end: f64,
}

// ── ThreePhaseGenerator ───────────────────────────────────────────────────────

/// Balanced three-phase sinusoidal signal source.
///
/// # Typical usage
///
/// ```
/// use pawer_examples::waveform_gen::ThreePhaseGenerator;
///
/// let mut wgen = ThreePhaseGenerator::new(100e-6); // 10 kHz
/// wgen.set_frequency(50.0);
/// wgen.set_amplitude(1.0);
///
/// wgen.update();
/// let abc = wgen.signal();
/// let alphabeta = abc.to_alphabeta();
/// ```
pub struct ThreePhaseGenerator {
    sampling_time: f64,
    simulation_time: f64,
    frequency: f64,
    amplitude: f64,
    /// Phase-A angle at the start of the current sample.
    angle: AngleWrapped,
    /// Computed signal — updated on each call to [`update`](Self::update).
    signal: Abc<Real>,
    // One-shot events
    freq_step: Option<Step<f64>>,
    freq_ramp: Option<Ramp<f64>>,
    amplitude_step: Option<Step<f64>>,
}

impl ThreePhaseGenerator {
    /// Create a generator with the given sampling period (seconds).
    pub fn new(sampling_time: f64) -> Self {
        assert!(sampling_time > 0.0, "Sampling time must be positive");
        Self {
            sampling_time,
            simulation_time: 0.0,
            frequency: 50.0,
            amplitude: 1.0,
            angle: AngleWrapped::default(),
            signal: Abc::default(),
            freq_step: None,
            freq_ramp: None,
            amplitude_step: None,
        }
    }

    // ── Configuration ─────────────────────────────────────────────────────

    /// Set the signal frequency in Hz.
    pub fn set_frequency(&mut self, hz: f64) {
        self.frequency = hz;
    }

    /// Set the peak amplitude.
    pub fn set_amplitude(&mut self, amplitude: f64) {
        self.amplitude = amplitude;
    }

    /// Set the phase-A starting angle.
    pub fn set_angle(&mut self, angle: AngleWrapped) {
        self.angle = angle;
    }

    // ── Events ────────────────────────────────────────────────────────────

    /// Schedule a one-shot frequency step: `delta_hz` is added to the
    /// current frequency when the simulation time reaches `time`.
    pub fn schedule_frequency_step(&mut self, delta_hz: f64, time: f64) {
        self.freq_step = Some(Step { delta: delta_hz, time });
    }

    /// Schedule a linear frequency ramp.
    ///
    /// `delta_hz` is the total frequency change spread uniformly over the
    /// interval `[start_time, end_time]`.
    pub fn schedule_frequency_ramp(&mut self, delta_hz: f64, start_time: f64, end_time: f64) {
        assert!(end_time > start_time, "Ramp end must be after start");
        self.freq_ramp = Some(Ramp { delta: delta_hz, start: start_time, end: end_time });
    }

    /// Schedule a one-shot amplitude step: `delta` is added to the
    /// current amplitude when the simulation time reaches `time`.
    pub fn schedule_amplitude_step(&mut self, delta: f64, time: f64) {
        self.amplitude_step = Some(Step { delta, time });
    }

    // ── Update ────────────────────────────────────────────────────────────

    /// Advance the generator by one sample.
    ///
    /// Call order mirrors the C++ reference implementation:
    /// 1. Apply pending step events.
    /// 2. Apply active ramp increments.
    /// 3. Compute the output signal from the **current** angle.
    /// 4. Advance the simulation time and the phase angle.
    pub fn update(&mut self) {
        self.apply_steps();
        self.apply_ramps();
        self.compute_signal();
        self.advance();
    }

    // ── Outputs ───────────────────────────────────────────────────────────

    /// Three-phase signal computed during the last [`update`](Self::update).
    pub fn signal(&self) -> Abc<Real> {
        self.signal
    }

    /// Phase-A angle at the start of the last completed sample.
    pub fn angle(&self) -> AngleWrapped {
        self.angle
    }

    /// Current signal frequency in Hz.
    pub fn frequency(&self) -> f64 {
        self.frequency
    }

    /// Current simulation time in seconds.
    pub fn time(&self) -> f64 {
        self.simulation_time
    }

    // ── Private helpers ───────────────────────────────────────────────────

    fn apply_steps(&mut self) {
        if let Some(s) = self.freq_step
            && self.is_step_time(s.time)
        {
            self.frequency += s.delta;
            self.freq_step = None;
        }

        if let Some(s) = self.amplitude_step
            && self.is_step_time(s.time)
        {
            self.amplitude += s.delta;
            self.amplitude_step = None;
        }
    }

    fn apply_ramps(&mut self) {
        if let Some(r) = self.freq_ramp
            && self.simulation_time >= r.start
            && self.simulation_time <= r.end
        {
            let increment = r.delta * self.sampling_time / (r.end - r.start);
            self.frequency += increment;
        }
    }

    fn compute_signal(&mut self) {
        let theta = self.angle.radians() as f64;
        let amp = self.amplitude;
        self.signal = Abc::new(
            (amp * theta.sin()) as Real,
            (amp * (theta - TWO_PI_F64 / 3.0).sin()) as Real,
            (amp * (theta + TWO_PI_F64 / 3.0).sin()) as Real,
        );
    }

    fn advance(&mut self) {
        self.simulation_time += self.sampling_time;
        let increment = (TWO_PI_F64 * self.frequency * self.sampling_time) as Real;
        self.angle += AngleWrapped::new(increment);
    }

    /// Returns `true` if the current simulation time is within ±Ts/2 of `t`.
    fn is_step_time(&self, t: f64) -> bool {
        let half = self.sampling_time / 2.0;
        self.simulation_time >= t - half && self.simulation_time <= t + half
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TS: f64 = 100e-6;
    const F: f64 = 50.0;
    const A: f64 = 1.0;

    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn initial_signal_is_zero() {
        let wgen = ThreePhaseGenerator::new(TS);
        let s = wgen.signal();
        assert_eq!(s.a(), 0.0);
        assert_eq!(s.b(), 0.0);
        assert_eq!(s.c(), 0.0);
    }

    #[test]
    fn first_sample_phase_a_is_zero() {
        let mut wgen = ThreePhaseGenerator::new(TS);
        wgen.set_frequency(F);
        wgen.set_amplitude(A);
        wgen.update(); // θ = 0 at sample start → sin(0) = 0
        assert!(
            approx(wgen.signal().a() as f64, 0.0, 1e-6),
            "Phase A should be 0 at θ=0, got {}",
            wgen.signal().a()
        );
    }

    #[test]
    fn balanced_signal_sums_to_zero() {
        let mut wgen = ThreePhaseGenerator::new(TS);
        wgen.set_frequency(F);
        wgen.set_amplitude(A);

        for _ in 0..1000 {
            wgen.update();
            let s = wgen.signal();
            let sum = s.a() as f64 + s.b() as f64 + s.c() as f64;
            assert!(approx(sum, 0.0, 1e-4), "Phases don't sum to zero: {sum:.6}");
        }
    }

    #[test]
    fn frequency_step_applied_once() {
        let mut wgen = ThreePhaseGenerator::new(TS);
        wgen.set_frequency(50.0);
        wgen.schedule_frequency_step(10.0, 0.5); // step at t = 0.5 s

        for _ in 0..5001 {
            wgen.update();
        }
        // At t ≈ 0.5 s the step fires; final freq should be 60 Hz
        assert!(
            approx(wgen.frequency(), 60.0, 1e-6),
            "Frequency after step: {}",
            wgen.frequency()
        );
    }

    #[test]
    fn frequency_ramp_changes_frequency() {
        let mut wgen = ThreePhaseGenerator::new(TS);
        wgen.set_frequency(50.0);
        wgen.schedule_frequency_ramp(10.0, 0.0, 1.0); // ramp 50→60 Hz over 1 s

        let steps = (1.0 / TS) as usize + 1;
        for _ in 0..steps {
            wgen.update();
        }
        // After ramp ends frequency should be ≈ 60 Hz
        assert!(
            approx(wgen.frequency(), 60.0, 0.01),
            "Frequency after ramp: {}",
            wgen.frequency()
        );
    }

    #[test]
    fn alphabeta_magnitude_equals_amplitude() {
        let mut wgen = ThreePhaseGenerator::new(TS);
        wgen.set_frequency(F);
        wgen.set_amplitude(A);

        // Skip the first quarter-cycle (sin is near zero)
        for _ in 0..50 {
            wgen.update();
        }

        let ab = wgen.signal().to_alphabeta();
        let magnitude = (ab.alpha() * ab.alpha() + ab.beta() * ab.beta()).sqrt();
        assert!(
            approx(magnitude as f64, A, 0.01),
            "αβ magnitude: {magnitude:.4}, expected {A}"
        );
    }
}
