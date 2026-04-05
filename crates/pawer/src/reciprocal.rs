//! Pre-calculated reciprocal for fast division.
//!
//! Stores `value` and `1 / value` so that division can be performed as a
//! single multiplication — useful in tight control loops.

use crate::types::Real;

/// Pre-calculated reciprocal for fast division.
///
/// Stores a value and its reciprocal (`1 / value`). Division by this value is
/// implemented as multiplication by the reciprocal, avoiding a potentially
/// expensive hardware divide on embedded targets.
#[derive(Debug, Clone, Copy)]
pub struct Reciprocal {
    value: Real,
    reciprocal: Real,
}

impl Reciprocal {
    /// Create a new reciprocal. Computes and stores `1 / value`.
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
// Operator overloads: `f32 / Reciprocal`
// ---------------------------------------------------------------------------

impl core::ops::Div<Reciprocal> for f32 {
    type Output = f32;

    #[inline]
    fn div(self, rhs: Reciprocal) -> f32 {
        rhs.divide(self)
    }
}

impl core::ops::Div<&Reciprocal> for f32 {
    type Output = f32;

    #[inline]
    fn div(self, rhs: &Reciprocal) -> f32 {
        rhs.divide(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f32 = 1e-6;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < TOL
    }

    #[test]
    fn construction_value_and_reciprocal() {
        let reciprocal = Reciprocal::new(4.0);
        assert_eq!(reciprocal.value(), 4.0);
        assert!(approx_eq(reciprocal.reciprocal(), 0.25));
    }

    #[test]
    fn divide_method() {
        let reciprocal = Reciprocal::new(5.0);
        assert!(approx_eq(reciprocal.divide(20.0), 4.0));
    }

    #[test]
    fn div_operator_owned() {
        let reciprocal = Reciprocal::new(5.0);
        assert!(approx_eq(20.0 / reciprocal, 4.0));
    }

    #[test]
    fn div_operator_ref() {
        let reciprocal = Reciprocal::new(5.0);
        assert!(approx_eq(20.0 / &reciprocal, 4.0));
    }

    #[test]
    fn divide_one() {
        let reciprocal = Reciprocal::new(1.0);
        assert!(approx_eq(reciprocal.divide(42.0), 42.0));
    }

    #[test]
    fn divide_small_value() {
        let reciprocal = Reciprocal::new(0.001);
        assert!((reciprocal.divide(1.0) - 1000.0).abs() < 0.1);
    }

    #[test]
    fn divide_large_value() {
        let reciprocal = Reciprocal::new(1000.0);
        assert!(approx_eq(reciprocal.divide(5000.0), 5.0));
    }

    #[test]
    fn divide_negative_numerator() {
        let reciprocal = Reciprocal::new(4.0);
        assert!(approx_eq(reciprocal.divide(-20.0), -5.0));
    }

    #[test]
    fn divide_negative_divisor() {
        let reciprocal = Reciprocal::new(-4.0);
        assert!(approx_eq(reciprocal.divide(20.0), -5.0));
    }

    #[test]
    fn divide_zero_numerator() {
        let reciprocal = Reciprocal::new(5.0);
        assert_eq!(reciprocal.divide(0.0), 0.0);
    }
}
