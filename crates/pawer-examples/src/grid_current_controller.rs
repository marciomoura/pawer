//! dq-frame current controller for grid-connected voltage-source inverters.
//!
//! Controls the grid current by computing an inverter voltage command in the
//! synchronous rotating (dq) reference frame. The controller comprises:
//!
//! - Two independent [`PiController`]s for the d and q axes.
//! - Optional **cross-coupling decoupling** (ωL terms).
//! - Optional **grid-voltage feedforward**.
//!
//! ```text
//!  i_dq_ref ──▸ ΔI ──▸ PI_d ──┐
//!               ▲              ├──▸ + decoupling + feedforward ──▸ v_dq_cmd
//!  i_dq_meas ──┘   PI_q ──┘
//! ```
//!
//! # Tuning
//!
//! The [`configure_imc`](GridCurrentController::configure_imc) method applies
//! Internal Model Control (IMC) tuning for an RL plant:
//!
//! ```text
//!   Kp = L / (2·τ_c)      Ki = R / (2·τ_c)
//! ```
//!
//! where `τ_c` is the desired closed-loop time constant.

use pawer::frames::Dq;
use pawer::pi_controller::PiController;
use pawer::types::Real;

/// dq-frame current controller for grid-connected inverters.
///
/// # Example
///
/// ```
/// use pawer::frames::Dq;
/// use pawer_examples::grid_current_controller::GridCurrentController;
///
/// let mut ctrl = GridCurrentController::new(100e-6);
/// ctrl.configure_imc(0.1, 2e-3, 1e-3);  // R, L, τ_c
///
/// let i_ref = Dq::new(5.0, 0.0);
/// let i_meas = Dq::new(0.0, 0.0);
/// let v_grid = Dq::new(325.0, 0.0);
/// let omega = 314.16;
///
/// let v_cmd = ctrl.update(i_ref, i_meas, v_grid, omega);
/// ```
pub struct GridCurrentController {
    pi_d: PiController,
    pi_q: PiController,

    // Plant parameters (for decoupling)
    inductance: Real,

    // Feature flags
    decoupling_enabled: bool,
    feedforward_enabled: bool,

    // Last computed outputs (for inspection)
    output: Dq<Real>,
    error: Dq<Real>,
}

impl GridCurrentController {
    /// Create a new controller with the given sampling time (seconds).
    pub fn new(sampling_time: f64) -> Self {
        let ts = sampling_time as Real;
        Self {
            pi_d: PiController::new(ts),
            pi_q: PiController::new(ts),
            inductance: 1e-3,
            decoupling_enabled: true,
            feedforward_enabled: true,
            output: Dq::default(),
            error: Dq::default(),
        }
    }

    // ── Configuration ─────────────────────────────────────────────────────

    /// Configure both PI controllers with proportional gain and integral
    /// time constant (`Ki = Kp / Ti`).
    pub fn configure(&mut self, kp: Real, ti: Real) {
        self.pi_d.configure_with_ti(kp, ti);
        self.pi_q.configure_with_ti(kp, ti);
    }

    /// Configure using IMC (Internal Model Control) tuning for an RL plant.
    ///
    /// - `resistance`: series resistance R (Ω)
    /// - `inductance`: series inductance L (H)
    /// - `tau_c`: desired closed-loop time constant (s)
    ///
    /// Resulting gains: `Kp = L / (2·τ_c)`, `Ti = L / R`.
    pub fn configure_imc(&mut self, resistance: Real, inductance: Real, tau_c: Real) {
        debug_assert!(tau_c > 0.0, "Time constant must be positive");
        debug_assert!(inductance > 0.0, "Inductance must be positive");
        debug_assert!(resistance > 0.0, "Resistance must be positive");

        let kp = inductance / (2.0 * tau_c);
        let ti = inductance / resistance;

        self.pi_d.configure_with_ti(kp, ti);
        self.pi_q.configure_with_ti(kp, ti);
        self.inductance = inductance;

        // Anti-windup: Kc = 1/Ti = R/L
        let kc = resistance / inductance;
        self.pi_d.set_antiwindup_gain(kc);
        self.pi_q.set_antiwindup_gain(kc);
    }

