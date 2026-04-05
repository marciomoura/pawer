use crate::types::Real;

/// On-Delay Timer.
///
/// Output becomes true only after input has been continuously true for
/// at least `delay_s` seconds. If input goes false at any point the
/// internal counter resets and the output is immediately false.
pub struct OnDelay {
    sampling_time: Real,
    samples_required: i32,
    sample_counter: i32,
    output: bool,
}

impl OnDelay {
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
            if self.sample_counter < self.samples_required {
                self.sample_counter += 1;
            }
            self.output = self.sample_counter >= self.samples_required;
        } else {
            self.sample_counter = 0;
            self.output = false;
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
        let mut timer = OnDelay::new(0.010, TS); // 10 ms delay → 10 samples
        for _ in 0..100 {
            timer.update(false);
            assert!(!timer.output());
        }
    }

    #[test]
    fn input_true_less_than_delay_output_stays_false() {
        let mut timer = OnDelay::new(0.010, TS); // 10 samples
        for _ in 0..9 {
            timer.update(true);
        }
        assert!(!timer.output());
    }

    #[test]
    fn input_true_exactly_delay_output_becomes_true() {
        let mut timer = OnDelay::new(0.010, TS); // 10 samples
        for i in 0..10 {
            timer.update(true);
            if i < 9 {
                assert!(!timer.output(), "should still be false at sample {i}");
            }
        }
        assert!(timer.output());
    }

    #[test]
    fn output_stays_true_while_input_remains_true() {
        let mut timer = OnDelay::new(0.005, TS); // 5 samples
        for _ in 0..5 {
            timer.update(true);
        }
        assert!(timer.output());
        for _ in 0..20 {
            timer.update(true);
            assert!(timer.output());
        }
    }

    #[test]
    fn input_false_resets_counter_and_output() {
        let mut timer = OnDelay::new(0.005, TS); // 5 samples
        // Reach output true
        for _ in 0..5 {
            timer.update(true);
        }
        assert!(timer.output());
        // Input goes false
        timer.update(false);
        assert!(!timer.output());
        // Need full delay again
        for _ in 0..4 {
            timer.update(true);
            assert!(!timer.output());
        }
        timer.update(true);
        assert!(timer.output());
    }

    #[test]
    fn interrupted_input_resets_progress() {
        let mut timer = OnDelay::new(0.010, TS); // 10 samples
        // 8 samples of true
        for _ in 0..8 {
            timer.update(true);
        }
        assert!(!timer.output());
        // Glitch false
        timer.update(false);
        assert!(!timer.output());
        // Must start counting from zero again
        for _ in 0..9 {
            timer.update(true);
            assert!(!timer.output());
        }
        timer.update(true);
        assert!(timer.output());
    }

    #[test]
    fn reconfigure_with_new_delay() {
        let mut timer = OnDelay::new(0.010, TS); // 10 samples
        // Reach output true
        for _ in 0..10 {
            timer.update(true);
        }
        assert!(timer.output());
        // Reconfigure to 3 samples
        timer.configure(0.003);
        assert!(!timer.output()); // configure calls reset
        for _ in 0..2 {
            timer.update(true);
            assert!(!timer.output());
        }
        timer.update(true);
        assert!(timer.output());
    }

    #[test]
    fn zero_delay_output_immediately_true() {
        let mut timer = OnDelay::new(0.0, TS); // 0 samples required
        timer.update(true);
        assert!(timer.output());
    }

    #[test]
    fn zero_delay_output_false_when_input_false() {
        let mut timer = OnDelay::new(0.0, TS);
        timer.update(true);
        assert!(timer.output());
        timer.update(false);
        assert!(!timer.output());
    }

    #[test]
    fn reset_clears_state() {
        let mut timer = OnDelay::new(0.005, TS);
        for _ in 0..5 {
            timer.update(true);
        }
        assert!(timer.output());
        timer.reset();
        assert!(!timer.output());
        // Must count full delay again
        for _ in 0..4 {
            timer.update(true);
            assert!(!timer.output());
        }
        timer.update(true);
        assert!(timer.output());
    }
}
