//! Pre-calculated reciprocal for fast division.
//!
//! Ported from C++ `precomputed_divisor.hpp`.
//! Stores `value` and `1 / value` so that division can be performed as a
//! single multiplication — useful in tight control loops.

use crate::types::Real;

/// Pre-calculated reciprocal for fast division.
///
/// Stores a value and its reciprocal (`1 / value`). Division by this divisor
/// is implemented as multiplication by the reciprocal, avoiding a potentially
/// expensive hardware divide on embedded targets.
#[derive(Debug, Clone, Copy)]
pub struct PrecomputedDivisor {
    value: Real,
    reciprocal: Real,
}

impl PrecomputedDivisor {
    /// Create a new divisor. Computes and stores `1 / value`.
    #[inline]
    pub fn new(value: Real) -> Self {
        Self {
            value,
            reciprocal: 1.0 / value,
        }
    }

    /// The original divisor value.
    #[inline]
    pub fn value(&self) -> Real {
        self.value
    }

    /// The pre-computed reciprocal (`1 / value`).
    #[inline]
    pub fn reciprocal(&self) -> Real {
        self.reciprocal
    }

    /// Fast division: `numerator / self.value` implemented as
    /// `numerator * self.reciprocal`.
    #[inline]
    pub fn divide(&self, numerator: Real) -> Real {
        numerator * self.reciprocal
    }
}

// ---------------------------------------------------------------------------
// Operator overloads: `f32 / PrecomputedDivisor`
// ---------------------------------------------------------------------------

impl core::ops::Div<PrecomputedDivisor> for f32 {
    type Output = f32;

    #[inline]
    fn div(self, rhs: PrecomputedDivisor) -> f32 {
        self * rhs.reciprocal()
    }
}

impl core::ops::Div<&PrecomputedDivisor> for f32 {
    type Output = f32;

    #[inline]
    fn div(self, rhs: &PrecomputedDivisor) -> f32 {
        self * rhs.reciprocal()
    }
}

// ---------------------------------------------------------------------------
// Tests — ported from C++ precomputed_divisor_test.cpp
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f32 = 1e-6;

    fn approx_eq(a: f32, b: f32) -> bool {
        libm::fabsf(a - b) < TOL
    }

    #[test]
    fn construction_value_and_reciprocal() {
        let d = PrecomputedDivisor::new(4.0);
        assert_eq!(d.value(), 4.0);
        assert!(approx_eq(d.reciprocal(), 0.25));
    }

    #[test]
    fn divide_method() {
        let d = PrecomputedDivisor::new(5.0);
        assert!(approx_eq(d.divide(20.0), 4.0));
    }

    #[test]
    fn div_operator_owned() {
        let d = PrecomputedDivisor::new(5.0);
        assert!(approx_eq(20.0 / d, 4.0));
    }

    #[test]
    fn div_operator_ref() {
        let d = PrecomputedDivisor::new(5.0);
        assert!(approx_eq(20.0 / &d, 4.0));
    }

    #[test]
    fn divide_one() {
        let d = PrecomputedDivisor::new(1.0);
        assert!(approx_eq(d.divide(42.0), 42.0));
    }

    #[test]
    fn divide_small_value() {
        let d = PrecomputedDivisor::new(0.001);
        assert!((d.divide(1.0) - 1000.0).abs() < 0.1); // f32 precision with small divisor
    }

    #[test]
    fn divide_large_value() {
        let d = PrecomputedDivisor::new(1000.0);
        assert!(approx_eq(d.divide(5000.0), 5.0));
    }

    #[test]
    fn divide_negative_numerator() {
        let d = PrecomputedDivisor::new(4.0);
        assert!(approx_eq(d.divide(-20.0), -5.0));
    }

    #[test]
    fn divide_negative_divisor() {
        let d = PrecomputedDivisor::new(-4.0);
        assert!(approx_eq(d.divide(20.0), -5.0));
    }

    #[test]
    fn divide_zero_numerator() {
        let d = PrecomputedDivisor::new(5.0);
        assert_eq!(d.divide(0.0), 0.0);
    }
}
