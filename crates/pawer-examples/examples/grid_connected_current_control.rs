/// Example: Grid-connected current control in the dq frame (per-unit system).
///
/// Demonstrates a complete vector-current-control loop using the
/// [`GridCurrentController`] to regulate grid current through a series RL
/// impedance (the [`GridModel`]).  All user-facing parameters and logged
/// signals are in the **per-unit system** based on configurable base voltage
/// and base current.
///
/// ```text
///  i_dq_ref ──▸ GridCurrentController ──▸ inv.Park ──▸ GridModel (RL + V_grid)
///   (pu)           ▲        ▲              (SI)          (SI)
///                  │        └── ω, v_grid_dq (from PLL) ──┤
///                  └── Park(i_αβ, θ) ◂────────────────────┘
/// ```
///
/// ## Per-unit base quantities
///
/// | Base        | Default | Derived quantities                     |
/// |-------------|---------|----------------------------------------|
/// | V_base      | 325.0 V | (peak phase voltage ≈ 230 Vrms)       |
/// | I_base      | 100.0 A | (rated peak current)                   |
/// | Z_base      | —       | V_base / I_base                        |
/// | ω_base      | —       | 2π · grid_freq_hz                      |
/// | L_base      | —       | Z_base / ω_base                        |
///
/// Run with:
/// ```bash
/// cargo run -p pawer-examples --example grid_connected_current_control
/// ```
///
/// Suggested session:
/// ```text
/// /simulate 0.1              # PLL locks, currents at zero
/// /plot i_d i_q i_d_ref i_q_ref
///
/// /set i_d_ref 0.5           # step d-axis to 0.5 pu (50 A)
/// /simulate 0.1
/// /plot i_d i_q i_d_ref i_q_ref
/// /plot v_d_cmd v_q_cmd
///
/// /set i_q_ref -0.3          # step q-axis to -0.3 pu
/// /simulate 0.1
/// /plot i_d i_q
///
/// /set enable_decoupling 0.0 # disable cross-coupling decoupling
/// /set i_d_ref 1.0
/// /simulate 0.1
/// /plot i_d i_q              # observe coupling transient
///
/// /set enable_decoupling 1.0 # re-enable
/// /simulate 0.2
/// /plot i_d i_q
///
/// /save results.csv
/// /quit
/// ```
///
/// | Parameter           | Default | Unit | Meaning                            |
/// |---------------------|---------|------|------------------------------------|
/// | `i_d_ref`           | 0.0     | pu   | d-axis current setpoint            |
/// | `i_q_ref`           | 0.0     | pu   | q-axis current setpoint            |
/// | `grid_freq_hz`      | 50.0    | Hz   | Grid ideal-source frequency        |
/// | `grid_amplitude`    | 1.0     | pu   | Grid peak voltage                  |
/// | `resistance`        | 0.031   | pu   | Series resistance                  |
/// | `inductance`        | 0.194   | pu   | Series inductance                  |
/// | `enable_decoupling` | 1.0     | —    | 1.0 = ωL decoupling on             |
/// | `enable_feedforward`| 1.0     | —    | 1.0 = grid voltage feedforward on  |
/// | `base_voltage`      | 325.0   | V    | Base peak phase voltage            |
/// | `base_current`      | 100.0   | A    | Base peak current                  |
use pawer::constants::TWO_PI;
use pawer::srf_pll::{GAINS_50HZ_CROSSOVER, SrfPll};
use pawer::types::Real;
use pawer_examples::grid_current_controller::GridCurrentController;
use pawer_examples::grid_model::GridModel;
use pawer_sim::prelude::*;

use pawer::frames::Dq;

const SAMPLING_TIME: f64 = 100e-6; // 10 kHz

