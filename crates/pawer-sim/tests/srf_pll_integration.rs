//! Integration tests: SRF-PLL driven by ThreePhaseGenerator.
//!
//! These tests verify the full signal-generation → Clarke-transform → PLL
//! pipeline at the `pawer-sim` level, including disturbance responses that
//! mirror the interactive scenario.

use pawer::constants::TWO_PI;
use pawer::srf_pll::{SrfPll, GAINS_50HZ_CROSSOVER};
use pawer::types::Real;
use pawer_sim::waveform_gen::ThreePhaseGenerator;

const TS: f64 = 100e-6; // 100 µs — 10 kHz
const NOMINAL_HZ: f64 = 50.0;
const AMPLITUDE: f64 = 1.0;
const FREQ_TOL_HZ: Real = 0.1; // ±0.1 Hz after settling
const Q_TOL: Real = 0.01; // q-axis tolerance when locked
const D_TOL: Real = 0.01; // d-axis tolerance relative to amplitude

/// Run `steps` samples through the generator + PLL.
/// The generator frequency is computed from `freq_fn(sim_time)` each step,
/// matching the stateless approach used in the example scenario.
fn simulate(
    pll: &mut SrfPll,
    wgen: &mut ThreePhaseGenerator,
    steps: usize,
    freq_fn: impl Fn(f64) -> f64,
) {
    for i in 0..steps {
        let t = i as f64 * TS;
        wgen.set_frequency(freq_fn(t));
        wgen.update();
        let ab = wgen.signal().to_alphabeta();
        pll.update(ab);
    }
}

fn make_pll() -> SrfPll {
    let mut pll = SrfPll::new(TS as Real);
    pll.configure_nominal_frequency(NOMINAL_HZ as Real);
    pll.configure_pi_controller(GAINS_50HZ_CROSSOVER.kp, GAINS_50HZ_CROSSOVER.ti);
    pll.reset_with_frequency(1.0);
    pll
}

