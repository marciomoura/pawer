//! Scalar PI controller with output clamping, anti-windup (back-calculation),
//! and integrator clamping.
//!
//! The controller implements the parallel form:
//!
//! ```text
//! u = Kp·e + ∫(Ki·e + Kc·(u_sat − u_unsat)) dt
//! ```
//!
//! where `Kc` is the back-calculation anti-windup gain and the integral is
//! discretised with a forward-Euler [`Integrator`].

use crate::integrator::Integrator;
use crate::types::Real;

/// Scalar PI controller operating on [`Real`] values.
pub struct PiController {
    kp: Real,
    ki: Real,
    integrator: Integrator,

    // Output limits
    output_min: Real,
    output_max: Real,
    has_output_limits: bool,

    // Anti-windup
    kc: Real,
    integrator_clamping_enabled: bool,

    // State
    integrator_enabled: bool,
    output_saturated: bool,
    saturation_error: Real,
}

impl PiController {
    /// Create a new PI controller with the given sampling time.
    pub fn new(sampling_time: Real) -> Self {
        Self {
            kp: 0.0,
            ki: 0.0,
            integrator: Integrator::new(sampling_time),
            output_min: Real::MIN,
            output_max: Real::MAX,
            has_output_limits: false,
            kc: 1.0,
            integrator_clamping_enabled: false,
            integrator_enabled: true,
            output_saturated: false,
            saturation_error: 0.0,
        }
    }

    /// Configure with proportional gain and continuous-time integral gain.
    pub fn configure(&mut self, kp: Real, ki_continuous: Real) {
        self.kp = kp;
        self.ki = ki_continuous;
    }

    /// Configure with proportional gain and integral time constant.
    ///
    /// `Ki = Kp / Ti`.
    pub fn configure_with_ti(&mut self, kp: Real, ti: Real) {
        debug_assert!(ti > 0.0, "Integral time constant must be positive");
        self.kp = kp;
        self.ki = if ti > 0.0 { kp / ti } else { 0.0 };
    }

    /// Set output clamping limits. Ignored if `min >= max`.
    pub fn set_output_limits(&mut self, min: Real, max: Real) {
        if min < max {
            self.output_min = min;
            self.output_max = max;
            self.has_output_limits = true;
        }
    }

    /// Enable or disable output limits.
    pub fn enable_output_limits(&mut self, enable: bool) {
        self.has_output_limits = enable;
    }

    /// Set anti-windup gain for back-calculation method.
    pub fn set_antiwindup_gain(&mut self, kc: Real) {
        debug_assert!(kc >= 0.0);
        self.kc = if kc >= 0.0 { kc } else { 0.0 };
    }

    /// Enable integrator clamping (freeze integrator when output is saturated).
    pub fn enable_integrator_clamping(&mut self, enable: bool) {
        self.integrator_clamping_enabled = enable;
    }

    /// Enable or disable the integrator.
    pub fn enable_integrator(&mut self, enable: bool) {
        self.integrator_enabled = enable;
    }

    /// Returns `true` if the output was saturated during the last [`update`](Self::update).
    pub fn is_output_saturated(&self) -> bool {
        self.output_saturated
    }

    /// Saturation error from the last update (`u_saturated − u_unsaturated`).
    pub fn saturation_error(&self) -> Real {
        self.saturation_error
    }

    /// Reset controller state to the given value.
    pub fn reset(&mut self, value: Real) {
        self.integrator.reset(value);
        self.output_saturated = false;
        self.saturation_error = 0.0;
    }

    /// Reset controller state to zero.
    pub fn reset_to_zero(&mut self) {
        self.reset(0.0);
    }

    /// Preset integrator for bumpless transfer.
    ///
    /// Sets the integrator so that, given `current_error`, the controller would
    /// produce `desired_output`:
    ///
    /// ```text
    /// integrator_preset = desired_output − Kp · current_error
    /// ```
    pub fn preset_for_bumpless_transfer(&mut self, desired_output: Real, current_error: Real) {
        let mut preset = desired_output - self.kp * current_error;
        if self.has_output_limits {
            preset = preset.clamp(self.output_min, self.output_max);
        }
        self.integrator.reset(preset);
        self.output_saturated = false;
        self.saturation_error = 0.0;
    }

