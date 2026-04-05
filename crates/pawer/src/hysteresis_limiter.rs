/// Schmitt-trigger (hysteresis) comparator for analog signals.
///
/// The output transitions to `true` when the input rises **above**
/// `high_threshold` and remains `true` until the input falls **below**
/// `low_threshold`. This dead-band prevents chattering when the signal hovers
/// near a single threshold.
use crate::types::Real;

#[derive(Default)]
pub struct HysteresisLimiter {
    low_threshold: Real,
    high_threshold: Real,
    output: bool,
}

impl HysteresisLimiter {
    /// Creates a new [`HysteresisLimiter`] with both thresholds at 0.0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the low and high thresholds.
    ///
    /// # Panics (debug only)
    /// Panics if `low_threshold > high_threshold`.
    pub fn configure_thresholds(&mut self, low_threshold: Real, high_threshold: Real) {
        debug_assert!(low_threshold <= high_threshold);
        self.low_threshold = low_threshold;
        self.high_threshold = high_threshold;
    }

    /// Evaluates the hysteresis comparator for the given `input`.
    pub fn update(&mut self, input: Real) {
        if self.output {
            if input < self.low_threshold {
                self.output = false;
            }
        } else if input > self.high_threshold {
            self.output = true;
        }
    }

    /// Resets the output to `false`.
    pub fn reset(&mut self) {
        self.output = false;
    }

    /// Returns the current comparator output.
    pub fn output(&self) -> bool {
        self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_limiter() -> HysteresisLimiter {
        let mut h = HysteresisLimiter::new();
        h.configure_thresholds(3.0, 7.0);
        h
    }

    #[test]
    fn rising_input_triggers_above_high_threshold() {
        let mut h = make_limiter();
        for v in 0..=7 {
            h.update(v as Real);
            assert!(!h.output(), "should be false at input {v}");
        }
        // Strictly above 7.0.
        h.update(7.5);
        assert!(h.output());
    }

    #[test]
    fn falling_input_clears_below_low_threshold() {
        let mut h = make_limiter();
        // Activate.
        h.update(8.0);
        assert!(h.output());

        // Fall toward low threshold — output stays true.
        for v in [7.0, 6.0, 5.0, 4.0, 3.0] {
            h.update(v);
            assert!(h.output(), "should stay true at input {v}");
        }

        // Below low threshold.
        h.update(2.9);
        assert!(!h.output());
    }

    #[test]
    fn oscillation_within_band_no_change_when_off() {
        let mut h = make_limiter();
        // Stay within the hysteresis band without ever exceeding high.
        for _ in 0..50 {
            h.update(4.0);
            assert!(!h.output());
            h.update(6.0);
            assert!(!h.output());
        }
    }

    #[test]
    fn oscillation_within_band_no_change_when_on() {
        let mut h = make_limiter();
        h.update(8.0);
        assert!(h.output());

        // Oscillate within band — output stays true.
        for _ in 0..50 {
            h.update(4.0);
            assert!(h.output());
            h.update(6.0);
            assert!(h.output());
        }
    }

    #[test]
    fn crossing_high_then_staying_in_band() {
        let mut h = make_limiter();
        h.update(8.0);
        assert!(h.output());

        // Stay between thresholds.
        h.update(5.0);
        assert!(h.output());
        h.update(5.0);
        assert!(h.output());
    }

    #[test]
    fn reset_returns_to_false() {
        let mut h = make_limiter();
        h.update(8.0);
        assert!(h.output());

        h.reset();
        assert!(!h.output());
    }

    #[test]
    fn full_cycle() {
        let mut h = make_limiter();

        // Rise.
        h.update(8.0);
        assert!(h.output());

        // Fall below low.
        h.update(2.0);
        assert!(!h.output());

        // Rise again.
        h.update(8.0);
        assert!(h.output());
    }

    #[test]
    fn exactly_at_thresholds_no_transition() {
        let mut h = make_limiter();

        // Exactly at high_threshold — not strictly above, so stays false.
        h.update(7.0);
        assert!(!h.output());

        // Force on.
        h.update(8.0);
        assert!(h.output());

        // Exactly at low_threshold — not strictly below, so stays true.
        h.update(3.0);
        assert!(h.output());
    }

    #[test]
    fn default_matches_new() {
        let h = HysteresisLimiter::default();
        assert!(!h.output());
    }
}
