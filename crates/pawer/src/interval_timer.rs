use crate::types::Real;

/// Interval Timer with microsecond precision.
///
/// Measures elapsed time in microseconds with explicit start/stop
/// semantics. Each [`update`] call advances the counter by one sampling
/// period (converted to µs) while the timer is running.
pub struct IntervalTimer {
    sampling_time: Real,
    elapsed_time_us: Real,
    running: bool,
}

impl IntervalTimer {
    pub fn new(sampling_time: Real) -> Self {
        debug_assert!(sampling_time > 0.0);
        Self {
            sampling_time,
            elapsed_time_us: 0.0,
            running: false,
        }
    }

    pub fn start(&mut self) {
        self.elapsed_time_us = 0.0;
        self.running = true;
    }

    pub fn stop(&mut self) -> Real {
        self.running = false;
        self.elapsed_time_us
    }

    pub fn update(&mut self) {
        if self.running {
            self.elapsed_time_us += self.sampling_time * 1e6;
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn elapsed_time_us(&self) -> Real {
        self.elapsed_time_us
    }

    pub fn reset(&mut self) {
        self.elapsed_time_us = 0.0;
        self.running = false;
    }

    pub fn configure(&mut self, sampling_time: Real) {
        debug_assert!(sampling_time > 0.0);
        self.sampling_time = sampling_time;
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TS: Real = 0.001; // 1 ms → 1000 µs per sample
    const EPSILON: Real = 1e-1; // µs-level comparisons on f32

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn not_started_elapsed_is_zero() {
        let timer = IntervalTimer::new(TS);
        assert!(approx_eq(timer.elapsed_time_us(), 0.0));
        assert!(!timer.is_running());
    }

    #[test]
    fn update_without_start_does_nothing() {
        let mut timer = IntervalTimer::new(TS);
        for _ in 0..10 {
            timer.update();
        }
        assert!(approx_eq(timer.elapsed_time_us(), 0.0));
    }

    #[test]
    fn start_and_update_10_times() {
        let mut timer = IntervalTimer::new(TS);
        timer.start();
        assert!(timer.is_running());
        for _ in 0..10 {
            timer.update();
        }
        // 10 samples × 1000 µs = 10000 µs
        assert!(approx_eq(timer.elapsed_time_us(), 10_000.0));
    }

    #[test]
    fn stop_returns_elapsed() {
        let mut timer = IntervalTimer::new(TS);
        timer.start();
        for _ in 0..5 {
            timer.update();
        }
        let elapsed = timer.stop();
        assert!(approx_eq(elapsed, 5_000.0));
        assert!(!timer.is_running());
    }

    #[test]
    fn update_after_stop_does_not_advance() {
        let mut timer = IntervalTimer::new(TS);
        timer.start();
        for _ in 0..5 {
            timer.update();
        }
        timer.stop();
        for _ in 0..10 {
            timer.update();
        }
        assert!(approx_eq(timer.elapsed_time_us(), 5_000.0));
    }

    #[test]
    fn reset_clears_everything() {
        let mut timer = IntervalTimer::new(TS);
        timer.start();
        for _ in 0..10 {
            timer.update();
        }
        timer.reset();
        assert!(approx_eq(timer.elapsed_time_us(), 0.0));
        assert!(!timer.is_running());
    }

    #[test]
    fn start_resets_elapsed_to_zero() {
        let mut timer = IntervalTimer::new(TS);
        timer.start();
        for _ in 0..10 {
            timer.update();
        }
        // Start again — should reset counter
        timer.start();
        assert!(approx_eq(timer.elapsed_time_us(), 0.0));
        assert!(timer.is_running());
    }

    #[test]
    fn configure_changes_sampling_time() {
        let mut timer = IntervalTimer::new(TS);
        timer.configure(0.0001); // 0.1 ms → 100 µs per sample
        assert!(!timer.is_running()); // configure calls reset
        timer.start();
        for _ in 0..10 {
            timer.update();
        }
        // 10 × 100 µs = 1000 µs
        assert!(approx_eq(timer.elapsed_time_us(), 1_000.0));
    }

    #[test]
    fn multiple_start_stop_cycles() {
        let mut timer = IntervalTimer::new(TS);

        // Cycle 1
        timer.start();
        for _ in 0..3 {
            timer.update();
        }
        let e1 = timer.stop();
        assert!(approx_eq(e1, 3_000.0));

        // Cycle 2
        timer.start();
        for _ in 0..7 {
            timer.update();
        }
        let e2 = timer.stop();
        assert!(approx_eq(e2, 7_000.0));
    }

    #[test]
    fn fine_sampling_time() {
        // 10 µs sampling → 10 µs per update
        let mut timer = IntervalTimer::new(0.00001);
        timer.start();
        for _ in 0..100 {
            timer.update();
        }
        // 100 × 10 µs = 1000 µs
        assert!(approx_eq(timer.elapsed_time_us(), 1_000.0));
    }
}