    /// Run one control cycle.
    ///
    /// `input` is the error signal (setpoint − measurement). Returns the
    /// control output after clamping and anti-windup.
    pub fn update(&mut self, input: Real) -> Real {
        // Proportional term
        let proportional = self.kp * input;

        // Total output before saturation
        let unsaturated = proportional + self.integrator.output();

        // Apply output limits
        let mut saturated = unsaturated;
        let mut is_sat = false;
        if self.has_output_limits {
            saturated = unsaturated.clamp(self.output_min, self.output_max);
            is_sat = libm::fabsf(saturated - unsaturated) > 1e-9;
        }

        self.output_saturated = is_sat;
        self.saturation_error = saturated - unsaturated;

        // If integrator disabled, return early
        if !self.integrator_enabled {
            return saturated;
        }

        // Integrator clamping: freeze when saturated
        if self.integrator_clamping_enabled && self.output_saturated {
            return saturated;
        }

        // Back-calculation anti-windup:
        // integral_input = Ki·e + Kc·(u_sat − u_unsat)
        let integral_input = self.ki * input + self.kc * self.saturation_error;
        self.integrator.update(integral_input);

        saturated
    }

    // ── Getters ──────────────────────────────────────────────────────────

    /// Proportional gain.
    pub fn kp(&self) -> Real {
        self.kp
    }

    /// Continuous-time integral gain.
    pub fn ki(&self) -> Real {
        self.ki
    }

    /// Configured sampling time.
    pub fn sampling_time(&self) -> Real {
        self.integrator.sampling_time()
    }

    /// Current integrator value.
    pub fn integral(&self) -> Real {
        self.integrator.output()
    }

    /// Whether the integrator is enabled.
    pub fn is_integrator_enabled(&self) -> bool {
        self.integrator_enabled
    }

    /// Whether output limits are active.
    pub fn has_output_limits(&self) -> bool {
        self.has_output_limits
    }

    /// Current anti-windup gain.
    pub fn antiwindup_gain(&self) -> Real {
        self.kc
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.001; // 1 kHz
    const EPS: Real = 1e-3;

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < EPS
    }

    fn approx_eq_eps(a: Real, b: Real, eps: Real) -> bool {
        (a - b).abs() < eps
    }

    // 1. configure_with_ti: kp=2, ti=4 → ki = 0.5
    #[test]
    fn configure_with_ti() {
        let mut pi = PiController::new(TS);
        pi.configure_with_ti(2.0, 4.0);
        assert!(approx_eq(pi.kp(), 2.0));
        assert!(approx_eq(pi.ki(), 0.5));
    }

    // 2. output_limit_clamping: limits [-1,1], large error → clamped
    #[test]
    fn output_limit_clamping() {
        let mut pi = PiController::new(TS);
        pi.configure(10.0, 0.0);
        pi.set_output_limits(-1.0, 1.0);

        let out = pi.update(100.0);
        assert!(approx_eq(out, 1.0));
        assert!(pi.is_output_saturated());

        let out = pi.update(-100.0);
        assert!(approx_eq(out, -1.0));
        assert!(pi.is_output_saturated());
    }

    // 3. closed_loop_integration_plant: PI drives integrator plant to setpoint
    #[test]
    fn closed_loop_integration_plant() {
        let ts: Real = 0.001;
        let mut pi = PiController::new(ts);
        pi.configure_with_ti(2.0, 1.0); // kp=2, ki=kp/ti=2

        let setpoint: Real = 1.0;
        let mut plant_state: Real = 0.0;
        let sim_time: Real = 5.0;
        let steps = (sim_time / ts) as u32;

        for _ in 0..steps {
            let error = setpoint - plant_state;
            let control = pi.update(error);
            // Integrator plant: y[k+1] = y[k] + u * Ts
            plant_state += control * ts;
        }

        assert!(
            approx_eq_eps(plant_state, setpoint, 0.05),
            "Plant did not converge: got {plant_state}, expected {setpoint}"
        );
    }

    // 4. integrator_disable: integral doesn't change when disabled
    #[test]
    fn integrator_disable() {
        let mut pi = PiController::new(TS);
        pi.configure(1.0, 100.0);

        // Run a few steps with integrator enabled
        for _ in 0..10 {
            pi.update(1.0);
        }
        let integral_before = pi.integral();
        assert!(integral_before.abs() > EPS, "Integral should be non-zero");

        // Disable integrator
        pi.enable_integrator(false);
        for _ in 0..100 {
            pi.update(1.0);
        }
        let integral_after = pi.integral();

        assert!(
            approx_eq(integral_before, integral_after),
            "Integral changed after disabling: before={integral_before}, after={integral_after}"
        );
    }

