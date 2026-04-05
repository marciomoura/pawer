//! SI ↔ per-unit conversion helpers.
//!
//! Ported from C++ `per_unit.hpp`.
//! Per-unit values are obtained by dividing an SI quantity by its base value.

use crate::types::Real;

/// Convert an SI value to per-unit by dividing by the base value.
///
/// # Panics (debug only)
/// Panics if `base_value` is zero.
#[inline]
pub fn to_pu(si_value: Real, base_value: Real) -> Real {
    debug_assert!(base_value != 0.0, "Base value must not be zero");
    si_value / base_value
}

/// Convert a per-unit value to SI by multiplying by the base value.
///
/// # Panics (debug only)
/// Panics if `base_value` is zero.
#[inline]
pub fn to_si(pu_value: Real, base_value: Real) -> Real {
    debug_assert!(base_value != 0.0, "Base value must not be zero");
    pu_value * base_value
}

// ---------------------------------------------------------------------------
// Tests — ported from C++ per_unit_test.cpp
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f32 = 1e-6;

    fn approx_eq(a: f32, b: f32) -> bool {
        libm::fabsf(a - b) < TOL
    }

    #[test]
    fn to_pu_basic() {
        assert!(approx_eq(to_pu(100.0, 200.0), 0.5));
    }

    #[test]
    fn to_si_basic() {
        assert!(approx_eq(to_si(0.5, 200.0), 100.0));
    }

    #[test]
    fn round_trip() {
        let value: Real = 123.456;
        let base: Real = 200.0;
        assert!(approx_eq(to_si(to_pu(value, base), base), value));
    }

    #[test]
    fn to_pu_unity_base() {
        assert!(approx_eq(to_pu(42.0, 1.0), 42.0));
    }

    #[test]
    fn to_si_unity_base() {
        assert!(approx_eq(to_si(42.0, 1.0), 42.0));
    }

    #[test]
    fn to_pu_equal_values() {
        assert!(approx_eq(to_pu(200.0, 200.0), 1.0));
    }

    #[test]
    fn to_si_one_pu() {
        assert!(approx_eq(to_si(1.0, 200.0), 200.0));
    }

    #[test]
    fn to_pu_negative_value() {
        assert!(approx_eq(to_pu(-100.0, 200.0), -0.5));
    }

    #[test]
    fn to_si_negative_pu() {
        assert!(approx_eq(to_si(-0.5, 200.0), -100.0));
    }

    #[test]
    fn round_trip_various_bases() {
        for &base in &[1.0, 50.0, 230.0, 400.0, 1000.0] {
            let value: Real = 77.7;
            assert!(approx_eq(to_si(to_pu(value, base), base), value));
        }
    }

    #[test]
    fn to_pu_small_base() {
        let result = to_pu(1.0, 0.001);
        assert!((result - 1000.0).abs() < 0.1); // f32 precision with small divisor
    }

    #[test]
    fn to_si_large_base() {
        assert!(approx_eq(to_si(0.001, 1000.0), 1.0));
    }
}
