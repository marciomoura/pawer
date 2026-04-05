//! 1-D lookup table with linear interpolation.
//!
//! Fixed-size, no-allocation implementation using const generics — suitable for
//! `no_std` embedded targets.

use crate::types::Real;

/// 1-D lookup table that performs linear interpolation between breakpoints.
///
/// `N` is the number of breakpoints and **must be ≥ 2** (debug-asserted).
/// The `x_axis` values must be provided in ascending order.
/// Queries outside the table range are clamped to the nearest endpoint.
pub struct LookupTable1D<const N: usize> {
    x_axis: [Real; N],
    y_values: [Real; N],
}

impl<const N: usize> LookupTable1D<N> {
    /// Create a new lookup table.
    ///
    /// # Debug assertions
    /// * `N >= 2`
    /// * `x_axis` is sorted in ascending order.
    pub fn new(x_axis: [Real; N], y_values: [Real; N]) -> Self {
        debug_assert!(N >= 2, "LookupTable1D requires at least 2 breakpoints");
        #[cfg(debug_assertions)]
        for i in 1..N {
            debug_assert!(
                x_axis[i] >= x_axis[i - 1],
                "x_axis must be sorted in ascending order"
            );
        }
        Self { x_axis, y_values }
    }

    /// Look up the interpolated value at `x`.
    ///
    /// Values outside the table range are clamped to the first / last entry.
    pub fn get_value(&self, x: Real) -> Real {
        let x_clamped = x.clamp(self.x_axis[0], self.x_axis[N - 1]);

        let idx = self.find_lower_index(x_clamped);

        let x0 = self.x_axis[idx];
        let x1 = self.x_axis[idx + 1];
        let y0 = self.y_values[idx];
        let y1 = self.y_values[idx + 1];

        let x_frac = if (x1 - x0) > 0.0 {
            (x_clamped - x0) / (x1 - x0)
        } else {
            0.0
        };

        y0 + x_frac * (y1 - y0)
    }

    /// Binary search for the index of the segment containing `x`.
    ///
    /// Returns `idx` such that `x_axis[idx] <= x < x_axis[idx + 1]`,
    /// clamped to `[0, N-2]`.
    fn find_lower_index(&self, x: Real) -> usize {        let mut low: usize = 0;
        let mut high: usize = N - 1;

        while low < high {
            let mid = (low + high) / 2;
            if self.x_axis[mid + 1] <= x {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        if low >= N - 1 { N - 2 } else { low }
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
        (a - b).abs() < EPSILON
    }

    // -- Simple linear table (y = 2*x) --------------------------------------

    #[test]
    fn linear_table_interpolation() {
        let lut = LookupTable1D::new([0.0, 1.0, 2.0, 3.0, 4.0], [0.0, 2.0, 4.0, 6.0, 8.0]);
        assert!(approx_eq(lut.get_value(1.5), 3.0));
    }

    #[test]
    fn linear_table_at_breakpoint() {
        let lut = LookupTable1D::new([0.0, 1.0, 2.0, 3.0, 4.0], [0.0, 2.0, 4.0, 6.0, 8.0]);
        assert!(approx_eq(lut.get_value(2.0), 4.0));
    }

    #[test]
    fn linear_table_at_first_breakpoint() {
        let lut = LookupTable1D::new([0.0, 1.0, 2.0, 3.0, 4.0], [0.0, 2.0, 4.0, 6.0, 8.0]);
        assert!(approx_eq(lut.get_value(0.0), 0.0));
    }

    #[test]
    fn linear_table_at_last_breakpoint() {
        let lut = LookupTable1D::new([0.0, 1.0, 2.0, 3.0, 4.0], [0.0, 2.0, 4.0, 6.0, 8.0]);
        assert!(approx_eq(lut.get_value(4.0), 8.0));
    }

    #[test]
    fn clamp_below_range() {
        let lut = LookupTable1D::new([0.0, 1.0, 2.0, 3.0, 4.0], [0.0, 2.0, 4.0, 6.0, 8.0]);
        assert!(approx_eq(lut.get_value(-1.0), 0.0));
    }

    #[test]
    fn clamp_above_range() {
        let lut = LookupTable1D::new([0.0, 1.0, 2.0, 3.0, 4.0], [0.0, 2.0, 4.0, 6.0, 8.0]);
        assert!(approx_eq(lut.get_value(10.0), 8.0));
    }

    // -- Non-linear table ----------------------------------------------------

    #[test]
    fn non_linear_table_interpolation() {
        let lut = LookupTable1D::new([0.0, 1.0, 2.0], [0.0, 1.0, 4.0]);
        assert!(approx_eq(lut.get_value(1.5), 2.5));
    }

    #[test]
    fn non_linear_table_first_segment() {
        let lut = LookupTable1D::new([0.0, 1.0, 2.0], [0.0, 1.0, 4.0]);
        assert!(approx_eq(lut.get_value(0.5), 0.5));
    }

    // -- Minimal table (N=2) -------------------------------------------------

    #[test]
    fn two_point_table_midpoint() {
        let lut = LookupTable1D::new([0.0, 1.0], [0.0, 10.0]);
        assert!(approx_eq(lut.get_value(0.5), 5.0));
    }

    #[test]
    fn two_point_table_quarter() {
        let lut = LookupTable1D::new([0.0, 1.0], [0.0, 10.0]);
        assert!(approx_eq(lut.get_value(0.25), 2.5));
    }

    #[test]
    fn two_point_table_clamp_below() {
        let lut = LookupTable1D::new([0.0, 1.0], [0.0, 10.0]);
        assert!(approx_eq(lut.get_value(-5.0), 0.0));
    }

    #[test]
    fn two_point_table_clamp_above() {
        let lut = LookupTable1D::new([0.0, 1.0], [0.0, 10.0]);
        assert!(approx_eq(lut.get_value(5.0), 10.0));
    }

    // -- Negative-valued table -----------------------------------------------

    #[test]
    fn negative_y_values() {
        let lut = LookupTable1D::new([0.0, 1.0, 2.0], [10.0, 0.0, -10.0]);
        assert!(approx_eq(lut.get_value(0.5), 5.0));
        assert!(approx_eq(lut.get_value(1.5), -5.0));
    }

    // -- Non-uniform spacing -------------------------------------------------

    #[test]
    fn non_uniform_spacing() {
        let lut = LookupTable1D::new([0.0, 1.0, 10.0], [0.0, 1.0, 10.0]);
        assert!(approx_eq(lut.get_value(5.5), 5.5));
    }
}