    /// Set the plant inductance used for cross-coupling decoupling.
    pub fn set_inductance(&mut self, inductance: Real) {
        debug_assert!(inductance > 0.0, "Inductance must be positive");
        self.inductance = inductance;
    }

    /// Set symmetric output voltage limits on both axes.
    pub fn set_output_limits(&mut self, v_max: Real) {
        self.pi_d.set_output_limits(-v_max, v_max);
        self.pi_q.set_output_limits(-v_max, v_max);
    }

    /// Enable or disable cross-coupling decoupling (ωL terms).
    pub fn enable_decoupling(&mut self, enable: bool) {
        self.decoupling_enabled = enable;
    }

    /// Enable or disable grid-voltage feedforward.
    pub fn enable_feedforward(&mut self, enable: bool) {
        self.feedforward_enabled = enable;
    }

    // ── Update ────────────────────────────────────────────────────────────

    /// Run one control cycle.
    ///
    /// - `i_ref`: dq current reference (A)
    /// - `i_meas`: measured dq current from Park transform (A)
    /// - `v_grid_dq`: grid voltage in dq frame (V) — used for feedforward
    /// - `omega`: angular frequency from PLL (rad/s) — used for decoupling
    ///
    /// Returns the inverter voltage command in the dq frame (V).
    pub fn update(
        &mut self,
        i_ref: Dq<Real>,
        i_meas: Dq<Real>,
        v_grid_dq: Dq<Real>,
        omega: Real,
    ) -> Dq<Real> {
        // Error
        let error_d = i_ref.d() - i_meas.d();
        let error_q = i_ref.q() - i_meas.q();
        self.error = Dq::new(error_d, error_q);

        // PI outputs
        let u_d = self.pi_d.update(error_d);
        let u_q = self.pi_q.update(error_q);

        // Cross-coupling decoupling: +ωL·i_q on d-axis, −ωL·i_d on q-axis
        let (decouple_d, decouple_q) = if self.decoupling_enabled {
            let wl = omega * self.inductance;
            (wl * i_meas.q(), -wl * i_meas.d())
        } else {
            (0.0, 0.0)
        };

        // Grid voltage feedforward
        let (ff_d, ff_q) = if self.feedforward_enabled {
            (v_grid_dq.d(), v_grid_dq.q())
        } else {
            (0.0, 0.0)
        };

        // Total voltage command
        self.output = Dq::new(u_d + decouple_d + ff_d, u_q + decouple_q + ff_q);
        self.output
    }

    /// Reset both PI controllers to zero.
    pub fn reset(&mut self) {
        self.pi_d.reset_to_zero();
        self.pi_q.reset_to_zero();
        self.output = Dq::default();
        self.error = Dq::default();
    }

    // ── Getters ───────────────────────────────────────────────────────────

    /// Last computed voltage command (dq).
    pub fn output(&self) -> Dq<Real> {
        self.output
    }

    /// Last computed current error (dq).
    pub fn error(&self) -> Dq<Real> {
        self.error
    }

    /// Whether cross-coupling decoupling is enabled.
    pub fn is_decoupling_enabled(&self) -> bool {
        self.decoupling_enabled
    }

    /// Whether grid-voltage feedforward is enabled.
    pub fn is_feedforward_enabled(&self) -> bool {
        self.feedforward_enabled
    }

    /// Configured inductance (H).
    pub fn inductance(&self) -> Real {
        self.inductance
    }

    /// Immutable access to the d-axis PI controller.
    pub fn pi_d(&self) -> &PiController {
        &self.pi_d
    }

