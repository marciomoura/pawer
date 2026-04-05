//! 2-D lookup table with bilinear interpolation.
//!
//! Ported from C++ lookup-table utilities. Fixed-size, no-allocation
//! implementation using const generics — suitable for `no_std` embedded targets.

use crate::types::Real;

/// 2-D lookup table that performs bilinear interpolation.
///
/// * `NX` — number of breakpoints along the X axis (must be ≥ 2).
/// * `NY` — number of breakpoints along the Y axis (must be ≥ 2).
///
/// `z_values` is laid out as `z_values[y_idx][x_idx]`.
/// Queries outside the table range are clamped to the nearest edge.
pub struct LookupTable2D<const NX: usize, const NY: usize> {
    x_axis: [Real; NX],
    y_axis: [Real; NY],
    z_values: [[Real; NX]; NY],
}

impl<const NX: usize, const NY: usize> LookupTable2D<NX, NY> {
    /// Create a new 2-D lookup table.
    ///
    /// # Debug assertions
    /// * `NX >= 2` and `NY >= 2`.
    /// * Both axes are sorted in ascending order.
    pub fn new(x_axis: [Real; NX], y_axis: [Real; NY], z_values: [[Real; NX]; NY]) -> Self {
        debug_assert!(NX >= 2, "LookupTable2D requires at least 2 X breakpoints");
        debug_assert!(NY >= 2, "LookupTable2D requires at least 2 Y breakpoints");
        #[cfg(debug_assertions)]
        {
            for i in 1..NX {
                debug_assert!(
                    x_axis[i] >= x_axis[i - 1],
                    "x_axis must be sorted in ascending order"
                );
            }
            for i in 1..NY {
                debug_assert!(
                    y_axis[i] >= y_axis[i - 1],
                    "y_axis must be sorted in ascending order"
                );
            }
        }
        Self {
            x_axis,
            y_axis,
            z_values,
        }
    }

    /// Look up the bilinearly interpolated value at `(x, y)`.
    ///
    /// Coordinates outside the table range are clamped to the edges.
    pub fn get_value(&self, x: Real, y: Real) -> Real {
        let x_c = clamp(x, self.x_axis[0], self.x_axis[NX - 1]);
        let y_c = clamp(y, self.y_axis[0], self.y_axis[NY - 1]);

        let xi = self.find_lower_index_x(x_c);
        let yi = self.find_lower_index_y(y_c);

        let x0 = self.x_axis[xi];
        let x1 = self.x_axis[xi + 1];
        let y0 = self.y_axis[yi];
        let y1 = self.y_axis[yi + 1];

        let z00 = self.z_values[yi][xi];
        let z10 = self.z_values[yi][xi + 1];
        let z01 = self.z_values[yi + 1][xi];
        let z11 = self.z_values[yi + 1][xi + 1];

        let xf = if (x1 - x0) > 0.0 {
            (x_c - x0) / (x1 - x0)
        } else {
            0.0
        };
        let yf = if (y1 - y0) > 0.0 {
            (y_c - y0) / (y1 - y0)
        } else {
            0.0
        };

        // Bilinear: interpolate along X for both Y rows, then along Y.
        let z_y0 = z00 + xf * (z10 - z00);
        let z_y1 = z01 + xf * (z11 - z01);
        z_y0 + yf * (z_y1 - z_y0)
    }

