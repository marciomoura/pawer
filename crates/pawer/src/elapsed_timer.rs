use crate::types::Real;

/// Elapsed Time Measurement.
///
/// Tracks elapsed time in seconds when enabled. Each call to [`update`]
/// with `enable = true` advances the internal counter by one sampling period.
pub struct ElapsedTimer {
    sampling_time: Real,
    elapsed_time: Real,
    enabled: bool,
}

impl ElapsedTimer {
    pub fn new(sampling_time: Real) -> Self {
        debug_assert!(sampling_time > 0.0);
        Self {
            sampling_time,
            elapsed_time: 0.0,
            enabled: false,
        }
    }

    pub fn configure_sampling_time(&mut self, sampling_time: Real) {
        debug_assert!(sampling_time > 0.0);
        self.sampling_time = sampling_time;
    }

    pub fn update(&mut self, enable: bool) {
        self.enabled = enable;
        if self.enabled {
            self.elapsed_time += self.sampling_time;
        }
    }

    pub fn reset(&mut self) {
        self.elapsed_time = 0.0;
    }

    pub fn reset_to(&mut self, initial_time: Real) {
        self.elapsed_time = initial_time;
    }

    pub fn elapsed_time(&self) -> Real {
        self.elapsed_time
    }

    pub fn has_elapsed(&self, threshold: Real) -> bool {
        self.elapsed_time >= threshold
    }

    pub fn remaining_time(&self, threshold: Real) -> Real {
        if self.elapsed_time >= threshold {
            0.0
        } else {
            threshold - self.elapsed_time
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.001;
    const EPSILON: Real = 1e-6;

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn disabled_time_does_not_advance() {
        let mut timer = ElapsedTimer::new(TS);
        for _ in 0..100 {
            timer.update(false);
        }
        assert!(approx_eq(timer.elapsed_time(), 0.0));
        assert!(!timer.is_enabled());
    }

    #[test]
    fn enabled_time_advances() {
        let mut timer = ElapsedTimer::new(TS);
        for _ in 0..10 {
            timer.update(true);
        }
        assert!(approx_eq(timer.elapsed_time(), 0.010));
        assert!(timer.is_enabled());
    }

    #[test]
    fn mixed_enable_disable() {
        let mut timer = ElapsedTimer::new(TS);
        // 5 enabled
        for _ in 0..5 {
            timer.update(true);
        }
        // 10 disabled
        for _ in 0..10 {
            timer.update(false);
        }
        // 5 more enabled
        for _ in 0..5 {
            timer.update(true);
        }
        assert!(approx_eq(timer.elapsed_time(), 0.010));
    }

    #[test]
    fn has_elapsed_below_threshold() {
        let mut timer = ElapsedTimer::new(TS);
        for _ in 0..5 {
            timer.update(true);
        }
        assert!(!timer.has_elapsed(0.010));
    }

    #[test]
    fn has_elapsed_at_threshold() {
        let mut timer = ElapsedTimer::new(TS);
        for _ in 0..10 {
            timer.update(true);
        }
        assert!(timer.has_elapsed(0.010));
    }

    #[test]
    fn has_elapsed_above_threshold() {
        let mut timer = ElapsedTimer::new(TS);
        for _ in 0..15 {
            timer.update(true);
        }
        assert!(timer.has_elapsed(0.010));
    }

    #[test]
    fn remaining_time_before_threshold() {
        let mut timer = ElapsedTimer::new(TS);
        for _ in 0..3 {
            timer.update(true);
        }
        let remaining = timer.remaining_time(0.010);
        assert!(approx_eq(remaining, 0.007));
    }

    #[test]
    fn remaining_time_at_threshold() {
        let mut timer = ElapsedTimer::new(TS);
        for _ in 0..10 {
            timer.update(true);
        }
        assert!(approx_eq(timer.remaining_time(0.010), 0.0));
    }

    #[test]
    fn remaining_time_past_threshold() {
        let mut timer = ElapsedTimer::new(TS);
        for _ in 0..15 {
            timer.update(true);
        }
        assert!(approx_eq(timer.remaining_time(0.010), 0.0));
    }

    #[test]
    fn reset_clears_elapsed() {
        let mut timer = ElapsedTimer::new(TS);
        for _ in 0..10 {
            timer.update(true);
        }
        assert!(timer.elapsed_time() > 0.0);
        timer.reset();
        assert!(approx_eq(timer.elapsed_time(), 0.0));
    }

    #[test]
    fn reset_to_sets_initial_value() {
        let mut timer = ElapsedTimer::new(TS);
        timer.reset_to(1.0);
        assert!(approx_eq(timer.elapsed_time(), 1.0));
        // Continues from there
        for _ in 0..10 {
            timer.update(true);
        }
        assert!(approx_eq(timer.elapsed_time(), 1.010));
    }

    #[test]
    fn configure_sampling_time() {
        let mut timer = ElapsedTimer::new(TS);
        timer.configure_sampling_time(0.01); // 10 ms
        for _ in 0..10 {
            timer.update(true);
        }
        assert!(approx_eq(timer.elapsed_time(), 0.1));
    }

    #[test]
    fn initial_state() {
        let timer = ElapsedTimer::new(TS);
        assert!(approx_eq(timer.elapsed_time(), 0.0));
        assert!(!timer.is_enabled());
        assert!(timer.has_elapsed(0.0)); // 0.0 >= 0.0 is true
    }

    #[test]
    fn has_elapsed_zero_threshold() {
        let timer = ElapsedTimer::new(TS);
        // elapsed=0.0, threshold=0.0 → 0.0 >= 0.0 is true
        assert!(timer.has_elapsed(0.0));
    }
}