fn make_gen() -> ThreePhaseGenerator {
    let mut wgen = ThreePhaseGenerator::new(TS);
    wgen.set_amplitude(AMPLITUDE);
    wgen
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn locks_to_50_hz() {
    let mut pll = make_pll();
    let mut wgen = make_gen();

    simulate(&mut pll, &mut wgen, (0.5 / TS) as usize, |_| NOMINAL_HZ);

    let err = (pll.estimated_frequency_hz() - NOMINAL_HZ as Real).abs();
    assert!(
        err < FREQ_TOL_HZ,
        "Expected lock at {NOMINAL_HZ} Hz, got {:.3} Hz (err = {err:.4})",
        pll.estimated_frequency_hz()
    );
    assert!(
        pll.dq().q().abs() < Q_TOL,
        "q-axis not zeroed after lock: {:.4}",
        pll.dq().q()
    );
    assert!(
        (pll.dq().d() - AMPLITUDE as Real).abs() < D_TOL,
        "d-axis not at amplitude after lock: {:.4}",
        pll.dq().d()
    );
}

/// This test mirrors the user workflow that was broken:
///
/// 1. First `/simulate 0.5`  → PLL locks at 50 Hz.
/// 2. User sets `freq_step = 10.0`, `freq_step_at = 0.5`.
/// 3. Second `/simulate 0.5` → freq is 60 Hz (t ≥ 0.5), PLL re-locks.
///
/// The stateless frequency function `nominal + step if t >= step_at`
/// ensures the step is applied correctly regardless of when the user sets it.
#[test]
fn frequency_step_response() {
    let step_hz: f64 = 10.0;
    let step_at: f64 = 0.5; // seconds (absolute simulation time)
    let target_hz = NOMINAL_HZ + step_hz;

    let mut pll = make_pll();
    let mut wgen = make_gen();

    let freq_fn = |t: f64| -> f64 {
        if t >= step_at { target_hz } else { NOMINAL_HZ }
    };

    // Phase 1: lock (0 → 0.5 s)
    simulate(
        &mut pll,
        &mut wgen,
        (step_at / TS) as usize,
        |t| if t >= step_at { target_hz } else { NOMINAL_HZ },
    );

    let locked_err = (pll.estimated_frequency_hz() - NOMINAL_HZ as Real).abs();
    assert!(
        locked_err < FREQ_TOL_HZ,
        "PLL did not lock before step: {:.3} Hz",
        pll.estimated_frequency_hz()
    );

    // Phase 2: freq step applied, re-lock (0.5 → 1.0 s)
    // Simulate with t offset by step_at so freq_fn correctly gives target_hz
    for i in 0..((0.5 / TS) as usize) {
        let t = step_at + i as f64 * TS;
        wgen.set_frequency(freq_fn(t));
        wgen.update();
        let ab = wgen.signal().to_alphabeta();
        pll.update(ab);
    }

    let relock_err = (pll.estimated_frequency_hz() - target_hz as Real).abs();
    assert!(
        relock_err < FREQ_TOL_HZ,
        "PLL did not re-lock at {target_hz} Hz after step: got {:.3} Hz (err = {relock_err:.4})",
        pll.estimated_frequency_hz()
    );
    assert!(
        pll.dq().q().abs() < Q_TOL,
        "q-axis not zeroed after re-lock: {:.4}",
        pll.dq().q()
    );
}

#[test]
fn amplitude_step_does_not_disturb_frequency() {
    let mut pll = make_pll();
    let mut wgen = make_gen();

    // Lock at unit amplitude
    simulate(&mut pll, &mut wgen, (0.5 / TS) as usize, |_| NOMINAL_HZ);

    // Continue with 1.5× amplitude
    wgen.set_amplitude(1.5);
    simulate(&mut pll, &mut wgen, (0.5 / TS) as usize, |_| NOMINAL_HZ);

    let err = (pll.estimated_frequency_hz() - NOMINAL_HZ as Real).abs();
    assert!(
        err < FREQ_TOL_HZ,
        "Amplitude step disturbed frequency: {:.3} Hz",
        pll.estimated_frequency_hz()
    );
    assert!(
        (pll.dq().d() - 1.5).abs() < D_TOL,
        "d-axis did not track new amplitude: {:.4}",
        pll.dq().d()
    );
}

#[test]
fn frequency_ramp_response() {
    let ramp_delta: f64 = 10.0; // 50 → 60 Hz
    let ramp_start: f64 = 0.5;
    let ramp_end: f64 = 2.5;
    let target_hz = NOMINAL_HZ + ramp_delta;

    let mut pll = make_pll();
    let mut wgen = make_gen();

    // Total simulation: 3.0 s
    let total_steps = (3.0 / TS) as usize;
    for i in 0..total_steps {
        let t = i as f64 * TS;
        let progress =
            ((t - ramp_start) / (ramp_end - ramp_start)).clamp(0.0, 1.0);
        let freq = if t < ramp_start {
            NOMINAL_HZ
        } else {
            NOMINAL_HZ + ramp_delta * progress
        };
        wgen.set_frequency(freq);
        wgen.update();
        let ab = wgen.signal().to_alphabeta();
        pll.update(ab);
    }

    let err = (pll.estimated_frequency_hz() - target_hz as Real).abs();
    assert!(
        err < FREQ_TOL_HZ,
        "PLL did not track frequency ramp end at {target_hz} Hz: got {:.3} Hz",
        pll.estimated_frequency_hz()
    );
    assert!(
        pll.dq().q().abs() < Q_TOL,
        "q-axis not zeroed after ramp: {:.4}",
        pll.dq().q()
    );
}

#[test]
fn angle_phase_a_tracks_source() {
    let mut pll = make_pll();
    let mut wgen = make_gen();

    let steps = (0.5 / TS) as usize;
    let angle_inc = (TWO_PI as f64 * NOMINAL_HZ * TS) as Real;
    let mut source_angle = pawer::angle::AngleWrapped::default();

    for _ in 0..steps {
        wgen.set_frequency(NOMINAL_HZ);
        wgen.update();
        let ab = wgen.signal().to_alphabeta();
        pll.update(ab);
        source_angle += pawer::angle::AngleWrapped::new(angle_inc);
    }

    // The PLL's phase-A angle should track the source angle.
    // Compute wrapped difference: both angles in [0, 2π), so difference ∈ (-π, π].
    let pll_rad = pll.estimated_angle_phase_a().radians();
    let src_rad = source_angle.radians();
    let mut diff = pll_rad - src_rad;
    // Wrap to (-π, π]
    let pi = core::f32::consts::PI;
    if diff > pi {
        diff -= 2.0 * pi;
    } else if diff < -pi {
        diff += 2.0 * pi;
    }
    assert!(
        diff.abs() < 0.05,
        "Angle tracking error too large: {:.4} rad ({:.2}°)",
        diff,
        diff.to_degrees()
    );
}