    /// Binary search on the X axis.
    fn find_lower_index_x(&self, x: Real) -> usize {
        let mut low: usize = 0;
        let mut high: usize = NX - 1;
        while low < high {
            let mid = (low + high) / 2;
            if self.x_axis[mid + 1] <= x {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        if low >= NX - 1 { NX - 2 } else { low }
    }

    /// Binary search on the Y axis.
    fn find_lower_index_y(&self, y: Real) -> usize {
        let mut low: usize = 0;
        let mut high: usize = NY - 1;
        while low < high {
            let mid = (low + high) / 2;
            if self.y_axis[mid + 1] <= y {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        if low >= NY - 1 { NY - 2 } else { low }
    }
}

#[inline]
fn clamp(v: Real, min: Real, max: Real) -> Real {
    if v < min {
        min
    } else if v > max {
        max
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

    // -- Flat table (all z = constant) ---------------------------------------

    #[test]
    fn flat_table_center() {
        let lut = LookupTable2D::new([0.0, 1.0], [0.0, 1.0], [[5.0, 5.0], [5.0, 5.0]]);
        assert!(approx_eq(lut.get_value(0.5, 0.5), 5.0));
    }

    #[test]
    fn flat_table_corner() {
        let lut = LookupTable2D::new([0.0, 1.0], [0.0, 1.0], [[5.0, 5.0], [5.0, 5.0]]);
        assert!(approx_eq(lut.get_value(0.0, 0.0), 5.0));
    }

    #[test]
    fn flat_table_outside() {
        let lut = LookupTable2D::new([0.0, 1.0], [0.0, 1.0], [[5.0, 5.0], [5.0, 5.0]]);
        assert!(approx_eq(lut.get_value(10.0, -10.0), 5.0));
    }

    // -- Linear gradient: z = x + y on a 3×3 grid ---------------------------

    fn linear_gradient_table() -> LookupTable2D<3, 3> {
        // x = [0, 1, 2], y = [0, 1, 2]
        // z[yi][xi] = x + y
        LookupTable2D::new(
            [0.0, 1.0, 2.0],
            [0.0, 1.0, 2.0],
            [
                [0.0, 1.0, 2.0], // y=0
                [1.0, 2.0, 3.0], // y=1
                [2.0, 3.0, 4.0], // y=2
            ],
        )
    }

    #[test]
    fn linear_gradient_at_breakpoints() {
        let lut = linear_gradient_table();
        assert!(approx_eq(lut.get_value(0.0, 0.0), 0.0));
        assert!(approx_eq(lut.get_value(1.0, 1.0), 2.0));
        assert!(approx_eq(lut.get_value(2.0, 2.0), 4.0));
        assert!(approx_eq(lut.get_value(2.0, 0.0), 2.0));
        assert!(approx_eq(lut.get_value(0.0, 2.0), 2.0));
    }

    #[test]
    fn linear_gradient_interior_point() {
        let lut = linear_gradient_table();
        // z(0.5, 0.5) should be 0.5 + 0.5 = 1.0
        assert!(approx_eq(lut.get_value(0.5, 0.5), 1.0));
    }

    #[test]
    fn linear_gradient_another_interior() {
        let lut = linear_gradient_table();
        // z(1.5, 0.5) should be 1.5 + 0.5 = 2.0
        assert!(approx_eq(lut.get_value(1.5, 0.5), 2.0));
    }

    #[test]
    fn linear_gradient_on_edge() {
        let lut = linear_gradient_table();
        // z(0.5, 0.0) should be 0.5
        assert!(approx_eq(lut.get_value(0.5, 0.0), 0.5));
    }

    // -- Clamping at corners -------------------------------------------------

    #[test]
    fn clamp_below_x_below_y() {
        let lut = linear_gradient_table();
        // Should clamp to (0,0) → 0.0
        assert!(approx_eq(lut.get_value(-5.0, -5.0), 0.0));
    }

    #[test]
    fn clamp_above_x_above_y() {
        let lut = linear_gradient_table();
        // Should clamp to (2,2) → 4.0
        assert!(approx_eq(lut.get_value(10.0, 10.0), 4.0));
    }

    #[test]
    fn clamp_below_x_above_y() {
        let lut = linear_gradient_table();
        // Should clamp to (0,2) → 2.0
        assert!(approx_eq(lut.get_value(-1.0, 5.0), 2.0));
    }

    #[test]
    fn clamp_above_x_below_y() {
        let lut = linear_gradient_table();
        // Should clamp to (2,0) → 2.0
        assert!(approx_eq(lut.get_value(5.0, -1.0), 2.0));
    }

    // -- Non-linear z surface ------------------------------------------------

    #[test]
    fn nonlinear_surface_bilinear_interpolation() {
        // z = x * y on a 2×2 grid: corners at (0,0), (2,0), (0,3), (2,3)
        let lut = LookupTable2D::new(
            [0.0, 2.0],
            [0.0, 3.0],
            [
                [0.0, 0.0], // y=0: 0*0, 2*0
                [0.0, 6.0], // y=3: 0*3, 2*3
            ],
        );
        // Bilinear at (1, 1.5): xf=0.5, yf=0.5
        // z_y0 = 0 + 0.5*(0 - 0) = 0.0
        // z_y1 = 0 + 0.5*(6 - 0) = 3.0
        // result = 0.0 + 0.5*(3.0 - 0.0) = 1.5
        assert!(approx_eq(lut.get_value(1.0, 1.5), 1.5));
    }

    // -- Minimal 2×2 table ---------------------------------------------------

    #[test]
    fn minimal_2x2_midpoint() {
        let lut = LookupTable2D::new([0.0, 1.0], [0.0, 1.0], [[0.0, 1.0], [1.0, 2.0]]);
        // z(0.5, 0.5): xf=0.5, yf=0.5
        // z_y0 = 0+0.5*1 = 0.5
        // z_y1 = 1+0.5*1 = 1.5
        // result = 0.5+0.5*1.0 = 1.0
        assert!(approx_eq(lut.get_value(0.5, 0.5), 1.0));
    }

    // -- Interpolation only along one axis -----------------------------------

    #[test]
    fn interpolate_along_x_only() {
        let lut = LookupTable2D::new(
            [0.0, 1.0, 2.0],
            [0.0, 1.0],
            [
                [0.0, 5.0, 10.0], // y=0
                [0.0, 5.0, 10.0], // y=1 (same row)
            ],
        );
        assert!(approx_eq(lut.get_value(0.5, 0.0), 2.5));
        assert!(approx_eq(lut.get_value(0.5, 1.0), 2.5));
        assert!(approx_eq(lut.get_value(0.5, 0.5), 2.5));
    }

    #[test]
    fn interpolate_along_y_only() {
        let lut = LookupTable2D::new(
            [0.0, 1.0],
            [0.0, 1.0, 2.0],
            [
                [0.0, 0.0],   // y=0
                [5.0, 5.0],   // y=1
                [10.0, 10.0], // y=2
            ],
        );
        assert!(approx_eq(lut.get_value(0.0, 0.5), 2.5));
        assert!(approx_eq(lut.get_value(1.0, 0.5), 2.5));
        assert!(approx_eq(lut.get_value(0.5, 1.5), 7.5));
    }
}
