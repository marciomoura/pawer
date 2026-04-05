use crate::types::Real;

/// Off-Delay Timer.
///
/// Output immediately becomes true when input goes true. When input goes
/// false the timer counts, and output becomes false only after input has
/// been continuously false for `delay_s` seconds.
pub struct OffDelay {
    sampling_time: Real,
    samples_required: i32,
    sample_counter: i32,
    output: bool,
}

impl OffDelay {
    pub fn new(delay_s: Real, sampling_time: Real) -> Self {
        debug_assert!(sampling_time > 0.0);
        let samples_required = libm::roundf(delay_s / sampling_time) as i32;
        Self {
            sampling_time,
            samples_required,
            sample_counter: 0,
            output: false,
        }
    }

    pub fn configure(&mut self, delay_s: Real) {
        debug_assert!(delay_s >= 0.0);
        self.samples_required = libm::roundf(delay_s / self.sampling_time) as i32;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.sample_counter = 0;
        self.output = false;
    }

    pub fn update(&mut self, input: bool) {
        if input {
            self.sample_counter = 0;
            self.output = true;
        } else if self.output {
            self.sample_counter += 1;
            if self.sample_counter >= self.samples_required {
                self.output = false;
            }
        } else {
            self.sample_counter = 0;
        }
    }

    pub fn output(&self) -> bool {
        self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.001; // 1 ms sampling time

    #[test]
    fn input_always_false_output_stays_false() {
        let mut timer = OffDelay::new(0.010, TS); // 10 samples
        for _ in 0..100 {
            timer.update(false);
            assert!(!timer.output());
        }
    }

    #[test]
    fn input_true_output_immediately_true() {
        let mut timer = OffDelay::new(0.010, TS);
        timer.update(true);
        assert!(timer.output());
    }

    #[test]
    fn output_stays_true_during_off_delay() {
        let mut timer = OffDelay::new(0.005, TS); // 5 samples
        timer.update(true);
        assert!(timer.output());
        // Input goes false — output stays true for 4 more samples
        for i in 0..4 {
            timer.update(false);
            assert!(timer.output(), "should still be true at sample {i}");
        }
        // 5th false sample → output goes false
        timer.update(false);
        assert!(!timer.output());
    }

    #[test]
    fn input_true_during_off_delay_resets_counter() {
        let mut timer = OffDelay::new(0.005, TS); // 5 samples
        timer.update(true);
        // 3 false samples (counter at 3, still in delay)
        for _ in 0..3 {
            timer.update(false);
        }
        assert!(timer.output());
        // Input true again — resets counter
        timer.update(true);
        assert!(timer.output());
        // Now need full 5 false samples again
        for _ in 0..4 {
            timer.update(false);
            assert!(timer.output());
        }
        timer.update(false);
        assert!(!timer.output());
    }

    #[test]
    fn zero_delay_output_false_immediately_when_input_false() {
        let mut timer = OffDelay::new(0.0, TS); // 0 samples required
        timer.update(true);
        assert!(timer.output());
        timer.update(false);
        assert!(!timer.output());
    }

    #[test]
    fn output_stays_true_while_input_remains_true() {
        let mut timer = OffDelay::new(0.010, TS);
        for _ in 0..50 {
            timer.update(true);
            assert!(timer.output());
        }
    }

    #[test]
    fn reconfigure_with_new_delay() {
        let mut timer = OffDelay::new(0.010, TS); // 10 samples
        timer.update(true);
        assert!(timer.output());
        // Reconfigure to 3 samples
        timer.configure(0.003);
        assert!(!timer.output()); // configure calls reset
        timer.update(true);
        assert!(timer.output());
        for _ in 0..2 {
            timer.update(false);
            assert!(timer.output());
        }
        timer.update(false);
        assert!(!timer.output());
    }

    #[test]
    fn reset_clears_state() {
        let mut timer = OffDelay::new(0.005, TS);
        timer.update(true);
        assert!(timer.output());
        timer.reset();
        assert!(!timer.output());
        // False input should not trigger delay since output is already false
        timer.update(false);
        assert!(!timer.output());
    }

    #[test]
    fn output_stays_false_after_delay_expires() {
        let mut timer = OffDelay::new(0.003, TS); // 3 samples
        timer.update(true);
        // Expire the off-delay
        for _ in 0..3 {
            timer.update(false);
        }
        assert!(!timer.output());
        // Further false inputs keep it false
        for _ in 0..10 {
            timer.update(false);
            assert!(!timer.output());
        }
    }

    #[test]
    fn multiple_on_off_cycles() {
        let mut timer = OffDelay::new(0.003, TS); // 3 samples
        for _ in 0..3 {
            timer.update(true);
            assert!(timer.output());
            for _ in 0..2 {
                timer.update(false);
                assert!(timer.output());
            }
            timer.update(false);
            assert!(!timer.output());
        }
    }
}
