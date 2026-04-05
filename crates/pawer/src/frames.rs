//! Coordinate frame types and transforms for three-phase power electronics.
//!
//! Provides [`Abc`] (three-phase), [`AlphaBeta`] (stationary αβ), and [`Dq`]
//! (rotating dq) frames with amplitude-invariant Clarke and Park transforms.

use core::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::angle::AngleWrapped;
use crate::constants::{SQRT_3, TWO_THIRDS, TWO_PI};
use crate::types::Real;

// ---------------------------------------------------------------------------
// Abc<T>
// ---------------------------------------------------------------------------

/// Three-phase (abc) coordinate frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Abc<T> {
    values: [T; 3],
}

// --- Constructors & accessors (generic) ------------------------------------

impl<T: Copy + Default> Abc<T> {
    pub fn new(a: T, b: T, c: T) -> Self {
        Self { values: [a, b, c] }
    }

    pub fn from_array(values: [T; 3]) -> Self {
        Self { values }
    }

    pub fn splat(value: T) -> Self {
        Self {
            values: [value, value, value],
        }
    }

    #[inline]
    pub fn a(&self) -> T {
        self.values[0]
    }
    #[inline]
    pub fn b(&self) -> T {
        self.values[1]
    }
    #[inline]
    pub fn c(&self) -> T {
        self.values[2]
    }

    #[inline]
    pub fn a_mut(&mut self) -> &mut T {
        &mut self.values[0]
    }
    #[inline]
    pub fn b_mut(&mut self) -> &mut T {
        &mut self.values[1]
    }
    #[inline]
    pub fn c_mut(&mut self) -> &mut T {
        &mut self.values[2]
    }

    #[inline]
    pub fn as_array(&self) -> &[T; 3] {
        &self.values
    }
}

impl<T: Copy + Default> Default for Abc<T> {
    fn default() -> Self {
        Self {
            values: [T::default(); 3],
        }
    }
}

// --- Indexing ---------------------------------------------------------------

impl<T> Index<usize> for Abc<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        &self.values[idx]
    }
}

impl<T> IndexMut<usize> for Abc<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.values[idx]
    }
}

// --- Element-wise arithmetic ------------------------------------------------

impl<T: Copy + Default + Add<Output = T>> Add for Abc<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            values: [
                self.values[0] + rhs.values[0],
                self.values[1] + rhs.values[1],
                self.values[2] + rhs.values[2],
            ],
        }
    }
}

impl<T: Copy + Default + Sub<Output = T>> Sub for Abc<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            values: [
                self.values[0] - rhs.values[0],
                self.values[1] - rhs.values[1],
                self.values[2] - rhs.values[2],
            ],
        }
    }
}

impl<T: Copy + Default + Add<Output = T>> AddAssign for Abc<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: Copy + Default + Sub<Output = T>> SubAssign for Abc<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: Copy + Default + Neg<Output = T>> Neg for Abc<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            values: [-self.values[0], -self.values[1], -self.values[2]],
        }
    }
}

// --- Scalar arithmetic ------------------------------------------------------

impl<T: Copy + Default + Mul<T, Output = T>> Mul<T> for Abc<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self {
        Self {
            values: [
                self.values[0] * scalar,
                self.values[1] * scalar,
                self.values[2] * scalar,
            ],
        }
    }
}

impl<T: Copy + Default + Div<T, Output = T>> Div<T> for Abc<T> {
    type Output = Self;
    fn div(self, scalar: T) -> Self {
        Self {
            values: [
                self.values[0] / scalar,
                self.values[1] / scalar,
                self.values[2] / scalar,
            ],
        }
    }
}

impl<T: Copy + Default + Mul<T, Output = T>> MulAssign<T> for Abc<T> {
    fn mul_assign(&mut self, scalar: T) {
        *self = *self * scalar;
    }
}

impl<T: Copy + Default + Div<T, Output = T>> DivAssign<T> for Abc<T> {
    fn div_assign(&mut self, scalar: T) {
        *self = *self / scalar;
    }
}

/// `scalar * Abc<f32>` — allows `2.0f32 * abc`.
impl Mul<Abc<f32>> for f32 {
    type Output = Abc<f32>;
    fn mul(self, rhs: Abc<f32>) -> Abc<f32> {
        rhs * self
    }
}

// --- Transforms (concrete Real) --------------------------------------------

