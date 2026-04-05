/// Example: Equivalent grid model — RL impedance with ideal voltage source.
///
/// Demonstrates how to connect an inverter voltage source to the
/// [`GridModel`](pawer_examples::grid_model::GridModel) and observe the
/// resulting grid current in the αβ frame.
///
/// Run with:
/// ```bash
/// cargo run -p pawer-examples --example grid_model
/// ```
///
/// Suggested session:
/// ```text
/// /simulate 0.2              # observe current build-up and steady state
/// /plot i_alpha i_beta
/// /plot v_inv_alpha v_grid_alpha
///
/// /set inv_amplitude 1.2     # increase inverter voltage → more current
/// /simulate 0.2
/// /plot i_alpha i_beta i_magnitude
///
/// /set grid_freq_hz 50.5     # small grid frequency deviation
/// /simulate 0.5
/// /plot i_alpha i_beta
///
/// /save results.csv
/// /quit
/// ```
///
/// | Parameter       | Default | Meaning                                       |
/// |-----------------|---------|-----------------------------------------------|
/// | `grid_freq_hz`  | 50.0    | Grid ideal-source frequency (Hz)              |
/// | `grid_amplitude`| 1.0     | Grid ideal-source peak amplitude (pu)         |
/// | `inv_freq_hz`   | 50.0    | Inverter voltage frequency (Hz)               |
/// | `inv_amplitude` | 1.0     | Inverter voltage peak amplitude (pu)          |
/// | `resistance`    | 0.1     | Series resistance (Ω)                         |
/// | `inductance`    | 2e-3    | Series inductance (H)                         |
use pawer::types::Real;
use pawer_examples::grid_model::GridModel;
use pawer_examples::waveform_gen::ThreePhaseGenerator;
use pawer_sim::prelude::*;

const SAMPLING_TIME: f64 = 100e-6; // 10 kHz

// ── Signal handles ────────────────────────────────────────────────────────────

#[derive(Default)]
struct Signals {
    v_inv_alpha: SignalId,
    v_inv_beta: SignalId,
    v_grid_alpha: SignalId,
    v_grid_beta: SignalId,
    i_alpha: SignalId,
    i_beta: SignalId,
    i_magnitude: SignalId,
    grid_freq_hz: SignalId,
    inv_freq_hz: SignalId,
}

// ── Scenario state ────────────────────────────────────────────────────────────

struct GridModelScenario {
    inverter: ThreePhaseGenerator,
    grid: GridModel,
    sigs: Signals,
}

impl GridModelScenario {
    fn new() -> Self {
        Self {
            inverter: ThreePhaseGenerator::new(SAMPLING_TIME),
            grid: GridModel::new(SAMPLING_TIME),
            sigs: Signals::default(),
        }
    }
}

// ── Scenario implementation ───────────────────────────────────────────────────

impl Scenario for GridModelScenario {
    fn init(&mut self, ctx: &mut SimContext) {
        // Parameters
        ctx.set_param("grid_freq_hz", 50.0);
        ctx.set_param("grid_amplitude", 1.0);
        ctx.set_param("inv_freq_hz", 50.0);
        ctx.set_param("inv_amplitude", 1.0);
        ctx.set_param("resistance", 0.1);
        ctx.set_param("inductance", 2e-3);

        // Register signals
        self.sigs.v_inv_alpha = ctx.register_signal("v_inv_alpha");
        self.sigs.v_inv_beta = ctx.register_signal("v_inv_beta");
        self.sigs.v_grid_alpha = ctx.register_signal("v_grid_alpha");
        self.sigs.v_grid_beta = ctx.register_signal("v_grid_beta");
        self.sigs.i_alpha = ctx.register_signal("i_alpha");
        self.sigs.i_beta = ctx.register_signal("i_beta");
        self.sigs.i_magnitude = ctx.register_signal("i_magnitude");
        self.sigs.grid_freq_hz = ctx.register_signal("grid_freq_hz");
        self.sigs.inv_freq_hz = ctx.register_signal("inv_freq_hz");

        // Configure blocks
        self.grid.configure(0.1, 2e-3);
        self.grid.set_grid_frequency(50.0);
        self.grid.set_grid_amplitude(1.0);

        self.inverter.set_frequency(50.0);
        self.inverter.set_amplitude(1.0);
    }

    fn step(&mut self, ctx: &mut SimContext) {
        // Generate inverter voltage and convert to αβ
        self.inverter.update();
        let v_inv = self.inverter.signal().to_alphabeta();

        // Update grid model (RL + ideal source)
        self.grid.update(v_inv);

        let i = self.grid.current();
        let v_grid = self.grid.grid_voltage();

        // Log signals
        ctx.log_id(self.sigs.v_inv_alpha, v_inv.alpha());
        ctx.log_id(self.sigs.v_inv_beta, v_inv.beta());
        ctx.log_id(self.sigs.v_grid_alpha, v_grid.alpha());
        ctx.log_id(self.sigs.v_grid_beta, v_grid.beta());
        ctx.log_id(self.sigs.i_alpha, i.alpha());
        ctx.log_id(self.sigs.i_beta, i.beta());
        ctx.log_id(
            self.sigs.i_magnitude,
            (i.alpha() * i.alpha() + i.beta() * i.beta()).sqrt(),
        );
        ctx.log_id(self.sigs.grid_freq_hz, self.grid.voltage_source().frequency() as Real);
        ctx.log_id(self.sigs.inv_freq_hz, self.inverter.frequency() as Real);
    }

    fn on_param_change(&mut self, name: &str, value: f64, ctx: &SimContext) {
        match name {
            "grid_freq_hz" => {
                self.grid.set_grid_frequency(value);
            }
            "grid_amplitude" => {
                self.grid.set_grid_amplitude(value);
            }
            "inv_freq_hz" => {
                self.inverter.set_frequency(value);
            }
            "inv_amplitude" => {
                self.inverter.set_amplitude(value);
            }
            "resistance" => {
                let l = ctx.param_f64("inductance");
                self.grid.configure(value as Real, l as Real);
            }
            "inductance" => {
                let r = ctx.param_f64("resistance");
                self.grid.configure(r as Real, value as Real);
            }
            _ => {}
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let scenario = GridModelScenario::new();
    let mut cli = SimCli::new(Box::new(scenario), SAMPLING_TIME);
    cli.run();
}
