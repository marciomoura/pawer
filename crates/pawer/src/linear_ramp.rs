//! Linear ramp generator.
//!
//! Ported from C++ ramp utilities. Ramps a value linearly from an initial to a
//! final value over a configurable duration. Suitable for `no_std` embedded
//! targets with no dynamic allocation.

use crate::types::Real;

/// Configuration for a [`LinearRamp`].
pub struct LinearRampConfig {
    /// Starting value of the ramp.
    pub initial: Real,
    /// Target value of the ramp (`final` is a Rust keyword, so we use
    /// `final_value`).
    pub final_value: Real,
    /// Ramp duration in seconds. A duration of zero causes the output to jump
    /// immediately to `final_value`.
    pub duration: Real,
}

/// Linear ramp generator that advances once per `update` call.
///
/// Typical usage:
/// 1. Create with [`LinearRamp::new`], passing the control-loop sampling time.
/// 2. Call [`LinearRamp::configure`] to set the ramp parameters.
/// 3. Call [`LinearRamp::update`] each control cycle; the `enable` flag
///    controls whether the ramp advances.
pub struct LinearRamp {
    sampling_time: Real,
    config: LinearRampConfig,
    elapsed: Real,
    range: Real,
    inv_duration: Real,
}

impl LinearRamp {
    /// Create a new ramp generator with the given control-loop sampling time
    /// (in seconds).
    pub fn new(sampling_time: Real) -> Self {
        Self {
            sampling_time,
            config: LinearRampConfig {
                initial: 0.0,
                final_value: 0.0,
                duration: 0.0,
            },
            elapsed: 0.0,
            range: 0.0,
            inv_duration: 0.0,
        }
    }

    /// Load a new ramp configuration. Resets elapsed time to zero.
    pub fn configure(&mut self, config: LinearRampConfig) {
        self.config = config;
        self.elapsed = 0.0;
        self.range = self.config.final_value - self.config.initial;
        self.inv_duration = if self.config.duration > 0.0 {
            1.0 / self.config.duration
        } else {
            0.0
        };
    }

    /// Compute the current ramp output.
    ///
    /// When `enable` is `true` the internal elapsed-time counter advances by
    /// one sampling period. When `false` the output is computed at the current
    /// position but time does not advance.
    ///
    /// If `duration` is zero (or negative) the output is always `final_value`.
    pub fn update(&mut self, enable: bool) -> Real {
        if self.config.duration <= 0.0 {
            if enable {
                self.elapsed += self.sampling_time;
            }
            return self.config.final_value;
        }

        let progress = clamp01(self.elapsed * self.inv_duration);
        let output = self.config.initial + self.range * progress;

        if enable {
            self.elapsed += self.sampling_time;
        }

        output
    }

    /// Returns `true` when the ramp has reached (or exceeded) its configured
    /// duration.
    pub fn is_finished(&self) -> bool {
        if self.config.duration <= 0.0 {
            return true;
        }
        self.elapsed >= self.config.duration
    }

    /// Reset the elapsed time to zero so the ramp can be re-run with the same
    /// configuration.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Return the current elapsed time in seconds.
    pub fn elapsed_time(&self) -> Real {
        self.elapsed
    }
}