impl Abc<Real> {
    /// Amplitude-invariant Clarke transform (abc → αβ).
    pub fn to_alphabeta(&self) -> AlphaBeta<Real> {
        let a = self.a();
        let b = self.b();
        let c = self.c();
        let alpha = TWO_THIRDS * (a - 0.5 * b - 0.5 * c);
        let beta = TWO_THIRDS * (SQRT_3 / 2.0) * (b - c);
        AlphaBeta::new(alpha, beta)
    }

    /// Combined Clarke + Park transform (abc → dq).
    pub fn to_dq(&self, theta: &AngleWrapped) -> Dq<Real> {
        self.to_alphabeta().to_dq(theta)
    }
}

// ---------------------------------------------------------------------------
// AlphaBeta<T>
// ---------------------------------------------------------------------------

/// Two-phase stationary (αβ) coordinate frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlphaBeta<T> {
    values: [T; 2],
}

// --- Constructors & accessors -----------------------------------------------

impl<T: Copy + Default> AlphaBeta<T> {
    pub fn new(alpha: T, beta: T) -> Self {
        Self {
            values: [alpha, beta],
        }
    }

    pub fn from_array(values: [T; 2]) -> Self {
        Self { values }
    }

    #[inline]
    pub fn alpha(&self) -> T {
        self.values[0]
    }
    #[inline]
    pub fn beta(&self) -> T {
        self.values[1]
    }

    #[inline]
    pub fn alpha_mut(&mut self) -> &mut T {
        &mut self.values[0]
    }
    #[inline]
    pub fn beta_mut(&mut self) -> &mut T {
        &mut self.values[1]
    }

    #[inline]
    pub fn as_array(&self) -> &[T; 2] {
        &self.values
    }
}

impl<T: Copy + Default> Default for AlphaBeta<T> {
    fn default() -> Self {
        Self {
            values: [T::default(); 2],
        }
    }
}

// --- Indexing ---------------------------------------------------------------

impl<T> Index<usize> for AlphaBeta<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        &self.values[idx]
    }
}

impl<T> IndexMut<usize> for AlphaBeta<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.values[idx]
    }
}

// --- Element-wise arithmetic ------------------------------------------------

impl<T: Copy + Default + Add<Output = T>> Add for AlphaBeta<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            values: [
                self.values[0] + rhs.values[0],
                self.values[1] + rhs.values[1],
            ],
        }
    }
}

impl<T: Copy + Default + Sub<Output = T>> Sub for AlphaBeta<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            values: [
                self.values[0] - rhs.values[0],
                self.values[1] - rhs.values[1],
            ],
        }
    }
}

impl<T: Copy + Default + Add<Output = T>> AddAssign for AlphaBeta<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: Copy + Default + Sub<Output = T>> SubAssign for AlphaBeta<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: Copy + Default + Neg<Output = T>> Neg for AlphaBeta<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            values: [-self.values[0], -self.values[1]],
        }
    }
}

// --- Scalar arithmetic ------------------------------------------------------

impl<T: Copy + Default + Mul<T, Output = T>> Mul<T> for AlphaBeta<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self {
        Self {
            values: [self.values[0] * scalar, self.values[1] * scalar],
        }
    }
}

impl<T: Copy + Default + Div<T, Output = T>> Div<T> for AlphaBeta<T> {
    type Output = Self;
    fn div(self, scalar: T) -> Self {
        Self {
            values: [self.values[0] / scalar, self.values[1] / scalar],
        }
    }
}

impl<T: Copy + Default + Mul<T, Output = T>> MulAssign<T> for AlphaBeta<T> {
    fn mul_assign(&mut self, scalar: T) {
        *self = *self * scalar;
    }
}

impl<T: Copy + Default + Div<T, Output = T>> DivAssign<T> for AlphaBeta<T> {
    fn div_assign(&mut self, scalar: T) {
        *self = *self / scalar;
    }
}

impl Mul<AlphaBeta<f32>> for f32 {
    type Output = AlphaBeta<f32>;
    fn mul(self, rhs: AlphaBeta<f32>) -> AlphaBeta<f32> {
        rhs * self
    }
}

// --- Transforms & helpers (concrete Real) -----------------------------------

impl AlphaBeta<Real> {
    /// Inverse Clarke transform (αβ → abc).
    pub fn to_abc(&self) -> Abc<Real> {
        let alpha = self.alpha();
        let beta = self.beta();
        let half_sqrt3 = SQRT_3 / 2.0;
        Abc::new(
            alpha,
            -0.5 * alpha + half_sqrt3 * beta,
            -0.5 * alpha - half_sqrt3 * beta,
        )
    }

