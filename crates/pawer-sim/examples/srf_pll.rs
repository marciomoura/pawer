/// Example: SRF-PLL lock-on and frequency-step response.
///
/// Demonstrates how to feed a three-phase waveform into a [`SrfPll`] and
/// observe the estimated frequency, dq components, and phase angle through
/// the interactive CLI.
///
/// Run with:
/// ```bash
/// cargo run -p pawer-sim --example srf_pll
/// ```
///
/// Suggested session:
/// ```text
/// /simulate 0.5              # PLL locks at 50 Hz
/// /plot freq_hz gen_freq_hz dq_d dq_q
///
/// /set freq_step 10.0        # step to 60 Hz — fires immediately (t >= freq_step_at=0.5)
/// /simulate 0.5              # observe re-lock
/// /plot freq_hz gen_freq_hz dq_d dq_q
///
/// /reset
/// /set freq_step 10.0        # same step, but deferred
/// /set freq_step_at 1.5      # step fires at t = 1.5 s
/// /simulate 3.0              # lock, step, re-lock in one run
/// /plot freq_hz gen_freq_hz
///
/// /save results.csv
/// /quit
/// ```
///
/// All parameters can be changed at any point and take effect immediately:
///
/// | Parameter      | Default | Meaning                                      |
/// |----------------|---------|----------------------------------------------|
/// | `nominal_hz`   | 50.0    | Base signal frequency (Hz)                   |
/// | `amplitude`    | 1.0     | Peak amplitude (pu)                          |
/// | `freq_step`    | 0.0     | Frequency delta applied at `freq_step_at`    |
/// | `freq_step_at` | 0.5     | Absolute simulation time for the step (s)    |
use pawer::constants::TWO_PI;
use pawer::srf_pll::{SrfPll, GAINS_50HZ_CROSSOVER};
use pawer::types::Real;
use pawer_sim::prelude::*;
use pawer_sim::waveform_gen::ThreePhaseGenerator;

const NOMINAL_HZ: f64 = 50.0;
const SAMPLING_TIME: f64 = 100e-6; // 10 kHz

// ── Scenario state ────────────────────────────────────────────────────────────

struct SrfPllScenario {
    pll: SrfPll,
    wgen: ThreePhaseGenerator,
}

impl SrfPllScenario {
    fn new() -> Self {
        Self {
            pll: SrfPll::new(SAMPLING_TIME as Real),
            wgen: ThreePhaseGenerator::new(SAMPLING_TIME),
        }
    }
}

// ── Scenario implementation ───────────────────────────────────────────────────

impl Scenario for SrfPllScenario {
    fn init(&mut self, ctx: &mut SimContext) {
        ctx.set_param("nominal_hz", NOMINAL_HZ);
        ctx.set_param("amplitude", 1.0);
        ctx.set_param("freq_step", 0.0);    // +Hz added after freq_step_at
        ctx.set_param("freq_step_at", 0.5); // absolute simulation time for step

        self.pll.configure_nominal_frequency(NOMINAL_HZ as Real);
        self.pll.configure_pi_controller(GAINS_50HZ_CROSSOVER.kp, GAINS_50HZ_CROSSOVER.ti);
        self.pll.reset_with_frequency(1.0);

        self.wgen.set_frequency(NOMINAL_HZ);
        self.wgen.set_amplitude(1.0);
    }

    fn step(&mut self, ctx: &mut SimContext) {
        let t = ctx.time();
        let nominal = ctx.param_f64("nominal_hz");
        let freq_step = ctx.param_f64("freq_step");
        let freq_step_at = ctx.param_f64("freq_step_at");

        // Compute target frequency purely from current time and parameters.
        // This is stateless and always correct regardless of when the user
        // sets parameters relative to /simulate calls.
        let current_freq = if freq_step.abs() > 1e-9 && t >= freq_step_at {
            nominal + freq_step
        } else {
            nominal
        };

        self.wgen.set_frequency(current_freq);
        self.wgen.set_amplitude(ctx.param_f64("amplitude"));

        // ── Simulation step ───────────────────────────────────────────────
        self.wgen.update();
        let signal_ab = self.wgen.signal().to_alphabeta();
        self.pll.update(signal_ab);

        // ── Logging ───────────────────────────────────────────────────────
        ctx.log("freq_hz", self.pll.estimated_frequency_hz());
        ctx.log("freq_pu", self.pll.estimated_frequency_pu());
        ctx.log("omega_rad_s", self.pll.estimated_angular_frequency());
        ctx.log("dq_d", self.pll.dq().d());
        ctx.log("dq_q", self.pll.dq().q());

        let angle_pu = self.pll.estimated_angle_phase_a().radians() / TWO_PI;
        ctx.log("angle_pu", angle_pu);
        ctx.log("angle_rad", self.pll.estimated_angle_phase_a().radians());

        ctx.log("signal_alpha", signal_ab.alpha());
        ctx.log("signal_beta", signal_ab.beta());
        ctx.log("gen_freq_hz", current_freq as Real);
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let scenario = SrfPllScenario::new();
    let mut cli = SimCli::new(Box::new(scenario), SAMPLING_TIME);
    cli.run();
}