// Default SI plant parameters (used to compute default pu values)
const DEFAULT_R_SI: Real = 0.1; // 0.1 Ω
const DEFAULT_L_SI: Real = 2e-3; // 2 mH
const DEFAULT_V_BASE: Real = 325.0; // V peak (≈ 230 Vrms)
const DEFAULT_I_BASE: Real = 100.0; // A peak
const DEFAULT_FREQ_HZ: Real = 50.0;

// ── Signal handles ────────────────────────────────────────────────────────────

#[derive(Default)]
struct Signals {
    // Current tracking (pu)
    i_d: SignalId,
    i_q: SignalId,
    i_d_ref: SignalId,
    i_q_ref: SignalId,
    error_d: SignalId,
    error_q: SignalId,
    // Voltage commands (pu, dq)
    v_d_cmd: SignalId,
    v_q_cmd: SignalId,
    // Inverter output (pu, αβ)
    v_inv_alpha: SignalId,
    v_inv_beta: SignalId,
    // Grid current (pu, αβ)
    i_alpha: SignalId,
    i_beta: SignalId,
    // Grid voltage feedforward (pu, dq)
    v_grid_d: SignalId,
    v_grid_q: SignalId,
    // PLL state
    pll_freq_hz: SignalId,
    pll_angle_rad: SignalId,
}

// ── Per-unit base helper ──────────────────────────────────────────────────────

struct PuBases {
    v_base: Real,
    i_base: Real,
    z_base: Real,
    l_base: Real,
}

impl PuBases {
    fn compute(v_base: Real, i_base: Real, freq_hz: Real) -> Self {
        let z_base = v_base / i_base;
        let omega_base = TWO_PI * freq_hz;
        let l_base = z_base / omega_base;
        Self {
            v_base,
            i_base,
            z_base,
            l_base,
        }
    }

    fn r_to_si(&self, r_pu: Real) -> Real {
        r_pu * self.z_base
    }
    fn l_to_si(&self, l_pu: Real) -> Real {
        l_pu * self.l_base
    }
    fn v_to_si(&self, v_pu: Real) -> Real {
        v_pu * self.v_base
    }
    fn i_to_si(&self, i_pu: Real) -> Real {
        i_pu * self.i_base
    }
    fn v_to_pu(&self, v_si: Real) -> Real {
        v_si / self.v_base
    }
    fn i_to_pu(&self, i_si: Real) -> Real {
        i_si / self.i_base
    }
}

// ── Scenario state ────────────────────────────────────────────────────────────

struct GridConnectedCurrentControl {
    grid: GridModel,
    pll: SrfPll,
    controller: GridCurrentController,
    bases: PuBases,
    sigs: Signals,
}

impl GridConnectedCurrentControl {
    fn new() -> Self {
        Self {
            grid: GridModel::new(SAMPLING_TIME),
            pll: SrfPll::new(SAMPLING_TIME as Real),
            controller: GridCurrentController::new(SAMPLING_TIME),
            bases: PuBases::compute(DEFAULT_V_BASE, DEFAULT_I_BASE, DEFAULT_FREQ_HZ),
            sigs: Signals::default(),
        }
    }

    /// Reconfigure plant and controller from current pu parameters.
    fn apply_plant_config(&mut self, r_pu: Real, l_pu: Real) {
        let r_si = self.bases.r_to_si(r_pu);
        let l_si = self.bases.l_to_si(l_pu);
        self.grid.configure(r_si, l_si);
        self.controller.configure_imc(r_si, l_si, 1e-3);
        self.controller.set_output_limits(self.bases.v_base * 1.2);
    }
}

// ── Scenario implementation ───────────────────────────────────────────────────