    /// Park transform (αβ → dq).
    pub fn to_dq(&self, theta: &AngleWrapped) -> Dq<Real> {
        let cos = libm::cosf(theta.radians());
        let sin = libm::sinf(theta.radians());
        let alpha = self.alpha();
        let beta = self.beta();
        Dq::new(
            cos * alpha + sin * beta,
            -sin * alpha + cos * beta,
        )
    }

    /// Vector magnitude √(α² + β²).
    pub fn magnitude(&self) -> Real {
        libm::sqrtf(self.alpha() * self.alpha() + self.beta() * self.beta())
    }

    /// Phase angle atan2(β, α) wrapped to [0, 2π).
    pub fn phase(&self) -> AngleWrapped {
        AngleWrapped::new(libm::atan2f(self.beta(), self.alpha()))
    }

    /// Counter-clockwise rotation by `theta`.
    pub fn rotate(&self, theta: &AngleWrapped) -> AlphaBeta<Real> {
        let cos = libm::cosf(theta.radians());
        let sin = libm::sinf(theta.radians());
        let alpha = self.alpha();
        let beta = self.beta();
        AlphaBeta::new(
            alpha * cos - beta * sin,
            alpha * sin + beta * cos,
        )
    }
}

// ---------------------------------------------------------------------------
// Dq<T>
// ---------------------------------------------------------------------------

/// Rotating (dq) reference frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dq<T> {
    values: [T; 2],
}

// --- Constructors & accessors -----------------------------------------------

impl<T: Copy + Default> Dq<T> {
    pub fn new(d: T, q: T) -> Self {
        Self { values: [d, q] }
    }

    pub fn from_array(values: [T; 2]) -> Self {
        Self { values }
    }

    #[inline]
    pub fn d(&self) -> T {
        self.values[0]
    }
    #[inline]
    pub fn q(&self) -> T {
        self.values[1]
    }

    #[inline]
    pub fn d_mut(&mut self) -> &mut T {
        &mut self.values[0]
    }
    #[inline]
    pub fn q_mut(&mut self) -> &mut T {
        &mut self.values[1]
    }

    #[inline]
    pub fn as_array(&self) -> &[T; 2] {
        &self.values
    }
}

impl<T: Copy + Default> Default for Dq<T> {
    fn default() -> Self {
        Self {
            values: [T::default(); 2],
        }
    }
}

// --- Indexing ---------------------------------------------------------------

impl<T> Index<usize> for Dq<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        &self.values[idx]
    }
}

impl<T> IndexMut<usize> for Dq<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.values[idx]
    }
}

// --- Element-wise arithmetic ------------------------------------------------

impl<T: Copy + Default + Add<Output = T>> Add for Dq<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            values: [
                self.values[0] + rhs.values[0],
                self.values[1] + rhs.values[1],
            ],
        }
    }
}

impl<T: Copy + Default + Sub<Output = T>> Sub for Dq<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            values: [
                self.values[0] - rhs.values[0],
                self.values[1] - rhs.values[1],
            ],
        }
    }
}

impl<T: Copy + Default + Add<Output = T>> AddAssign for Dq<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: Copy + Default + Sub<Output = T>> SubAssign for Dq<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: Copy + Default + Neg<Output = T>> Neg for Dq<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            values: [-self.values[0], -self.values[1]],
        }
    }
}

// --- Scalar arithmetic ------------------------------------------------------

impl<T: Copy + Default + Mul<T, Output = T>> Mul<T> for Dq<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self {
        Self {
            values: [self.values[0] * scalar, self.values[1] * scalar],
        }
    }
}

impl<T: Copy + Default + Div<T, Output = T>> Div<T> for Dq<T> {
    type Output = Self;
    fn div(self, scalar: T) -> Self {
        Self {
            values: [self.values[0] / scalar, self.values[1] / scalar],
        }
    }
}

impl<T: Copy + Default + Mul<T, Output = T>> MulAssign<T> for Dq<T> {
    fn mul_assign(&mut self, scalar: T) {
        *self = *self * scalar;
    }
}

impl<T: Copy + Default + Div<T, Output = T>> DivAssign<T> for Dq<T> {
    fn div_assign(&mut self, scalar: T) {
        *self = *self / scalar;
    }
}

impl Mul<Dq<f32>> for f32 {
    type Output = Dq<f32>;
    fn mul(self, rhs: Dq<f32>) -> Dq<f32> {
        rhs * self
    }
}

// --- Transforms & helpers (concrete Real) -----------------------------------

