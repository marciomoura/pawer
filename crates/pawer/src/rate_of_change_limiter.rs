/// Slew-rate limiter for analog signals.
///
/// When enabled, the output ramps toward the input at a maximum rate of
/// `rate_limit_per_second` units/s (i.e., `max_change_per_sample` per control
/// cycle). When disabled the input passes through unchanged.
use crate::types::Real;

pub struct RateOfChangeLimiter {
    sampling_time: Real,
    max_change_per_sample: Real,
    current_output: Real,
    enabled: bool,
}

impl RateOfChangeLimiter {
    /// Creates a disabled limiter with the given sampling time and a very large
    /// default rate.
    ///
    /// # Panics (debug only)
    /// Panics if `sampling_time` is not positive.
    pub fn new(sampling_time: Real) -> Self {
        debug_assert!(sampling_time > 0.0);
        Self {
            sampling_time,
            max_change_per_sample: 1e6,
            current_output: 0.0,
            enabled: false,
        }
    }

    /// Creates a fully configured and **enabled** limiter.
    ///
    /// # Panics (debug only)
    /// Panics if `sampling_time` or `rate_limit_per_second` is not positive.
    pub fn with_config(
        rate_limit_per_second: Real,
        sampling_time: Real,
        initial_value: Real,
    ) -> Self {
        debug_assert!(sampling_time > 0.0);
        debug_assert!(rate_limit_per_second > 0.0);
        Self {
            sampling_time,
            max_change_per_sample: rate_limit_per_second * sampling_time,
            current_output: initial_value,
            enabled: true,
        }
    }

    /// Configures the maximum rate of change in units per second.
    ///
    /// # Panics (debug only)
    /// Panics if `rate_limit_per_second` is not positive.
    pub fn configure(&mut self, rate_limit_per_second: Real) {
        debug_assert!(rate_limit_per_second > 0.0);
        self.max_change_per_sample = rate_limit_per_second * self.sampling_time;
    }

    /// Enables or disables the rate limiter.
    pub fn configure_enable(&mut self, enable: bool) {
        self.enabled = enable;
    }

    /// Returns `true` if the limiter is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Resets the output to the given `value` immediately (no ramping).
    pub fn reset(&mut self, value: Real) {
        self.current_output = value;
    }

    /// Advances the limiter by one sampling period.
    pub fn update(&mut self, input: Real) {
        if !self.enabled {
            self.current_output = input;
            return;
        }
        let error = input - self.current_output;
        let change = (error).clamp(-self.max_change_per_sample, self.max_change_per_sample);
        self.current_output += change;
    }

    /// Returns the current (rate-limited) output.
    pub fn output(&self) -> Real {
        self.current_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.001;
    const EPSILON: Real = 1e-5;

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn disabled_passes_through() {
        let mut lim = RateOfChangeLimiter::new(TS);
        assert!(!lim.is_enabled());

        lim.update(42.0);
        assert!(approx_eq(lim.output(), 42.0));

        lim.update(-10.0);
        assert!(approx_eq(lim.output(), -10.0));
    }

    #[test]
    fn step_ramps_at_limited_rate() {
        // 1000 units/s * 0.001 s = 1.0 per sample.
        let mut lim = RateOfChangeLimiter::with_config(1000.0, TS, 0.0);

        lim.update(10.0);
        assert!(
            approx_eq(lim.output(), 1.0),
            "expected 1.0, got {}",
            lim.output()
        );

        lim.update(10.0);
        assert!(
            approx_eq(lim.output(), 2.0),
            "expected 2.0, got {}",
            lim.output()
        );
    }

    #[test]
    fn step_from_zero_to_ten_takes_ten_samples() {
        let mut lim = RateOfChangeLimiter::with_config(1000.0, TS, 0.0);

        for i in 1..=10 {
            lim.update(10.0);
            assert!(
                approx_eq(lim.output(), i as Real),
                "step {i}: expected {}, got {}",
                i as Real,
                lim.output()
            );
        }

        // After reaching target, stays at target.
        lim.update(10.0);
        assert!(approx_eq(lim.output(), 10.0));
    }

    #[test]
    fn negative_step_ramps_down() {
        let mut lim = RateOfChangeLimiter::with_config(1000.0, TS, 10.0);

        for i in 1..=10 {
            lim.update(0.0);
            let expected = 10.0 - i as Real;
            assert!(
                approx_eq(lim.output(), expected),
                "step {i}: expected {expected}, got {}",
                lim.output()
            );
        }
    }

    #[test]
    fn reset_sets_output_immediately() {
        let mut lim = RateOfChangeLimiter::with_config(1000.0, TS, 0.0);
        lim.update(10.0);
        assert!(approx_eq(lim.output(), 1.0));

        lim.reset(5.0);
        assert!(approx_eq(lim.output(), 5.0));
    }

    #[test]
    fn enable_disable_toggle() {
        let mut lim = RateOfChangeLimiter::new(TS);
        lim.configure(1000.0);
        lim.configure_enable(true);
        assert!(lim.is_enabled());

        lim.update(10.0);
        assert!(approx_eq(lim.output(), 1.0));

        lim.configure_enable(false);
        lim.update(10.0);
        assert!(approx_eq(lim.output(), 10.0));
    }

    #[test]
    fn with_config_is_enabled() {
        let lim = RateOfChangeLimiter::with_config(500.0, TS, 0.0);
        assert!(lim.is_enabled());
        assert!(approx_eq(lim.output(), 0.0));
    }

    #[test]
    fn small_step_within_limit_passes_through() {
        // Max 1.0 per sample, but input only changes by 0.5.
        let mut lim = RateOfChangeLimiter::with_config(1000.0, TS, 0.0);
        lim.update(0.5);
        assert!(
            approx_eq(lim.output(), 0.5),
            "small step should pass through, got {}",
            lim.output()
        );
    }

    #[test]
    fn symmetric_rate_limit() {
        let mut lim = RateOfChangeLimiter::with_config(1000.0, TS, 5.0);

        // Step up.
        lim.update(15.0);
        assert!(approx_eq(lim.output(), 6.0));

        // Step down.
        lim.reset(5.0);
        lim.update(-5.0);
        assert!(approx_eq(lim.output(), 4.0));
    }

    #[test]
    fn configure_changes_rate() {
        let mut lim = RateOfChangeLimiter::with_config(1000.0, TS, 0.0);

        // Rate = 1.0/sample.
        lim.update(10.0);
        assert!(approx_eq(lim.output(), 1.0));

        // Double the rate → 2.0/sample.
        lim.configure(2000.0);
        lim.update(10.0);
        assert!(
            approx_eq(lim.output(), 3.0),
            "expected 3.0 after rate change, got {}",
            lim.output()
        );
    }
}