#[inline]
fn clamp01(v: Real) -> Real {
    if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: Real = 1e-6;

    fn approx_eq(a: Real, b: Real) -> bool {
        libm::fabsf(a - b) < EPSILON
    }

    // -- Basic ramp 0 → 10 over 1 s, Ts = 0.1 s ----------------------------

    #[test]
    fn ramp_halfway() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 10.0,
            duration: 1.0,
        });

        // 5 updates with enable → elapsed reaches 0.5 s
        let mut out = 0.0;
        for _ in 0..5 {
            out = ramp.update(true);
        }
        // After 5 calls: outputs at elapsed 0.0, 0.1, 0.2, 0.3, 0.4
        // Last output is at elapsed=0.4 → progress=0.4 → 4.0
        // elapsed is now 0.5
        assert!(approx_eq(out, 4.0));
        assert!(approx_eq(ramp.elapsed_time(), 0.5));
    }

    #[test]
    fn ramp_complete() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 10.0,
            duration: 1.0,
        });

        let mut out = 0.0;
        for _ in 0..10 {
            out = ramp.update(true);
        }
        // 10 calls: last output at elapsed=0.9 → progress=0.9 → 9.0
        assert!(approx_eq(out, 9.0));

        // 11th call: elapsed=1.0 → progress=1.0 → 10.0
        out = ramp.update(true);
        assert!(approx_eq(out, 10.0));
    }

    #[test]
    fn ramp_saturates_past_duration() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 10.0,
            duration: 1.0,
        });

        // Run 15 updates (beyond duration)
        let mut out = 0.0;
        for _ in 0..15 {
            out = ramp.update(true);
        }
        assert!(approx_eq(out, 10.0));
    }

    // -- Finished flag -------------------------------------------------------

    #[test]
    fn is_finished_initially_false() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 10.0,
            duration: 1.0,
        });
        assert!(!ramp.is_finished());
    }

    #[test]
    fn is_finished_after_duration() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 10.0,
            duration: 1.0,
        });
        for _ in 0..10 {
            ramp.update(true);
        }
        assert!(ramp.is_finished());
    }

    // -- Zero duration -------------------------------------------------------

    #[test]
    fn zero_duration_returns_final_value_immediately() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 42.0,
            duration: 0.0,
        });
        assert!(approx_eq(ramp.update(true), 42.0));
        assert!(ramp.is_finished());
    }

    #[test]
    fn zero_duration_always_finished() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 7.0,
            duration: 0.0,
        });
        assert!(ramp.is_finished());
    }

    // -- Enable / disable ----------------------------------------------------

    #[test]
    fn disabled_holds_output() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 10.0,
            duration: 1.0,
        });

        // Advance 3 steps
        for _ in 0..3 {
            ramp.update(true);
        }
        let held = ramp.update(false);

        // Calling with enable=false should not advance elapsed
        let again = ramp.update(false);
        assert!(approx_eq(held, again));
        assert!(approx_eq(ramp.elapsed_time(), 0.3));
    }

    #[test]
    fn disabled_then_enabled_continues() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 10.0,
            duration: 1.0,
        });

        // Advance 3 steps (elapsed → 0.3)
        for _ in 0..3 {
            ramp.update(true);
        }

        // Hold for 5 calls
        for _ in 0..5 {
            ramp.update(false);
        }
        assert!(approx_eq(ramp.elapsed_time(), 0.3));

        // Resume — elapsed is still 0.3, so output = 3.0
        let out = ramp.update(true);
        assert!(approx_eq(out, 3.0));
        assert!(approx_eq(ramp.elapsed_time(), 0.4));
    }

    // -- Decreasing ramp (10 → 0) -------------------------------------------

    #[test]
    fn decreasing_ramp() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 10.0,
            final_value: 0.0,
            duration: 1.0,
        });

        // First output at elapsed=0 → 10.0
        let out = ramp.update(true);
        assert!(approx_eq(out, 10.0));
    }

    #[test]
    fn decreasing_ramp_midpoint() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 10.0,
            final_value: 0.0,
            duration: 1.0,
        });

        let mut out = 0.0;
        for _ in 0..6 {
            out = ramp.update(true);
        }
        // output at elapsed=0.5 → 10 + (-10)*0.5 = 5.0
        assert!(approx_eq(out, 5.0));
    }

    // -- Reset ---------------------------------------------------------------

    #[test]
    fn reset_restarts_ramp() {
        let mut ramp = LinearRamp::new(0.1);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 10.0,
            duration: 1.0,
        });

        for _ in 0..5 {
            ramp.update(true);
        }
        assert!(approx_eq(ramp.elapsed_time(), 0.5));

        ramp.reset();
        assert!(approx_eq(ramp.elapsed_time(), 0.0));
        assert!(!ramp.is_finished());

        // First output after reset at elapsed=0 → 0.0
        let out = ramp.update(true);
        assert!(approx_eq(out, 0.0));
    }

    // -- Negative initial / final values -------------------------------------

    #[test]
    fn negative_to_positive_ramp() {
        let mut ramp = LinearRamp::new(0.25);
        ramp.configure(LinearRampConfig {
            initial: -10.0,
            final_value: 10.0,
            duration: 1.0,
        });

        // elapsed=0 → output = -10
        let out = ramp.update(true);
        assert!(approx_eq(out, -10.0));

        // elapsed=0.25 → progress=0.25 → -10 + 20*0.25 = -5
        let out = ramp.update(true);
        assert!(approx_eq(out, -5.0));

        // elapsed=0.5 → progress=0.5 → -10 + 20*0.5 = 0
        let out = ramp.update(true);
        assert!(approx_eq(out, 0.0));
    }

    // -- Elapsed time accessor -----------------------------------------------

    #[test]
    fn elapsed_time_tracks_correctly() {
        let mut ramp = LinearRamp::new(0.2);
        ramp.configure(LinearRampConfig {
            initial: 0.0,
            final_value: 1.0,
            duration: 1.0,
        });

        assert!(approx_eq(ramp.elapsed_time(), 0.0));
        ramp.update(true);
        assert!(approx_eq(ramp.elapsed_time(), 0.2));
        ramp.update(true);
        assert!(approx_eq(ramp.elapsed_time(), 0.4));
        ramp.update(false);
        assert!(approx_eq(ramp.elapsed_time(), 0.4));
    }
}
