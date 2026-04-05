use crate::off_delay::OffDelay;
use crate::on_delay::OnDelay;
use crate::types::Real;

/// Combined On-Off Delay Timer.
///
/// Chains an [`OnDelay`] followed by an [`OffDelay`]. The on-delay gates
/// false→true transitions and the off-delay gates true→false transitions,
/// providing debouncing in both directions.
pub struct OnOffDelay {
    on_timer: OnDelay,
    off_timer: OffDelay,
}

impl OnOffDelay {
    pub fn new(sampling_time: Real) -> Self {
        Self {
            on_timer: OnDelay::new(0.0, sampling_time),
            off_timer: OffDelay::new(0.0, sampling_time),
        }
    }

    pub fn configure(&mut self, on_delay_s: Real, off_delay_s: Real) {
        self.on_timer.configure(on_delay_s);
        self.off_timer.configure(off_delay_s);
    }

    pub fn reset(&mut self) {
        self.on_timer.reset();
        self.off_timer.reset();
    }

    pub fn update(&mut self, input: bool) {
        self.on_timer.update(input);
        self.off_timer.update(self.on_timer.output());
    }

    pub fn output(&self) -> bool {
        self.off_timer.output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.001;

    #[test]
    fn on_delay_gates_rising_edge() {
        let mut timer = OnOffDelay::new(TS);
        timer.configure(0.005, 0.003); // on: 5 samples, off: 3 samples

        // Input true for 4 samples — not enough for on-delay
        for _ in 0..4 {
            timer.update(true);
            assert!(!timer.output());
        }
        // 5th sample — on-delay satisfied, off-delay input goes true → output true
        timer.update(true);
        assert!(timer.output());
    }

    #[test]
    fn off_delay_gates_falling_edge() {
        let mut timer = OnOffDelay::new(TS);
        timer.configure(0.005, 0.003); // on: 5, off: 3

        // Activate
        for _ in 0..5 {
            timer.update(true);
        }
        assert!(timer.output());

        // Input goes false — on-delay resets immediately, off-delay starts counting
        for i in 0..2 {
            timer.update(false);
            assert!(
                timer.output(),
                "should still be true at off-delay sample {i}"
            );
        }
        timer.update(false);
        assert!(!timer.output());
    }

    #[test]
    fn quick_toggle_does_not_pass_through() {
        let mut timer = OnOffDelay::new(TS);
        timer.configure(0.005, 0.003); // on: 5, off: 3

        // Short pulse — 3 samples true, then false
        for _ in 0..3 {
            timer.update(true);
        }
        timer.update(false);
        // On-delay was never satisfied, so output stays false
        assert!(!timer.output());
    }

    #[test]
    fn input_always_false_output_stays_false() {
        let mut timer = OnOffDelay::new(TS);
        timer.configure(0.005, 0.003);
        for _ in 0..50 {
            timer.update(false);
            assert!(!timer.output());
        }
    }

    #[test]
    fn full_on_off_cycle() {
        let mut timer = OnOffDelay::new(TS);
        timer.configure(0.005, 0.003); // on: 5, off: 3

        // Phase 1: off
        for _ in 0..10 {
            timer.update(false);
            assert!(!timer.output());
        }

        // Phase 2: ramp up — 5 true samples needed
        for _ in 0..4 {
            timer.update(true);
            assert!(!timer.output());
        }
        timer.update(true);
        assert!(timer.output());

        // Phase 3: stay on
        for _ in 0..10 {
            timer.update(true);
            assert!(timer.output());
        }

        // Phase 4: ramp down — 3 false samples needed
        for _ in 0..2 {
            timer.update(false);
            assert!(timer.output());
        }
        timer.update(false);
        assert!(!timer.output());

        // Phase 5: stay off
        for _ in 0..10 {
            timer.update(false);
            assert!(!timer.output());
        }
    }

    #[test]
    fn reset_clears_both_timers() {
        let mut timer = OnOffDelay::new(TS);
        timer.configure(0.005, 0.003);

        // Activate
        for _ in 0..5 {
            timer.update(true);
        }
        assert!(timer.output());

        timer.reset();
        assert!(!timer.output());

        // Must satisfy on-delay from scratch
        for _ in 0..4 {
            timer.update(true);
            assert!(!timer.output());
        }
        timer.update(true);
        assert!(timer.output());
    }

    #[test]
    fn zero_delays_pass_through_immediately() {
        let mut timer = OnOffDelay::new(TS);
        timer.configure(0.0, 0.0);

        timer.update(true);
        assert!(timer.output());
        timer.update(false);
        assert!(!timer.output());
    }

    #[test]
    fn interrupted_on_delay_resets() {
        let mut timer = OnOffDelay::new(TS);
        timer.configure(0.005, 0.003); // on: 5, off: 3

        // 4 true samples
        for _ in 0..4 {
            timer.update(true);
        }
        assert!(!timer.output());

        // Glitch false — resets on-delay
        timer.update(false);
        assert!(!timer.output());

        // Must count full 5 again
        for _ in 0..4 {
            timer.update(true);
            assert!(!timer.output());
        }
        timer.update(true);
        assert!(timer.output());
    }

    #[test]
    fn retriggered_during_off_delay() {
        let mut timer = OnOffDelay::new(TS);
        timer.configure(0.005, 0.003);

        // Activate
        for _ in 0..5 {
            timer.update(true);
        }
        assert!(timer.output());

        // Start off-delay (2 false samples)
        for _ in 0..2 {
            timer.update(false);
        }
        assert!(timer.output());

        // Input true again — on-delay must count from 0
        // The on-delay was reset when input went false, so we need 5 true samples
        // Meanwhile off-delay continues running because on-delay output is false
        for _ in 0..4 {
            timer.update(true);
        }
        // On-delay not yet satisfied (only 4 samples), off-timer input still false
        // from the on-delay perspective: on-delay just got 4 true samples
        // Actually let's trace carefully:
        // After 2 false: on_timer counter=0, on_timer output=false; off_timer counting
        // true sample 1: on_timer counter=1, output=false; off_timer sees false → still counting
        // true sample 2: on_timer counter=2, output=false; off_timer sees false → counting
        // true sample 3: on_timer counter=3, output=false; off_timer sees false → off expires (was at 2+3=5 > 3)
        // Wait — the off_timer counter started when on_timer output went false.
        // After activation, on_timer output = true. When input goes false:
        //   false 1: on_timer counter=0, output=false. off_timer sees false, was true → counter=1
        //   false 2: on_timer counter=0, output=false. off_timer sees false → counter=2
        // Now true again:
        //   true 1: on_timer counter=1, output=false. off_timer sees false → counter=3 → output=false
        // So after this point off_timer output is false! Let's adjust the test.

        // Let's just verify the output after the retriggered sequence
        // At this point (4 true after 2 false), the off-delay has expired
        // The on-delay has 4 samples counted. One more needed.
        timer.update(true); // 5th true → on-delay fires → off-delay sees true → output true
        assert!(timer.output());
    }
}
