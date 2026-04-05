//! Wrapped angle type that maps the full `u32` range to \[0, 2π).
//!
//! Arithmetic on [`AngleWrapped`] uses native `u32` overflow so wrapping past
//! 0 or 2π is free on every target, including bare-metal embedded.

use crate::constants::{PI, TWO_PI};
use crate::types::Real;

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Scale factor: 2^32 / 2π  (radians → u32).
const RAD_TO_U32_SCALE: Real = 4_294_967_296.0 / TWO_PI;

/// Scale factor: 2π / 2^32  (u32 → radians).
const U32_TO_RAD_SCALE: Real = TWO_PI / 4_294_967_296.0;

/// Convert a radian value (any range) to its wrapped `u32` representation.
fn radians_to_u32(radians: Real) -> u32 {
    let mut normalized = radians;
    if !(0.0..TWO_PI).contains(&normalized) {
        normalized -= TWO_PI * libm::floorf(normalized / TWO_PI);
    }
    (normalized * RAD_TO_U32_SCALE) as u32
}

/// Convert a `u32` representation back to radians in \[0, 2π).
fn u32_to_radians(value: u32) -> Real {
    value as Real * U32_TO_RAD_SCALE
}

// ── AngleWrapped ─────────────────────────────────────────────────────────────

/// An angle stored as a `u32` that wraps naturally to \[0, 2π) via integer
/// overflow.
#[derive(Clone, Copy, Debug, Default)]
pub struct AngleWrapped {
    value: u32,
}

impl AngleWrapped {
    /// Create an angle from a value in radians.
    pub fn new(radians: Real) -> Self {
        Self {
            value: radians_to_u32(radians),
        }
    }

    /// Create an angle from a value in radians (alias for [`new`](Self::new)).
    pub fn from_radians(radians: Real) -> Self {
        Self::new(radians)
    }

    /// Create an angle from a value in degrees.
    pub fn from_degrees(degrees: Real) -> Self {
        Self::new(degrees * PI / 180.0)
    }

    /// Return the angle in radians, in \[0, 2π).
    pub fn radians(&self) -> Real {
        u32_to_radians(self.value)
    }

    /// Return the angle in degrees, in \[0, 360).
    pub fn degrees(&self) -> Real {
        self.radians() * 180.0 / PI
    }

    /// Return the raw `u32` representation (useful for embedded bit-level
    /// work).
    pub fn raw(&self) -> u32 {
        self.value
    }
}

// ── AngleWrapped <op> AngleWrapped ───────────────────────────────────────────

impl core::ops::Add for AngleWrapped {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            value: self.value.wrapping_add(rhs.value),
        }
    }
}

impl core::ops::Sub for AngleWrapped {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            value: self.value.wrapping_sub(rhs.value),
        }
    }
}

impl core::ops::AddAssign for AngleWrapped {
    fn add_assign(&mut self, rhs: Self) {
        self.value = self.value.wrapping_add(rhs.value);
    }
}

impl core::ops::SubAssign for AngleWrapped {
    fn sub_assign(&mut self, rhs: Self) {
        self.value = self.value.wrapping_sub(rhs.value);
    }
}

// ── AngleWrapped <op> Real (radians) ─────────────────────────────────────────

impl core::ops::Add<Real> for AngleWrapped {
    type Output = Self;
    fn add(self, rhs: Real) -> Self {
        self + Self::new(rhs)
    }
}

impl core::ops::Sub<Real> for AngleWrapped {
    type Output = Self;
    fn sub(self, rhs: Real) -> Self {
        self - Self::new(rhs)
    }
}

// ── Scalar multiply / divide ─────────────────────────────────────────────────

impl core::ops::Mul<Real> for AngleWrapped {
    type Output = Self;
    fn mul(self, rhs: Real) -> Self {
        Self::new(self.radians() * rhs)
    }
}

impl core::ops::Div<Real> for AngleWrapped {
    type Output = Self;
    fn div(self, rhs: Real) -> Self {
        Self::new(self.radians() / rhs)
    }
}

// ── Negation ─────────────────────────────────────────────────────────────────

impl core::ops::Neg for AngleWrapped {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value: self.value.wrapping_neg(),
        }
    }
}

// ── Approximate equality ─────────────────────────────────────────────────────

/// Maximum u32 difference considered equal (~1e-10 radians).
const EQ_EPSILON: u32 = 682;

impl PartialEq for AngleWrapped {
    fn eq(&self, other: &Self) -> bool {
        let diff = self.value.wrapping_sub(other.value);
        // The wrapping difference can be close to 0 or close to u32::MAX
        // (i.e. just below zero); take the smaller side.
        let min_diff = diff.min(diff.wrapping_neg());
        min_diff <= EQ_EPSILON
    }
}

// ── Ordering (raw u32) ──────────────────────────────────────────────────────

