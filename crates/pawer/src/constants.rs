//! Mathematical and engineering constants used throughout `pawer`, together
//! with common unit-conversion helpers.

use crate::types::Real;

// ── Mathematical constants ───────────────────────────────────────────────────

pub const PI: Real = core::f32::consts::PI;
pub const TWO_PI: Real = 2.0 * PI;
pub const SQRT_2: Real = core::f32::consts::SQRT_2;
pub const SQRT_3: Real = 1.732_050_8;
pub const SQRT_3_OVER_2: Real = SQRT_3 / 2.0;
pub const TWO_THIRDS: Real = 2.0 / 3.0;
pub const SQRT_TWO_THIRDS: Real = 0.816_496_6;

// ── Voltage / current scaling factors ────────────────────────────────────────

/// Multiply an RMS value to obtain its peak value (√2).
pub const RMS_TO_PEAK: Real = SQRT_2;

/// Multiply a peak value to obtain its RMS value (1/√2).
pub const PEAK_TO_RMS: Real = 1.0 / SQRT_2;

/// Multiply a phase RMS voltage to obtain the line-to-line RMS voltage (√3).
pub const PHASE_TO_LINE_RMS: Real = SQRT_3;

/// Multiply a line-to-line RMS voltage to obtain the phase RMS voltage (1/√3).
pub const LINE_TO_PHASE_RMS: Real = 1.0 / SQRT_3;

// ── Conversion functions ─────────────────────────────────────────────────────

/// Convert degrees to radians.
pub const fn deg_to_rad(degrees: Real) -> Real {
    degrees * PI / 180.0
}

/// Convert radians to degrees.
pub const fn rad_to_deg(radians: Real) -> Real {
    radians * 180.0 / PI
}

/// Convert a frequency in hertz to angular velocity in rad/s.
pub const fn hz_to_rad_per_sec(hertz: Real) -> Real {
    hertz * TWO_PI
}

/// Convert phase RMS to line-to-line RMS.
pub const fn phase_to_line(phase_val: Real) -> Real {
    phase_val * PHASE_TO_LINE_RMS
}

/// Convert line-to-line RMS to phase RMS.
pub const fn line_to_phase(line_val: Real) -> Real {
    line_val * LINE_TO_PHASE_RMS
}

/// Convert line-to-line RMS voltage to phase peak voltage.
pub const fn line_to_phase_peak(line_rms: Real) -> Real {
    line_rms * LINE_TO_PHASE_RMS * RMS_TO_PEAK
}

/// Convert milliseconds to seconds.
pub const fn ms(milliseconds: Real) -> Real {
    milliseconds / 1_000.0
}

/// Convert microseconds to seconds.
pub const fn us(microseconds: Real) -> Real {
    microseconds / 1_000_000.0
}

/// Convert nanoseconds to seconds.
pub const fn ns(nanoseconds: Real) -> Real {
    nanoseconds / 1_000_000_000.0
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: Real, b: Real) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn test_deg_to_rad() {
        assert!(approx_eq(deg_to_rad(0.0), 0.0));
        assert!(approx_eq(deg_to_rad(180.0), PI));
        assert!(approx_eq(deg_to_rad(360.0), TWO_PI));
        assert!(approx_eq(deg_to_rad(90.0), PI / 2.0));
    }

    #[test]
    fn test_rad_to_deg() {
        assert!(approx_eq(rad_to_deg(0.0), 0.0));
        assert!(approx_eq(rad_to_deg(PI), 180.0));
        assert!(approx_eq(rad_to_deg(TWO_PI), 360.0));
        assert!(approx_eq(rad_to_deg(PI / 2.0), 90.0));
    }

    #[test]
    fn test_deg_rad_roundtrip() {
        let original = 123.456;
        assert!(approx_eq(rad_to_deg(deg_to_rad(original)), original));
    }

    #[test]
    fn test_hz_to_rad_per_sec() {
        assert!(approx_eq(hz_to_rad_per_sec(1.0), TWO_PI));
        assert!(approx_eq(hz_to_rad_per_sec(50.0), 50.0 * TWO_PI));
        assert!(approx_eq(hz_to_rad_per_sec(0.0), 0.0));
    }

    #[test]
    fn test_phase_to_line() {
        assert!(approx_eq(phase_to_line(1.0), SQRT_3));
        assert!(approx_eq(phase_to_line(230.0), 230.0 * SQRT_3));
    }

    #[test]
    fn test_line_to_phase() {
        assert!(approx_eq(line_to_phase(SQRT_3), 1.0));
        assert!(approx_eq(line_to_phase(400.0), 400.0 / SQRT_3));
    }

    #[test]
    fn test_line_to_phase_peak() {
        // line_rms * (1/√3) * √2
        let expected = 400.0 * LINE_TO_PHASE_RMS * RMS_TO_PEAK;
        assert!(approx_eq(line_to_phase_peak(400.0), expected));
    }

    #[test]
    fn test_ms() {
        assert!(approx_eq(ms(1000.0), 1.0));
        assert!(approx_eq(ms(1.0), 0.001));
        assert!(approx_eq(ms(0.0), 0.0));
    }

    #[test]
    fn test_us() {
        assert!(approx_eq(us(1_000_000.0), 1.0));
        assert!(approx_eq(us(1.0), 1e-6));
        assert!(approx_eq(us(0.0), 0.0));
    }

    #[test]
    fn test_ns() {
        assert!(approx_eq(ns(1_000_000_000.0), 1.0));
        assert!(approx_eq(ns(1.0), 1e-9));
        assert!(approx_eq(ns(0.0), 0.0));
    }
}
