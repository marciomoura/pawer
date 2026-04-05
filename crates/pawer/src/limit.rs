//! Clamping / saturation helpers for control signals.
//!
//! Ported from C++ `limit.hpp` (namespace `yeet::control`).
//! Provides both `f32` (with epsilon comparison) and `i32` (exact comparison)
//! variants.

const EPSILON: f32 = 1e-10;

/// Result of a clamping operation that also reports whether the value was
/// limited.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LimitResult<T> {
    pub value: T,
    pub was_limited: bool,
}

// ---------------------------------------------------------------------------
// f32 functions
// ---------------------------------------------------------------------------

/// Clamp `value` to at most `max_value`.
#[inline]
pub fn upper(value: f32, max_value: f32) -> f32 {
    if value > max_value {
        max_value
    } else {
        value
    }
}

/// Clamp `value` to at least `min_value`.
#[inline]
pub fn lower(value: f32, min_value: f32) -> f32 {
    if value < min_value {
        min_value
    } else {
        value
    }
}

/// Clamp `value` to `[min_value, max_value]`.
///
/// # Panics (debug only)
/// Panics if `max_value <= min_value`.
#[inline]
pub fn range(value: f32, min_value: f32, max_value: f32) -> f32 {
    debug_assert!(max_value > min_value, "max_value must be greater than min_value");
    lower(upper(value, max_value), min_value)
}

/// Returns `true` if `value` exceeds `max_value` by more than [`EPSILON`].
#[inline]
pub fn is_above_upper_limit(value: f32, max_value: f32) -> bool {
    (value - max_value) > EPSILON
}

/// Clamp to `[min_value, max_value]` and report whether the value was limited
/// (either direction). Uses epsilon comparison.
#[inline]
pub fn range_with_status(value: f32, min_value: f32, max_value: f32) -> LimitResult<f32> {
    let clamped = range(value, min_value, max_value);
    let diff = libm::fabsf(value - clamped);
    LimitResult {
        value: clamped,
        was_limited: diff > EPSILON,
    }
}

/// Clamp to `[min_value, max_value]` but only flag `was_limited` when the
/// **upper** limit was hit (lower clamping is silent).
#[inline]
pub fn range_with_upper_limit_status(
    value: f32,
    min_value: f32,
    max_value: f32,
) -> LimitResult<f32> {
    let clamped = range(value, min_value, max_value);
    let was_upper_limited = (value - max_value) > EPSILON;
    LimitResult {
        value: clamped,
        was_limited: was_upper_limited,
    }
}

// ---------------------------------------------------------------------------
// i32 functions
// ---------------------------------------------------------------------------

/// Clamp `value` to at most `max_value` (integer).
#[inline]
pub fn upper_i32(value: i32, max_value: i32) -> i32 {
    if value > max_value {
        max_value
    } else {
        value
    }
}

/// Clamp `value` to at least `min_value` (integer).
#[inline]
pub fn lower_i32(value: i32, min_value: i32) -> i32 {
    if value < min_value {
        min_value
    } else {
        value
    }
}

/// Clamp `value` to `[min_value, max_value]` (integer).
#[inline]
pub fn range_i32(value: i32, min_value: i32, max_value: i32) -> i32 {
    debug_assert!(max_value > min_value, "max_value must be greater than min_value");
    lower_i32(upper_i32(value, max_value), min_value)
}

/// Returns `true` if `value` exceeds `max_value` (integer, exact comparison).
#[inline]
pub fn is_above_upper_limit_i32(value: i32, max_value: i32) -> bool {
    value > max_value
}

/// Clamp to `[min_value, max_value]` and report whether the value was limited
/// (integer, exact comparison).
#[inline]
pub fn range_with_status_i32(value: i32, min_value: i32, max_value: i32) -> LimitResult<i32> {
    let clamped = range_i32(value, min_value, max_value);
    LimitResult {
        value: clamped,
        was_limited: value != clamped,
    }
}

/// Clamp to `[min_value, max_value]` but only flag `was_limited` when the
/// **upper** limit was hit (integer).
#[inline]
pub fn range_with_upper_limit_status_i32(
    value: i32,
    min_value: i32,
    max_value: i32,
) -> LimitResult<i32> {
    let clamped = range_i32(value, min_value, max_value);
    LimitResult {
        value: clamped,
        was_limited: value > max_value,
    }
}

