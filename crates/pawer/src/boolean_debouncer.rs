/// Integrator-based boolean debouncer with configurable on/off delays.
///
/// Uses a saturating integrator in the range \[0.0, 1.0\]. When the input is
/// `true` the integrator increments by `sampling_time / on_delay`; when `false`
/// it decrements by `sampling_time / off_delay`. The output transitions to
/// `true` only when the integrator reaches 1.0, and to `false` only when it
/// reaches 0.0, providing hysteresis against noisy boolean signals.
use crate::types::Real;

pub struct BooleanDebouncer {
    sampling_time: Real,
    on_delay_s: Real,
    off_delay_s: Real,
    on_increment: Real,
    off_decrement: Real,
    integrator: Real,
    output: bool,
}

impl BooleanDebouncer {
    /// Creates a new [`BooleanDebouncer`] with all parameters zeroed.
    ///
    /// Call [`configure_sampling_time`](Self::configure_sampling_time) and
    /// [`configure_delay`](Self::configure_delay) before use.
    pub fn new() -> Self {
        Self {
            sampling_time: 0.0,
            on_delay_s: 0.0,
            off_delay_s: 0.0,
            on_increment: 0.0,
            off_decrement: 0.0,
            integrator: 0.0,
            output: false,
        }
    }

    /// Sets the control-loop sampling time and recalculates internal
    /// increments.
    pub fn configure_sampling_time(&mut self, sampling_time: Real) {
        self.sampling_time = sampling_time;
        self.update_increments();
    }

    /// Sets the on-delay and off-delay in seconds and recalculates internal
    /// increments.
    pub fn configure_delay(&mut self, on_delay_s: Real, off_delay_s: Real) {
        self.on_delay_s = on_delay_s;
        self.off_delay_s = off_delay_s;
        self.update_increments();
    }

    fn update_increments(&mut self) {
        self.on_increment = if self.on_delay_s > 0.0 {
            self.sampling_time / self.on_delay_s
        } else {
            1.1
        };
        self.off_decrement = if self.off_delay_s > 0.0 {
            self.sampling_time / self.off_delay_s
        } else {
            1.1
        };
    }

    /// Advances the debouncer by one sampling period.
    pub fn update(&mut self, input: bool) {
        if input {
            self.integrator += self.on_increment;
        } else {
            self.integrator -= self.off_decrement;
        }

        // Saturate to [0, 1].
        if self.integrator > 1.0 {
            self.integrator = 1.0;
        }
        if self.integrator < 0.0 {
            self.integrator = 0.0;
        }

        // Hysteresis: output changes only at the saturation boundaries.
        if self.integrator >= 1.0 {
            self.output = true;
        } else if self.integrator <= 0.0 {
            self.output = false;
        }
    }

    /// Resets the debouncer to its initial state (output `false`, integrator
    /// 0).
    pub fn reset(&mut self) {
        self.integrator = 0.0;
        self.output = false;
    }

    /// Resets the debouncer to a specific initial state.
    pub fn reset_to(&mut self, initial_state: bool) {
        self.output = initial_state;
        self.integrator = if initial_state { 1.0 } else { 0.0 };
    }

    /// Returns the current debounced output.
    pub fn output(&self) -> bool {
        self.output
    }

    /// Returns the current value of the internal integrator \[0.0, 1.0\].
    pub fn integrator(&self) -> Real {
        self.integrator
    }
}

impl Default for BooleanDebouncer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_debouncer() -> BooleanDebouncer {
        let mut d = BooleanDebouncer::new();
        d.configure_sampling_time(0.001);
        d.configure_delay(0.01, 0.01);
        d
    }

    #[test]
    fn ten_true_updates_turns_output_on() {
        let mut d = make_debouncer();
        for _ in 0..9 {
            d.update(true);
            assert!(!d.output(), "output should not be true before 10 steps");
        }
        d.update(true);
        assert!(d.output());
    }

    #[test]
    fn ten_false_updates_turns_output_off() {
        let mut d = make_debouncer();
        // First turn on.
        for _ in 0..10 {
            d.update(true);
        }
        assert!(d.output());

        // Now turn off.
        for _ in 0..9 {
            d.update(false);
            assert!(d.output(), "output should stay true before 10 false steps");
        }
        d.update(false);
        assert!(!d.output());
    }

    #[test]
    fn alternating_input_stays_false() {
        let mut d = make_debouncer();
        for i in 0..100 {
            d.update(i % 2 == 0);
        }
        // Alternating true/false keeps integrator near the middle — never
        // reaches 1.0.
        assert!(!d.output());
    }

    #[test]
    fn zero_delay_gives_immediate_transition() {
        let mut d = BooleanDebouncer::new();
        d.configure_sampling_time(0.001);
        d.configure_delay(0.0, 0.0);

        d.update(true);
        assert!(d.output(), "zero on_delay should give immediate on");

        d.update(false);
        assert!(!d.output(), "zero off_delay should give immediate off");
    }

    #[test]
    fn reset_clears_state() {
        let mut d = make_debouncer();
        for _ in 0..10 {
            d.update(true);
        }
        assert!(d.output());

        d.reset();
        assert!(!d.output());
        assert_eq!(d.integrator(), 0.0);
    }

    #[test]
    fn reset_to_true() {
        let mut d = make_debouncer();
        d.reset_to(true);
        assert!(d.output());
        assert_eq!(d.integrator(), 1.0);
    }

    #[test]
    fn reset_to_false() {
        let mut d = make_debouncer();
        d.reset_to(true);
        d.reset_to(false);
        assert!(!d.output());
        assert_eq!(d.integrator(), 0.0);
    }

    #[test]
    fn integrator_stays_in_bounds() {
        let mut d = make_debouncer();
        // Many true inputs — integrator must not exceed 1.0.
        for _ in 0..100 {
            d.update(true);
        }
        assert_eq!(d.integrator(), 1.0);

        // Many false inputs — integrator must not go below 0.0.
        for _ in 0..100 {
            d.update(false);
        }
        assert_eq!(d.integrator(), 0.0);
    }

    #[test]
    fn hysteresis_holds_output_in_band() {
        let mut d = make_debouncer();
        // Charge to about 50%.
        for _ in 0..5 {
            d.update(true);
        }
        assert!(!d.output(), "output should remain false at 50%");

        // Now discharge back to 0%.
        for _ in 0..5 {
            d.update(false);
        }
        assert!(!d.output(), "output should still be false");
    }

    #[test]
    fn default_matches_new() {
        let d = BooleanDebouncer::default();
        assert!(!d.output());
        assert_eq!(d.integrator(), 0.0);
    }
}