impl Dq<Real> {
    /// Inverse Park transform (dq → αβ).
    pub fn to_alphabeta(&self, theta: &AngleWrapped) -> AlphaBeta<Real> {
        let cos = libm::cosf(theta.radians());
        let sin = libm::sinf(theta.radians());
        let d = self.d();
        let q = self.q();
        AlphaBeta::new(
            cos * d - sin * q,
            sin * d + cos * q,
        )
    }

    /// Combined inverse Park + inverse Clarke (dq → abc).
    pub fn to_abc(&self, theta: &AngleWrapped) -> Abc<Real> {
        self.to_alphabeta(theta).to_abc()
    }

    /// Vector magnitude √(d² + q²).
    pub fn magnitude(&self) -> Real {
        libm::sqrtf(self.d() * self.d() + self.q() * self.q())
    }

    /// Absolute phase angle: frame_angle + atan2(q, d).
    pub fn phase(&self, frame_angle: &AngleWrapped) -> AngleWrapped {
        *frame_angle + libm::atan2f(self.q(), self.d())
    }

    /// Counter-clockwise rotation by `theta`.
    pub fn rotate(&self, theta: &AngleWrapped) -> Dq<Real> {
        let cos = libm::cosf(theta.radians());
        let sin = libm::sinf(theta.radians());
        let d = self.d();
        let q = self.q();
        Dq::new(
            d * cos - q * sin,
            d * sin + q * cos,
        )
    }
}

// ---------------------------------------------------------------------------
// Factory functions
// ---------------------------------------------------------------------------

/// Create a balanced three-phase vector from magnitude and phase angle.
pub fn make_abc(magnitude: Real, phase: &AngleWrapped) -> Abc<Real> {
    let angle = phase.radians();
    Abc::new(
        magnitude * libm::cosf(angle),
        magnitude * libm::cosf(angle - TWO_PI / 3.0),
        magnitude * libm::cosf(angle + TWO_PI / 3.0),
    )
}

/// Create an αβ vector from magnitude and phase angle.
pub fn make_alphabeta(magnitude: Real, phase: &AngleWrapped) -> AlphaBeta<Real> {
    let angle = phase.radians();
    AlphaBeta::new(
        magnitude * libm::cosf(angle),
        magnitude * libm::sinf(angle),
    )
}

// ---------------------------------------------------------------------------
// Cross products
// ---------------------------------------------------------------------------

/// 2-D cross product of two αβ vectors (scalar result).
pub fn cross_product_alphabeta(a: &AlphaBeta<Real>, b: &AlphaBeta<Real>) -> Real {
    a.alpha() * b.beta() - a.beta() * b.alpha()
}