impl PartialOrd for AngleWrapped {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < 1e-4
    }

    // -- Construction ---------------------------------------------------------

    #[test]
    fn default_is_zero() {
        let a = AngleWrapped::default();
        assert!(approx_eq(a.radians(), 0.0));
    }

    #[test]
    fn from_radians_value() {
        let a = AngleWrapped::new(1.0);
        assert!(approx_eq(a.radians(), 1.0));
    }

    #[test]
    fn from_degrees_value() {
        let a = AngleWrapped::from_degrees(90.0);
        assert!(approx_eq(a.radians(), PI / 2.0));
    }

    #[test]
    fn negative_wraps() {
        let a = AngleWrapped::new(-1.0);
        assert!(approx_eq(a.radians(), TWO_PI - 1.0));
    }

    #[test]
    fn greater_than_two_pi_wraps() {
        let a = AngleWrapped::new(TWO_PI + 1.0);
        assert!(approx_eq(a.radians(), 1.0));
    }

    #[test]
    fn exactly_two_pi_wraps_to_zero() {
        let a = AngleWrapped::new(TWO_PI);
        assert!(approx_eq(a.radians(), 0.0));
    }

    // -- Addition -------------------------------------------------------------

    #[test]
    fn add_two_angles() {
        let a = AngleWrapped::new(1.0);
        let b = AngleWrapped::new(2.0);
        let c = a + b;
        assert!(approx_eq(c.radians(), 3.0));
    }

    #[test]
    fn add_wraps_past_two_pi() {
        let a = AngleWrapped::new(5.0);
        let b = AngleWrapped::new(2.0);
        let c = a + b;
        // 5 + 2 = 7, 7 - 2π ≈ 0.7168
        assert!(approx_eq(c.radians(), 7.0 - TWO_PI));
    }

    #[test]
    fn add_real() {
        let a = AngleWrapped::new(1.0);
        let b = a + 0.5;
        assert!(approx_eq(b.radians(), 1.5));
    }

    // -- Subtraction ----------------------------------------------------------

    #[test]
    fn sub_wraps() {
        let a = AngleWrapped::new(1.0);
        let b = AngleWrapped::new(2.0);
        let c = a - b;
        // 1 - 2 wraps to 2π - 1
        assert!(approx_eq(c.radians(), TWO_PI - 1.0));
    }

    #[test]
    fn sub_real() {
        let a = AngleWrapped::new(2.0);
        let b = a - 0.5;
        assert!(approx_eq(b.radians(), 1.5));
    }

    // -- Negation -------------------------------------------------------------

    #[test]
    fn negate_angle() {
        let a = AngleWrapped::new(1.0);
        let b = -a;
        assert!(approx_eq(b.radians(), TWO_PI - 1.0));
    }

    // -- Scalar multiply ------------------------------------------------------

    #[test]
    fn scalar_mul() {
        let a = AngleWrapped::new(2.0);
        let b = a * 2.0;
        assert!(approx_eq(b.radians(), 4.0));
    }

    #[test]
    fn scalar_div() {
        let a = AngleWrapped::new(4.0);
        let b = a / 2.0;
        assert!(approx_eq(b.radians(), 2.0));
    }

    // -- Equality -------------------------------------------------------------

    #[test]
    fn equality_same() {
        let a = AngleWrapped::new(1.0);
        let b = AngleWrapped::new(1.0);
        assert_eq!(a, b);
    }

    #[test]
    fn inequality_different() {
        let a = AngleWrapped::new(1.0);
        let b = AngleWrapped::new(2.0);
        assert_ne!(a, b);
    }

    #[test]
    fn equality_wrapping_zero_and_two_pi() {
        let a = AngleWrapped::new(0.0);
        let b = AngleWrapped::new(TWO_PI);
        assert_eq!(a, b);
    }

    // -- Ordering -------------------------------------------------------------

    #[test]
    fn ordering_less_than() {
        let a = AngleWrapped::new(1.0);
        let b = AngleWrapped::new(2.0);
        assert!(a < b);
        assert!(a <= b);
    }

    #[test]
    fn ordering_greater_than() {
        let a = AngleWrapped::new(3.0);
        let b = AngleWrapped::new(1.0);
        assert!(a > b);
        assert!(a >= b);
    }

    #[test]
    fn ordering_boundary() {
        let zero = AngleWrapped::new(0.0);
        let almost_two_pi = AngleWrapped::new(TWO_PI - 0.01);
        assert!(zero < almost_two_pi);
    }

    // -- AddAssign / SubAssign ------------------------------------------------

    #[test]
    fn add_assign() {
        let mut a = AngleWrapped::new(1.0);
        a += AngleWrapped::new(1.5);
        assert!(approx_eq(a.radians(), 2.5));
    }

    #[test]
    fn sub_assign() {
        let mut a = AngleWrapped::new(3.0);
        a -= AngleWrapped::new(1.0);
        assert!(approx_eq(a.radians(), 2.0));
    }
}