impl Scenario for GridConnectedCurrentControl {
    fn init(&mut self, ctx: &mut SimContext) {
        // Base quantities
        ctx.set_param("base_voltage", DEFAULT_V_BASE as f64);
        ctx.set_param("base_current", DEFAULT_I_BASE as f64);

        // Per-unit parameters
        let r_pu = DEFAULT_R_SI / self.bases.z_base;
        let l_pu = DEFAULT_L_SI / self.bases.l_base;
        ctx.set_param("i_d_ref", 0.0);
        ctx.set_param("i_q_ref", 0.0);
        ctx.set_param("grid_freq_hz", DEFAULT_FREQ_HZ as f64);
        ctx.set_param("grid_amplitude", 1.0); // 1.0 pu
        ctx.set_param("resistance", r_pu as f64);
        ctx.set_param("inductance", l_pu as f64);
        ctx.set_param("enable_decoupling", 1.0);
        ctx.set_param("enable_feedforward", 1.0);

        // Register signals
        self.sigs.i_d = ctx.register_signal("i_d");
        self.sigs.i_q = ctx.register_signal("i_q");
        self.sigs.i_d_ref = ctx.register_signal("i_d_ref");
        self.sigs.i_q_ref = ctx.register_signal("i_q_ref");
        self.sigs.error_d = ctx.register_signal("error_d");
        self.sigs.error_q = ctx.register_signal("error_q");
        self.sigs.v_d_cmd = ctx.register_signal("v_d_cmd");
        self.sigs.v_q_cmd = ctx.register_signal("v_q_cmd");
        self.sigs.v_inv_alpha = ctx.register_signal("v_inv_alpha");
        self.sigs.v_inv_beta = ctx.register_signal("v_inv_beta");
        self.sigs.i_alpha = ctx.register_signal("i_alpha");
        self.sigs.i_beta = ctx.register_signal("i_beta");
        self.sigs.v_grid_d = ctx.register_signal("v_grid_d");
        self.sigs.v_grid_q = ctx.register_signal("v_grid_q");
        self.sigs.pll_freq_hz = ctx.register_signal("pll_freq_hz");
        self.sigs.pll_angle_rad = ctx.register_signal("pll_angle_rad");

        // Configure plant (SI internally)
        self.apply_plant_config(r_pu, l_pu);
        self.grid.set_grid_frequency(DEFAULT_FREQ_HZ as f64);
        self.grid.set_grid_amplitude(DEFAULT_V_BASE as f64);

        // Configure PLL
        self.pll.configure_nominal_frequency(DEFAULT_FREQ_HZ);
        self.pll
            .configure_pi_controller(GAINS_50HZ_CROSSOVER.kp, GAINS_50HZ_CROSSOVER.ti);
        self.pll.reset_with_frequency(1.0);
    }