// ---------------------------------------------------------------------------
// Tests — ported from C++ limit_tests.cpp
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    // -- f32: upper ----------------------------------------------------------

    #[test]
    fn upper_below_max() {
        assert_eq!(upper(5.0, 10.0), 5.0);
    }

    #[test]
    fn upper_at_max() {
        assert_eq!(upper(10.0, 10.0), 10.0);
    }

    #[test]
    fn upper_above_max() {
        assert_eq!(upper(15.0, 10.0), 10.0);
    }

    // -- f32: lower ----------------------------------------------------------

    #[test]
    fn lower_above_min() {
        assert_eq!(lower(5.0, 0.0), 5.0);
    }

    #[test]
    fn lower_at_min() {
        assert_eq!(lower(0.0, 0.0), 0.0);
    }

    #[test]
    fn lower_below_min() {
        assert_eq!(lower(-5.0, 0.0), 0.0);
    }

    // -- f32: range ----------------------------------------------------------

    #[test]
    fn range_within() {
        assert_eq!(range(5.0, 0.0, 10.0), 5.0);
    }

    #[test]
    fn range_at_lower_bound() {
        assert_eq!(range(0.0, 0.0, 10.0), 0.0);
    }

    #[test]
    fn range_at_upper_bound() {
        assert_eq!(range(10.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn range_below() {
        assert_eq!(range(-5.0, 0.0, 10.0), 0.0);
    }

    #[test]
    fn range_above() {
        assert_eq!(range(15.0, 0.0, 10.0), 10.0);
    }

    // -- f32: is_above_upper_limit -------------------------------------------

    #[test]
    fn is_above_upper_limit_below() {
        assert!(!is_above_upper_limit(5.0, 10.0));
    }

    #[test]
    fn is_above_upper_limit_at() {
        assert!(!is_above_upper_limit(10.0, 10.0));
    }

    #[test]
    fn is_above_upper_limit_above() {
        assert!(is_above_upper_limit(11.0, 10.0));
    }

    #[test]
    fn is_above_upper_limit_just_above_epsilon() {
        // 1e-11 above max — within epsilon, so NOT above limit
        assert!(!is_above_upper_limit(10.0 + 1e-11, 10.0));
    }

    #[test]
    fn is_above_upper_limit_beyond_epsilon() {
        // Clearly above max — exceeds epsilon
        assert!(is_above_upper_limit(10.0 + 1e-5, 10.0));
    }

    // -- f32: range_with_status ----------------------------------------------

    #[test]
    fn range_with_status_within() {
        let r = range_with_status(5.0, 0.0, 10.0);
        assert_eq!(r.value, 5.0);
        assert!(!r.was_limited);
    }

    #[test]
    fn range_with_status_below() {
        let r = range_with_status(-5.0, 0.0, 10.0);
        assert_eq!(r.value, 0.0);
        assert!(r.was_limited);
    }

    #[test]
    fn range_with_status_above() {
        let r = range_with_status(15.0, 0.0, 10.0);
        assert_eq!(r.value, 10.0);
        assert!(r.was_limited);
    }

    #[test]
    fn range_with_status_epsilon_not_limited() {
        // Value within epsilon of the boundary → not limited
        let r = range_with_status(10.0 + 1e-11, 0.0, 10.0);
        assert!(!r.was_limited);
    }

    #[test]
    fn range_with_status_epsilon_limited() {
        // Value clearly beyond the boundary
        let r = range_with_status(10.0 + 1e-5, 0.0, 10.0);
        assert!(r.was_limited);
    }

    // -- f32: range_with_upper_limit_status ----------------------------------

    #[test]
    fn range_with_upper_limit_status_within() {
        let r = range_with_upper_limit_status(5.0, 0.0, 10.0);
        assert_eq!(r.value, 5.0);
        assert!(!r.was_limited);
    }

    #[test]
    fn range_with_upper_limit_status_below_lower() {
        // Lower limit hit — but was_limited should be false (only upper triggers)
        let r = range_with_upper_limit_status(-5.0, 0.0, 10.0);
        assert_eq!(r.value, 0.0);
        assert!(!r.was_limited);
    }

    #[test]
    fn range_with_upper_limit_status_above_upper() {
        let r = range_with_upper_limit_status(15.0, 0.0, 10.0);
        assert_eq!(r.value, 10.0);
        assert!(r.was_limited);
    }

    #[test]
    fn range_with_upper_limit_status_epsilon_not_limited() {
        let r = range_with_upper_limit_status(10.0 + 1e-11, 0.0, 10.0);
        assert!(!r.was_limited);
    }

    #[test]
    fn range_with_upper_limit_status_epsilon_limited() {
        let r = range_with_upper_limit_status(10.0 + 1e-5, 0.0, 10.0);
        assert!(r.was_limited);
    }

    // -- f32: negative range -------------------------------------------------

    #[test]
    fn range_negative_bounds() {
        assert_eq!(range(-3.0, -10.0, -1.0), -3.0);
        assert_eq!(range(-15.0, -10.0, -1.0), -10.0);
        assert_eq!(range(0.0, -10.0, -1.0), -1.0);
    }

    // -- i32: upper ----------------------------------------------------------

    #[test]
    fn upper_i32_below_max() {
        assert_eq!(upper_i32(5, 10), 5);
    }

    #[test]
    fn upper_i32_at_max() {
        assert_eq!(upper_i32(10, 10), 10);
    }

    #[test]
    fn upper_i32_above_max() {
        assert_eq!(upper_i32(15, 10), 10);
    }

    // -- i32: lower ----------------------------------------------------------

    #[test]
    fn lower_i32_above_min() {
        assert_eq!(lower_i32(5, 0), 5);
    }

    #[test]
    fn lower_i32_at_min() {
        assert_eq!(lower_i32(0, 0), 0);
    }

    #[test]
    fn lower_i32_below_min() {
        assert_eq!(lower_i32(-5, 0), 0);
    }

    // -- i32: range ----------------------------------------------------------

    #[test]
    fn range_i32_within() {
        assert_eq!(range_i32(5, 0, 10), 5);
    }

    #[test]
    fn range_i32_at_bounds() {
        assert_eq!(range_i32(0, 0, 10), 0);
        assert_eq!(range_i32(10, 0, 10), 10);
    }

    #[test]
    fn range_i32_below() {
        assert_eq!(range_i32(-5, 0, 10), 0);
    }

    #[test]
    fn range_i32_above() {
        assert_eq!(range_i32(15, 0, 10), 10);
    }

    // -- i32: is_above_upper_limit -------------------------------------------

    #[test]
    fn is_above_upper_limit_i32_below() {
        assert!(!is_above_upper_limit_i32(5, 10));
    }

    #[test]
    fn is_above_upper_limit_i32_at() {
        assert!(!is_above_upper_limit_i32(10, 10));
    }

    #[test]
    fn is_above_upper_limit_i32_above() {
        assert!(is_above_upper_limit_i32(11, 10));
    }

    // -- i32: range_with_status ----------------------------------------------

    #[test]
    fn range_with_status_i32_within() {
        let r = range_with_status_i32(5, 0, 10);
        assert_eq!(r.value, 5);
        assert!(!r.was_limited);
    }

    #[test]
    fn range_with_status_i32_below() {
        let r = range_with_status_i32(-5, 0, 10);
        assert_eq!(r.value, 0);
        assert!(r.was_limited);
    }

    #[test]
    fn range_with_status_i32_above() {
        let r = range_with_status_i32(15, 0, 10);
        assert_eq!(r.value, 10);
        assert!(r.was_limited);
    }

    // -- i32: range_with_upper_limit_status ----------------------------------

    #[test]
    fn range_with_upper_limit_status_i32_within() {
        let r = range_with_upper_limit_status_i32(5, 0, 10);
        assert_eq!(r.value, 5);
        assert!(!r.was_limited);
    }

    #[test]
    fn range_with_upper_limit_status_i32_below_lower() {
        let r = range_with_upper_limit_status_i32(-5, 0, 10);
        assert_eq!(r.value, 0);
        assert!(!r.was_limited);
    }

    #[test]
    fn range_with_upper_limit_status_i32_above_upper() {
        let r = range_with_upper_limit_status_i32(15, 0, 10);
        assert_eq!(r.value, 10);
        assert!(r.was_limited);
    }

    // -- i32: negative range -------------------------------------------------

    #[test]
    fn range_i32_negative_bounds() {
        assert_eq!(range_i32(-3, -10, -1), -3);
        assert_eq!(range_i32(-15, -10, -1), -10);
        assert_eq!(range_i32(0, -10, -1), -1);
    }
}