    // 5. anti_windup_prevents_windup: back-calculation limits integral growth
    // Matches C++ test parameters: kp=2, ki=10, kc=ki/kp=5
    #[test]
    fn anti_windup_prevents_windup() {
        let ts: Real = 0.01;
        let kp: Real = 2.0;
        let ki: Real = 10.0;
        let mut pi = PiController::new(ts);
        pi.configure(kp, ki);
        pi.set_output_limits(-1.0, 1.0);

        // kc = 1/Ti = ki/kp (common choice)
        let kc = 1.0 / (kp / ki);
        pi.set_antiwindup_gain(kc);
        pi.reset_to_zero();

        let large_error: Real = 10.0;
        for _ in 0..10 {
            let out = pi.update(large_error);
            assert!(out <= 1.0 + EPS);
            assert!(out >= -1.0 - EPS);
        }
        assert!(pi.is_output_saturated());

        let integral_after_saturation = pi.integral();

        // Continue driving — integral should have reached steady state
        for _ in 0..10 {
            pi.update(large_error);
        }
        let integral_final = pi.integral();

        let growth = (integral_final - integral_after_saturation).abs();
        assert!(
            growth < 0.3,
            "Integral grew too much: {integral_after_saturation} → {integral_final} (growth={growth})"
        );
    }

    // 6. integrator_clamping_prevents_windup
    #[test]
    fn integrator_clamping_prevents_windup() {
        let mut pi = PiController::new(TS);
        pi.configure(1.0, 100.0);
        pi.set_output_limits(-1.0, 1.0);
        pi.enable_integrator_clamping(true);
        // Disable back-calculation so only clamping is active
        pi.set_antiwindup_gain(0.0);

        // Saturate with large positive error
        for _ in 0..1000 {
            pi.update(10.0);
        }
        let integral_clamped = pi.integral();

        // Continue — integral should be frozen
        for _ in 0..1000 {
            pi.update(10.0);
        }
        let integral_still_clamped = pi.integral();
        assert!(
            approx_eq(integral_clamped, integral_still_clamped),
            "Integral grew despite clamping: {integral_clamped} → {integral_still_clamped}"
        );

        // Now remove saturation (error=0, proportional=0, integral alone is
        // within limits) → integrator should resume.
        // First, reset to a value within limits so output is not saturated.
        pi.reset(0.5);
        pi.update(0.0); // not saturated
        assert!(!pi.is_output_saturated());

        // Apply small error — integral should change
        let integral_before = pi.integral();
        for _ in 0..100 {
            pi.update(0.01);
        }
        let integral_after = pi.integral();
        assert!(
            (integral_after - integral_before).abs() > 1e-6,
            "Integral did not resume after clamping was released"
        );
    }

    // 7. bumpless_transfer: preset so first output ≈ desired
    #[test]
    fn bumpless_transfer() {
        let mut pi = PiController::new(TS);
        pi.configure(1.0, 10.0);

        let desired_output: Real = 0.5;
        let current_error: Real = 0.1;
        pi.preset_for_bumpless_transfer(desired_output, current_error);

        // First update with same error should yield ≈ desired_output
        // output = kp*error + integral = 1.0*0.1 + (0.5 - 1.0*0.1) = 0.5
        let out = pi.update(current_error);
        assert!(
            approx_eq_eps(out, desired_output, 0.05),
            "Bumpless transfer failed: got {out}, expected ~{desired_output}"
        );
    }

    // 8. proportional_only: ki=0 → output = kp * input
    #[test]
    fn proportional_only() {
        let mut pi = PiController::new(TS);
        pi.configure(3.5, 0.0);

        let inputs: [Real; 4] = [1.0, -2.0, 0.0, 0.5];
        for &inp in &inputs {
            let out = pi.update(inp);
            assert!(
                approx_eq(out, 3.5 * inp),
                "P-only mismatch: input={inp}, got={out}, expected={}",
                3.5 * inp
            );
        }
    }

    // 9. zero_error_no_change: constant zero error → output stays at initial
    #[test]
    fn zero_error_no_change() {
        let mut pi = PiController::new(TS);
        pi.configure(5.0, 100.0);

        for _ in 0..1000 {
            let out = pi.update(0.0);
            assert!(approx_eq(out, 0.0), "Non-zero output for zero error: {out}");
        }
    }

    // 10. reset_clears_state
    #[test]
    fn reset_clears_state() {
        let mut pi = PiController::new(TS);
        pi.configure(1.0, 100.0);

        // Accumulate some integral
        for _ in 0..100 {
            pi.update(1.0);
        }
        assert!(pi.integral().abs() > EPS);

        pi.reset_to_zero();
        assert!(
            approx_eq(pi.integral(), 0.0),
            "Integral not zero after reset: {}",
            pi.integral()
        );
        assert!(!pi.is_output_saturated());
        assert!(approx_eq(pi.saturation_error(), 0.0));
    }
}
