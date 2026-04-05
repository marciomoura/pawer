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
/// /set freq_step 10.0        # on_param_change reacts: generator → 60 Hz
/// /simulate 0.5              # observe re-lock
/// /plot freq_hz gen_freq_hz dq_d dq_q
///
/// /reset
/// /set nominal_hz 60.0       # event-driven: instantly reconfigures PLL + generator
/// /simulate 1.0
/// /plot freq_hz gen_freq_hz
///
/// /save results.csv
/// /quit
/// ```
///
/// All parameters can be changed at any point and take effect immediately
/// via the `on_param_change` callback — no if-else logic in `step()`.
///
/// | Parameter      | Default | Meaning                                      |
/// |----------------|---------|----------------------------------------------|
/// | `nominal_hz`   | 50.0    | Base signal frequency (Hz)                   |
/// | `amplitude`    | 1.0     | Peak amplitude (pu)                          |
/// | `freq_step`    | 0.0     | Frequency delta added to nominal (Hz)        |
use pawer::constants::TWO_PI;
use pawer::srf_pll::{SrfPll, GAINS_50HZ_CROSSOVER};
use pawer::types::Real;
use pawer_sim::prelude::*;
use pawer_examples::waveform_gen::ThreePhaseGenerator;

const NOMINAL_HZ: f64 = 50.0;
const SAMPLING_TIME: f64 = 100e-6; // 10 kHz

// ── Signal handles ────────────────────────────────────────────────────────────

#[derive(Default)]
struct Signals {
    freq_hz: SignalId,
    freq_pu: SignalId,
    omega_rad_s: SignalId,
    dq_d: SignalId,
    dq_q: SignalId,
    angle_pu: SignalId,
    angle_rad: SignalId,
    signal_alpha: SignalId,
    signal_beta: SignalId,
    gen_freq_hz: SignalId,
}

// ── Scenario state ────────────────────────────────────────────────────────────

struct SrfPllScenario {
    pll: SrfPll,
    wgen: ThreePhaseGenerator,
    sigs: Signals,
}

impl SrfPllScenario {
    fn new() -> Self {
        Self {
            pll: SrfPll::new(SAMPLING_TIME as Real),
            wgen: ThreePhaseGenerator::new(SAMPLING_TIME),
            sigs: Signals::default(),
        }
    }
}

// ── Scenario implementation ───────────────────────────────────────────────────

impl Scenario for SrfPllScenario {
    fn init(&mut self, ctx: &mut SimContext) {
        // Parameters
        ctx.set_param("nominal_hz", NOMINAL_HZ);
        ctx.set_param("amplitude", 1.0);
        ctx.set_param("freq_step", 0.0);

        // Register signals upfront → fast indexed logging in step()
        self.sigs.freq_hz = ctx.register_signal("freq_hz");
        self.sigs.freq_pu = ctx.register_signal("freq_pu");
        self.sigs.omega_rad_s = ctx.register_signal("omega_rad_s");
        self.sigs.dq_d = ctx.register_signal("dq_d");
        self.sigs.dq_q = ctx.register_signal("dq_q");
        self.sigs.angle_pu = ctx.register_signal("angle_pu");
        self.sigs.angle_rad = ctx.register_signal("angle_rad");
        self.sigs.signal_alpha = ctx.register_signal("signal_alpha");
        self.sigs.signal_beta = ctx.register_signal("signal_beta");
        self.sigs.gen_freq_hz = ctx.register_signal("gen_freq_hz");

        // Configure blocks
        self.pll.configure_nominal_frequency(NOMINAL_HZ as Real);
        self.pll.configure_pi_controller(GAINS_50HZ_CROSSOVER.kp, GAINS_50HZ_CROSSOVER.ti);
        self.pll.reset_with_frequency(1.0);

        self.wgen.set_frequency(NOMINAL_HZ);
        self.wgen.set_amplitude(1.0);
    }

    fn step(&mut self, ctx: &mut SimContext) {
        // Pure computation — no parameter polling, no string-based logging
        self.wgen.update();
        let signal_ab = self.wgen.signal().to_alphabeta();
        self.pll.update(signal_ab);

        // Fast indexed logging (no allocations, no HashMap lookups)
        ctx.log_id(self.sigs.freq_hz, self.pll.estimated_frequency_hz());
        ctx.log_id(self.sigs.freq_pu, self.pll.estimated_frequency_pu());
        ctx.log_id(self.sigs.omega_rad_s, self.pll.estimated_angular_frequency());
        ctx.log_id(self.sigs.dq_d, self.pll.dq().d());
        ctx.log_id(self.sigs.dq_q, self.pll.dq().q());

        let angle_pu = self.pll.estimated_angle_phase_a().radians() / TWO_PI;
        ctx.log_id(self.sigs.angle_pu, angle_pu);
        ctx.log_id(self.sigs.angle_rad, self.pll.estimated_angle_phase_a().radians());

        ctx.log_id(self.sigs.signal_alpha, signal_ab.alpha());
        ctx.log_id(self.sigs.signal_beta, signal_ab.beta());
        ctx.log_id(self.sigs.gen_freq_hz, self.wgen.frequency() as Real);
    }

    fn on_param_change(&mut self, name: &str, value: f64, ctx: &SimContext) {
        match name {
            "nominal_hz" => {
                self.pll.configure_nominal_frequency(value as Real);
                let full_freq = value + ctx.param_f64("freq_step");
                self.wgen.set_frequency(full_freq);
            }
            "amplitude" => {
                self.wgen.set_amplitude(value);
            }
            "freq_step" => {
                let nominal = ctx.param_f64("nominal_hz");
                self.wgen.set_frequency(nominal + value);
            }
            _ => {}
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let scenario = SrfPllScenario::new();
    let mut cli = SimCli::new(Box::new(scenario), SAMPLING_TIME);
    cli.run();
}