/// 2-D cross product of two dq vectors (scalar result).
pub fn cross_product_dq(a: &Dq<Real>, b: &Dq<Real>) -> Real {
    a.d() * b.q() - a.q() * b.d()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::PI;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    // ---- Abc tests --------------------------------------------------------

    #[test]
    fn abc_default_is_zero() {
        let abc: Abc<f32> = Abc::default();
        assert!(approx_eq(abc.a(), 0.0));
        assert!(approx_eq(abc.b(), 0.0));
        assert!(approx_eq(abc.c(), 0.0));
    }

    #[test]
    fn abc_new_values() {
        let abc = Abc::new(1.0_f32, -0.5, -0.5);
        assert!(approx_eq(abc.a(), 1.0));
        assert!(approx_eq(abc.b(), -0.5));
        assert!(approx_eq(abc.c(), -0.5));
    }

    #[test]
    fn abc_array_access() {
        let abc = Abc::new(1.0_f32, 2.0, 3.0);
        assert!(approx_eq(abc[0], 1.0));
        assert!(approx_eq(abc[1], 2.0));
        assert!(approx_eq(abc[2], 3.0));
    }

    #[test]
    fn abc_to_alphabeta_balanced() {
        let abc = Abc::new(1.0_f32, -0.5, -0.5);
        let ab = abc.to_alphabeta();
        assert!(approx_eq(ab.alpha(), 1.0));
        assert!(approx_eq(ab.beta(), 0.0));
    }

    #[test]
    fn abc_addition() {
        let a = Abc::new(1.0_f32, 2.0, 3.0);
        let b = Abc::new(4.0_f32, 5.0, 6.0);
        let c = a + b;
        assert!(approx_eq(c.a(), 5.0));
        assert!(approx_eq(c.b(), 7.0));
        assert!(approx_eq(c.c(), 9.0));
    }

    #[test]
    fn abc_subtraction() {
        let a = Abc::new(4.0_f32, 5.0, 6.0);
        let b = Abc::new(1.0_f32, 2.0, 3.0);
        let c = a - b;
        assert!(approx_eq(c.a(), 3.0));
        assert!(approx_eq(c.b(), 3.0));
        assert!(approx_eq(c.c(), 3.0));
    }

    #[test]
    fn abc_negation() {
        let abc = Abc::new(1.0_f32, -0.5, -0.5);
        let neg = -abc;
        assert!(approx_eq(neg.a(), -1.0));
        assert!(approx_eq(neg.b(), 0.5));
        assert!(approx_eq(neg.c(), 0.5));
    }

    #[test]
    fn abc_add_assign() {
        let mut a = Abc::new(1.0_f32, 2.0, 3.0);
        a += Abc::new(0.5, 0.5, 0.5);
        assert!(approx_eq(a.a(), 1.5));
        assert!(approx_eq(a.b(), 2.5));
        assert!(approx_eq(a.c(), 3.5));
    }

    #[test]
    fn abc_sub_assign() {
        let mut a = Abc::new(1.0_f32, 2.0, 3.0);
        a -= Abc::new(0.5, 0.5, 0.5);
        assert!(approx_eq(a.a(), 0.5));
        assert!(approx_eq(a.b(), 1.5));
        assert!(approx_eq(a.c(), 2.5));
    }

    #[test]
    fn abc_scalar_multiply() {
        let abc = Abc::new(1.0_f32, 2.0, 3.0);
        let scaled = abc * 2.0;
        assert!(approx_eq(scaled.a(), 2.0));
        assert!(approx_eq(scaled.b(), 4.0));
        assert!(approx_eq(scaled.c(), 6.0));
    }

    #[test]
    fn abc_scalar_multiply_lhs() {
        let abc = Abc::new(1.0_f32, 2.0, 3.0);
        let scaled = 2.0_f32 * abc;
        assert!(approx_eq(scaled.a(), 2.0));
        assert!(approx_eq(scaled.b(), 4.0));
        assert!(approx_eq(scaled.c(), 6.0));
    }

    #[test]
    fn abc_scalar_divide() {
        let abc = Abc::new(2.0_f32, 4.0, 6.0);
        let scaled = abc / 2.0;
        assert!(approx_eq(scaled.a(), 1.0));
        assert!(approx_eq(scaled.b(), 2.0));
        assert!(approx_eq(scaled.c(), 3.0));
    }

    #[test]
    fn abc_mul_assign() {
        let mut abc = Abc::new(1.0_f32, 2.0, 3.0);
        abc *= 3.0;
        assert!(approx_eq(abc.a(), 3.0));
        assert!(approx_eq(abc.b(), 6.0));
        assert!(approx_eq(abc.c(), 9.0));
    }

    #[test]
    fn abc_div_assign() {
        let mut abc = Abc::new(3.0_f32, 6.0, 9.0);
        abc /= 3.0;
        assert!(approx_eq(abc.a(), 1.0));
        assert!(approx_eq(abc.b(), 2.0));
        assert!(approx_eq(abc.c(), 3.0));
    }

    #[test]
    fn abc_splat() {
        let abc = Abc::splat(5.0_f32);
        assert!(approx_eq(abc.a(), 5.0));
        assert!(approx_eq(abc.b(), 5.0));
        assert!(approx_eq(abc.c(), 5.0));
    }

    #[test]
    fn abc_mutable_accessors() {
        let mut abc = Abc::new(1.0_f32, 2.0, 3.0);
        *abc.a_mut() = 10.0;
        *abc.b_mut() = 20.0;
        *abc.c_mut() = 30.0;
        assert!(approx_eq(abc.a(), 10.0));
        assert!(approx_eq(abc.b(), 20.0));
        assert!(approx_eq(abc.c(), 30.0));
    }

    // ---- AlphaBeta tests --------------------------------------------------

    #[test]
    fn alphabeta_default_is_zero() {
        let ab: AlphaBeta<f32> = AlphaBeta::default();
        assert!(approx_eq(ab.alpha(), 0.0));
        assert!(approx_eq(ab.beta(), 0.0));
    }

    #[test]
    fn alphabeta_new_values() {
        let ab = AlphaBeta::new(1.0_f32, 2.0);
        assert!(approx_eq(ab.alpha(), 1.0));
        assert!(approx_eq(ab.beta(), 2.0));
    }

    #[test]
    fn alphabeta_to_abc_inverse_of_clarke() {
        let abc_orig = Abc::new(1.0_f32, -0.5, -0.5);
        let ab = abc_orig.to_alphabeta();
        let abc_recovered = ab.to_abc();
        assert!(approx_eq(abc_recovered.a(), abc_orig.a()));
        assert!(approx_eq(abc_recovered.b(), abc_orig.b()));
        assert!(approx_eq(abc_recovered.c(), abc_orig.c()));
    }

    #[test]
    fn alphabeta_to_dq_at_zero_angle() {
        let ab = AlphaBeta::new(1.0_f32, 2.0);
        let theta = AngleWrapped::new(0.0);
        let dq = ab.to_dq(&theta);
        assert!(approx_eq(dq.d(), 1.0));
        assert!(approx_eq(dq.q(), 2.0));
    }

    #[test]
    fn alphabeta_magnitude() {
        let ab = AlphaBeta::new(1.0_f32, 1.0);
        let mag = ab.magnitude();
        assert!(approx_eq(mag, libm::sqrtf(2.0)));
    }

    #[test]
    fn alphabeta_phase() {
        let ab = AlphaBeta::new(1.0_f32, 1.0);
        let phase = ab.phase();
        assert!(approx_eq(phase.radians(), PI / 4.0));
    }

    #[test]
    fn alphabeta_addition() {
        let a = AlphaBeta::new(1.0_f32, 2.0);
        let b = AlphaBeta::new(3.0_f32, 4.0);
        let c = a + b;
        assert!(approx_eq(c.alpha(), 4.0));
        assert!(approx_eq(c.beta(), 6.0));
    }

    #[test]
    fn alphabeta_subtraction() {
        let a = AlphaBeta::new(3.0_f32, 4.0);
        let b = AlphaBeta::new(1.0_f32, 2.0);
        let c = a - b;
        assert!(approx_eq(c.alpha(), 2.0));
        assert!(approx_eq(c.beta(), 2.0));
    }

    #[test]
    fn alphabeta_negation() {
        let ab = AlphaBeta::new(1.0_f32, -2.0);
        let neg = -ab;
        assert!(approx_eq(neg.alpha(), -1.0));
        assert!(approx_eq(neg.beta(), 2.0));
    }

    #[test]
    fn alphabeta_add_assign() {
        let mut a = AlphaBeta::new(1.0_f32, 2.0);
        a += AlphaBeta::new(0.5, 0.5);
        assert!(approx_eq(a.alpha(), 1.5));
        assert!(approx_eq(a.beta(), 2.5));
    }

    #[test]
    fn alphabeta_sub_assign() {
        let mut a = AlphaBeta::new(1.0_f32, 2.0);
        a -= AlphaBeta::new(0.5, 0.5);
        assert!(approx_eq(a.alpha(), 0.5));
        assert!(approx_eq(a.beta(), 1.5));
    }

    #[test]
    fn alphabeta_scalar_multiply() {
        let ab = AlphaBeta::new(1.0_f32, 2.0);
        let scaled = ab * 3.0;
        assert!(approx_eq(scaled.alpha(), 3.0));
        assert!(approx_eq(scaled.beta(), 6.0));
    }

    #[test]
    fn alphabeta_scalar_multiply_lhs() {
        let ab = AlphaBeta::new(1.0_f32, 2.0);
        let scaled = 3.0_f32 * ab;
        assert!(approx_eq(scaled.alpha(), 3.0));
        assert!(approx_eq(scaled.beta(), 6.0));
    }

    #[test]
    fn alphabeta_scalar_divide() {
        let ab = AlphaBeta::new(3.0_f32, 6.0);
        let scaled = ab / 3.0;
        assert!(approx_eq(scaled.alpha(), 1.0));
        assert!(approx_eq(scaled.beta(), 2.0));
    }

    #[test]
    fn alphabeta_mul_assign() {
        let mut ab = AlphaBeta::new(1.0_f32, 2.0);
        ab *= 2.0;
        assert!(approx_eq(ab.alpha(), 2.0));
        assert!(approx_eq(ab.beta(), 4.0));
    }

    #[test]
    fn alphabeta_div_assign() {
        let mut ab = AlphaBeta::new(4.0_f32, 6.0);
        ab /= 2.0;
        assert!(approx_eq(ab.alpha(), 2.0));
        assert!(approx_eq(ab.beta(), 3.0));
    }

    // ---- Dq tests ---------------------------------------------------------

    #[test]
    fn dq_default_is_zero() {
        let dq: Dq<f32> = Dq::default();
        assert!(approx_eq(dq.d(), 0.0));
        assert!(approx_eq(dq.q(), 0.0));
    }

    #[test]
    fn dq_to_alphabeta_at_zero_angle() {
        let dq = Dq::new(1.0_f32, 2.0);
        let theta = AngleWrapped::new(0.0);
        let ab = dq.to_alphabeta(&theta);
        assert!(approx_eq(ab.alpha(), 1.0));
        assert!(approx_eq(ab.beta(), 2.0));
    }

    #[test]
    fn dq_magnitude() {
        let dq = Dq::new(3.0_f32, 4.0);
        assert!(approx_eq(dq.magnitude(), 5.0));
    }

    #[test]
    fn dq_addition() {
        let a = Dq::new(1.0_f32, 2.0);
        let b = Dq::new(3.0_f32, 4.0);
        let c = a + b;
        assert!(approx_eq(c.d(), 4.0));
        assert!(approx_eq(c.q(), 6.0));
    }

    #[test]
    fn dq_subtraction() {
        let a = Dq::new(3.0_f32, 4.0);
        let b = Dq::new(1.0_f32, 2.0);
        let c = a - b;
        assert!(approx_eq(c.d(), 2.0));
        assert!(approx_eq(c.q(), 2.0));
    }

    #[test]
    fn dq_negation() {
        let dq = Dq::new(1.0_f32, -2.0);
        let neg = -dq;
        assert!(approx_eq(neg.d(), -1.0));
        assert!(approx_eq(neg.q(), 2.0));
    }

    #[test]
    fn dq_add_assign() {
        let mut a = Dq::new(1.0_f32, 2.0);
        a += Dq::new(0.5, 0.5);
        assert!(approx_eq(a.d(), 1.5));
        assert!(approx_eq(a.q(), 2.5));
    }

    #[test]
    fn dq_sub_assign() {
        let mut a = Dq::new(1.0_f32, 2.0);
        a -= Dq::new(0.5, 0.5);
        assert!(approx_eq(a.d(), 0.5));
        assert!(approx_eq(a.q(), 1.5));
    }

    #[test]
    fn dq_scalar_multiply() {
        let dq = Dq::new(1.0_f32, 2.0);
        let scaled = dq * 3.0;
        assert!(approx_eq(scaled.d(), 3.0));
        assert!(approx_eq(scaled.q(), 6.0));
    }

    #[test]
    fn dq_scalar_multiply_lhs() {
        let dq = Dq::new(1.0_f32, 2.0);
        let scaled = 3.0_f32 * dq;
        assert!(approx_eq(scaled.d(), 3.0));
        assert!(approx_eq(scaled.q(), 6.0));
    }

    #[test]
    fn dq_scalar_divide() {
        let dq = Dq::new(3.0_f32, 6.0);
        let scaled = dq / 3.0;
        assert!(approx_eq(scaled.d(), 1.0));
        assert!(approx_eq(scaled.q(), 2.0));
    }

    #[test]
    fn dq_mul_assign() {
        let mut dq = Dq::new(1.0_f32, 2.0);
        dq *= 2.0;
        assert!(approx_eq(dq.d(), 2.0));
        assert!(approx_eq(dq.q(), 4.0));
    }

    #[test]
    fn dq_div_assign() {
        let mut dq = Dq::new(4.0_f32, 6.0);
        dq /= 2.0;
        assert!(approx_eq(dq.d(), 2.0));
        assert!(approx_eq(dq.q(), 3.0));
    }

    // ---- Round-trip tests -------------------------------------------------

    #[test]
    fn roundtrip_abc_alphabeta_abc() {
        let abc_orig = Abc::new(1.0_f32, -0.5, -0.5);
        let abc_back = abc_orig.to_alphabeta().to_abc();
        assert!(approx_eq(abc_back.a(), abc_orig.a()));
        assert!(approx_eq(abc_back.b(), abc_orig.b()));
        assert!(approx_eq(abc_back.c(), abc_orig.c()));
    }

    #[test]
    fn roundtrip_alphabeta_dq_alphabeta_at_zero() {
        let ab_orig = AlphaBeta::new(1.0_f32, 2.0);
        let theta = AngleWrapped::new(0.0);
        let ab_back = ab_orig.to_dq(&theta).to_alphabeta(&theta);
        assert!(approx_eq(ab_back.alpha(), ab_orig.alpha()));
        assert!(approx_eq(ab_back.beta(), ab_orig.beta()));
    }

    #[test]
    fn roundtrip_alphabeta_dq_alphabeta_at_pi_over_4() {
        let ab_orig = AlphaBeta::new(1.0_f32, 2.0);
        let theta = AngleWrapped::new(PI / 4.0);
        let ab_back = ab_orig.to_dq(&theta).to_alphabeta(&theta);
        assert!(approx_eq(ab_back.alpha(), ab_orig.alpha()));
        assert!(approx_eq(ab_back.beta(), ab_orig.beta()));
    }

    #[test]
    fn roundtrip_abc_dq_abc_at_various_angles() {
        let abc_orig = Abc::new(1.0_f32, -0.5, -0.5);
        for &angle in &[0.0, PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0, PI] {
            let theta = AngleWrapped::new(angle);
            let abc_back = abc_orig.to_dq(&theta).to_abc(&theta);
            assert!(
                approx_eq(abc_back.a(), abc_orig.a()),
                "a mismatch at angle {}",
                angle
            );
            assert!(
                approx_eq(abc_back.b(), abc_orig.b()),
                "b mismatch at angle {}",
                angle
            );
            assert!(
                approx_eq(abc_back.c(), abc_orig.c()),
                "c mismatch at angle {}",
                angle
            );
        }
    }

    // ---- Factory function tests -------------------------------------------

    #[test]
    fn make_abc_at_zero_phase() {
        let abc = make_abc(1.0, &AngleWrapped::new(0.0));
        assert!(approx_eq(abc.a(), 1.0));
        assert!(approx_eq(abc.b(), -0.5));
        assert!(approx_eq(abc.c(), -0.5));
    }

    #[test]
    fn make_alphabeta_at_zero_phase() {
        let ab = make_alphabeta(1.0, &AngleWrapped::new(0.0));
        assert!(approx_eq(ab.alpha(), 1.0));
        assert!(approx_eq(ab.beta(), 0.0));
    }

    #[test]
    fn make_alphabeta_at_pi_over_2() {
        let ab = make_alphabeta(1.0, &AngleWrapped::new(PI / 2.0));
        assert!(approx_eq(ab.alpha(), 0.0));
        assert!(approx_eq(ab.beta(), 1.0));
    }

    // ---- Cross product tests ----------------------------------------------

    #[test]
    fn cross_product_alphabeta_test() {
        let a = AlphaBeta::new(1.0_f32, 0.0);
        let b = AlphaBeta::new(0.0_f32, 1.0);
        assert!(approx_eq(cross_product_alphabeta(&a, &b), 1.0));
        assert!(approx_eq(cross_product_alphabeta(&b, &a), -1.0));
    }

    #[test]
    fn cross_product_dq_test() {
        let a = Dq::new(1.0_f32, 0.0);
        let b = Dq::new(0.0_f32, 1.0);
        assert!(approx_eq(cross_product_dq(&a, &b), 1.0));
        assert!(approx_eq(cross_product_dq(&b, &a), -1.0));
    }

    // ---- Rotate tests -----------------------------------------------------

    #[test]
    fn alphabeta_rotate_by_zero() {
        let ab = AlphaBeta::new(1.0_f32, 0.0);
        let rotated = ab.rotate(&AngleWrapped::new(0.0));
        assert!(approx_eq(rotated.alpha(), 1.0));
        assert!(approx_eq(rotated.beta(), 0.0));
    }

    #[test]
    fn alphabeta_rotate_by_pi_over_2() {
        let ab = AlphaBeta::new(1.0_f32, 0.0);
        let rotated = ab.rotate(&AngleWrapped::new(PI / 2.0));
        assert!(approx_eq(rotated.alpha(), 0.0));
        assert!(approx_eq(rotated.beta(), 1.0));
    }

    #[test]
    fn dq_rotate_by_zero() {
        let dq = Dq::new(1.0_f32, 0.0);
        let rotated = dq.rotate(&AngleWrapped::new(0.0));
        assert!(approx_eq(rotated.d(), 1.0));
        assert!(approx_eq(rotated.q(), 0.0));
    }

    #[test]
    fn dq_rotate_by_pi_over_2() {
        let dq = Dq::new(1.0_f32, 0.0);
        let rotated = dq.rotate(&AngleWrapped::new(PI / 2.0));
        assert!(approx_eq(rotated.d(), 0.0));
        assert!(approx_eq(rotated.q(), 1.0));
    }
}