    fn step(&mut self, ctx: &mut SimContext) {
        let i_d_ref_pu = ctx.param("i_d_ref");
        let i_q_ref_pu = ctx.param("i_q_ref");
        let decoupling_on = ctx.param("enable_decoupling") > 0.5;
        let feedforward_on = ctx.param("enable_feedforward") > 0.5;

        // Convert current references from pu to SI
        let i_d_ref_si = self.bases.i_to_si(i_d_ref_pu);
        let i_q_ref_si = self.bases.i_to_si(i_q_ref_pu);

        // ── Measure (SI) ──────────────────────────────────────────────────
        let i_ab = self.grid.current();
        let v_grid_ab = self.grid.grid_voltage();

        // ── PLL (expects pu input) ────────────────────────────────────────
        // SrfPll expects an amplitude-≈1.0 per-unit vector. Feeding SI
        // voltage (e.g. 325 V) would cause the PI to see a 325× larger error,
        // diverging the frequency estimate.
        let v_grid_ab_pu = v_grid_ab * (1.0 / self.bases.v_base);
        self.pll.update(v_grid_ab_pu);
        let theta = self.pll.estimated_angle_phase_a();
        let omega = self.pll.estimated_angular_frequency();

        // ── Park transform (SI) ───────────────────────────────────────────
        let i_dq = i_ab.to_dq(theta);
        let v_grid_dq = v_grid_ab.to_dq(theta);

        // ── Current controller (SI) ───────────────────────────────────────
        self.controller.enable_decoupling(decoupling_on);
        self.controller.enable_feedforward(feedforward_on);

        let i_ref_si = Dq::new(i_d_ref_si, i_q_ref_si);
        let v_dq_cmd = self.controller.update(i_ref_si, i_dq, v_grid_dq, omega);
        let error = self.controller.error();

        // ── Inverse Park → apply to plant (SI) ───────────────────────────
        let v_ab_cmd = v_dq_cmd.to_alphabeta(theta);
        self.grid.update(v_ab_cmd);

        // ── Log signals (pu) ──────────────────────────────────────────────
        ctx.log_id(self.sigs.i_d, self.bases.i_to_pu(i_dq.d()));
        ctx.log_id(self.sigs.i_q, self.bases.i_to_pu(i_dq.q()));
        ctx.log_id(self.sigs.i_d_ref, i_d_ref_pu);
        ctx.log_id(self.sigs.i_q_ref, i_q_ref_pu);
        ctx.log_id(self.sigs.error_d, self.bases.i_to_pu(error.d()));
        ctx.log_id(self.sigs.error_q, self.bases.i_to_pu(error.q()));
        ctx.log_id(self.sigs.v_d_cmd, self.bases.v_to_pu(v_dq_cmd.d()));
        ctx.log_id(self.sigs.v_q_cmd, self.bases.v_to_pu(v_dq_cmd.q()));
        ctx.log_id(self.sigs.v_inv_alpha, self.bases.v_to_pu(v_ab_cmd.alpha()));
        ctx.log_id(self.sigs.v_inv_beta, self.bases.v_to_pu(v_ab_cmd.beta()));
        ctx.log_id(self.sigs.i_alpha, self.bases.i_to_pu(i_ab.alpha()));
        ctx.log_id(self.sigs.i_beta, self.bases.i_to_pu(i_ab.beta()));
        ctx.log_id(self.sigs.v_grid_d, self.bases.v_to_pu(v_grid_dq.d()));
        ctx.log_id(self.sigs.v_grid_q, self.bases.v_to_pu(v_grid_dq.q()));
        ctx.log_id(self.sigs.pll_freq_hz, self.pll.estimated_frequency_hz());
        ctx.log_id(
            self.sigs.pll_angle_rad,
            self.pll.estimated_angle_phase_a().radians(),
        );
    }

    fn on_param_change(&mut self, name: &str, value: f64, ctx: &SimContext) {
        match name {
            "base_voltage" | "base_current" | "grid_freq_hz" => {
                // Recompute bases
                let v_base = ctx.param("base_voltage");
                let i_base = ctx.param("base_current");
                let freq = ctx.param("grid_freq_hz");
                self.bases = PuBases::compute(v_base, i_base, freq);

                // Reapply plant config with current pu values
                let r_pu = ctx.param("resistance");
                let l_pu = ctx.param("inductance");
                self.apply_plant_config(r_pu, l_pu);

                if name == "grid_freq_hz" {
                    self.grid.set_grid_frequency(value);
                    self.pll.configure_nominal_frequency(value as Real);
                }

                // Reapply grid amplitude (pu → SI)
                let amp_pu = ctx.param("grid_amplitude");
                self.grid
                    .set_grid_amplitude(self.bases.v_to_si(amp_pu) as f64);
            }
            "grid_amplitude" => {
                let amp_si = self.bases.v_to_si(value as Real);
                self.grid.set_grid_amplitude(amp_si as f64);
            }
            "resistance" | "inductance" => {
                let r_pu = ctx.param("resistance");
                let l_pu = ctx.param("inductance");
                self.apply_plant_config(r_pu, l_pu);
            }
            _ => {}
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let scenario = GridConnectedCurrentControl::new();
    let mut cli = SimCli::new(Box::new(scenario), SAMPLING_TIME);
    cli.run();
}