    /// Immutable access to the q-axis PI controller.
    pub fn pi_q(&self) -> &PiController {
        &self.pi_q
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TS: f64 = 100e-6;
    const R: Real = 0.1;
    const L: Real = 2e-3;
    const TAU_C: Real = 1e-3;
    const EPS: Real = 1e-3;

    fn approx(a: Real, b: Real, tol: Real) -> bool {
        (a - b).abs() < tol
    }

    fn make_controller() -> GridCurrentController {
        let mut ctrl = GridCurrentController::new(TS);
        ctrl.configure_imc(R, L, TAU_C);
        ctrl.set_output_limits(400.0);
        ctrl
    }

    #[test]
    fn zero_reference_zero_output_without_feedforward() {
        let mut ctrl = make_controller();
        ctrl.enable_feedforward(false);
        ctrl.enable_decoupling(false);

        let zero = Dq::new(0.0, 0.0);
        let v = ctrl.update(zero, zero, zero, 0.0);

        assert!(approx(v.d(), 0.0, EPS));
        assert!(approx(v.q(), 0.0, EPS));
    }

    #[test]
    fn positive_d_error_produces_positive_d_voltage() {
        let mut ctrl = make_controller();
        ctrl.enable_feedforward(false);
        ctrl.enable_decoupling(false);

        let i_ref = Dq::new(10.0, 0.0);
        let i_meas = Dq::new(0.0, 0.0);
        let v = ctrl.update(i_ref, i_meas, Dq::default(), 0.0);

        assert!(v.d() > 0.0, "Expected positive v_d, got {}", v.d());
        assert!(approx(v.q(), 0.0, EPS), "Expected ~zero v_q, got {}", v.q());
    }

    #[test]
    fn feedforward_passes_through() {
        let mut ctrl = make_controller();
        ctrl.enable_feedforward(true);
        ctrl.enable_decoupling(false);

        let zero = Dq::new(0.0, 0.0);
        let v_grid = Dq::new(100.0, -50.0);
        let v = ctrl.update(zero, zero, v_grid, 0.0);

        // With zero error and zero current, output ≈ feedforward
        assert!(approx(v.d(), 100.0, EPS));
        assert!(approx(v.q(), -50.0, EPS));
    }

    #[test]
    fn decoupling_adds_cross_terms() {
        let mut ctrl = make_controller();
        ctrl.enable_feedforward(false);
        ctrl.enable_decoupling(true);

        let i_meas = Dq::new(0.0, 10.0); // only q-axis current
        let zero = Dq::new(0.0, 0.0);
        let omega: Real = 314.16;
        let v = ctrl.update(zero, i_meas, zero, omega);

        // Decoupling on d-axis: +ωL·i_q
        let expected_decouple_d = omega * L * 10.0;
        // PI sees error_d = 0 - 0 = 0 (i_ref_d = 0), but error_q = 0 - 10 = -10
        // So v_d should be ≈ decoupling term (PI_d output ≈ 0)
        assert!(
            v.d() > expected_decouple_d * 0.9,
            "Expected d-axis decoupling ~{expected_decouple_d}, got {}",
            v.d()
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut ctrl = make_controller();
        ctrl.enable_feedforward(false);
        ctrl.enable_decoupling(false);

        let i_ref = Dq::new(10.0, 5.0);
        let zero = Dq::new(0.0, 0.0);
        for _ in 0..100 {
            ctrl.update(i_ref, zero, zero, 0.0);
        }
        assert!(ctrl.output().d().abs() > EPS);

        ctrl.reset();
        let v = ctrl.update(zero, zero, zero, 0.0);
        assert!(approx(v.d(), 0.0, EPS));
        assert!(approx(v.q(), 0.0, EPS));
    }

    #[test]
    fn imc_tuning_gains() {
        let ctrl = make_controller();
        // Kp = L / (2·τ_c) = 2e-3 / 2e-3 = 1.0
        assert!(approx(ctrl.pi_d().kp(), 1.0, EPS));
        // Ti = L / R = 2e-3 / 0.1 = 0.02  →  Ki = Kp / Ti = 1.0 / 0.02 = 50.0
        assert!(approx(ctrl.pi_d().ki(), 50.0, EPS));
    }
}
