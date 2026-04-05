//! Equivalent grid model: series RL network with ideal voltage source.
//!
//! Models the grid-side connection of a voltage-source inverter through a
//! series R-L impedance terminated by an ideal three-phase voltage source:
//!
//! ```text
//!  V_inv ──┤ R ├──┤ L ├── V_grid  (ideal source)
//!            i_grid →
//! ```
//!
//! Kirchhoff's voltage law in the stationary αβ frame:
//!
//! ```text
//! V_inv = R · i + L · di/dt + V_grid
//! ```
//!
//! Discretised with forward Euler:
//!
//! ```text
//! i[k+1] = i[k] + (Ts / L) · (V_inv[k] − R · i[k] − V_grid[k])
//! ```
//!
//! The ideal voltage source is driven by a [`ThreePhaseGenerator`] internally,
//! so the grid voltage tracks a programmable frequency and amplitude.

use pawer::frames::AlphaBeta;
use pawer::types::Real;

use crate::waveform_gen::ThreePhaseGenerator;

/// Equivalent grid model with series RL impedance and ideal voltage source.
///
/// # Example
///
/// ```
/// use pawer::frames::AlphaBeta;
/// use pawer_examples::grid_model::GridModel;
///
/// let mut grid = GridModel::new(100e-6);
/// grid.configure(0.1, 2e-3);          // R = 0.1 Ω, L = 2 mH
/// grid.set_grid_frequency(50.0);
/// grid.set_grid_amplitude(325.0);      // 230 Vrms peak
///
/// let v_inv = AlphaBeta::new(300.0, 0.0);
/// grid.update(v_inv);
/// let i_grid = grid.current();
/// ```
pub struct GridModel {
    resistance: Real,
    inductance: Real,
    sampling_time: Real,
    current: AlphaBeta<Real>,
    grid_voltage_ab: AlphaBeta<Real>,
    voltage_source: ThreePhaseGenerator,
}

impl GridModel {
    /// Create a grid model with the given sampling period (seconds).
    ///
    /// Defaults: R = 0 Ω, L = 1 mH, grid at 50 Hz / 1.0 pu amplitude.
    pub fn new(sampling_time: f64) -> Self {
        let mut vs = ThreePhaseGenerator::new(sampling_time);
        vs.set_frequency(50.0);
        vs.set_amplitude(1.0);
        Self {
            resistance: 0.0,
            inductance: 1e-3,
            sampling_time: sampling_time as Real,
            current: AlphaBeta::default(),
            grid_voltage_ab: AlphaBeta::default(),
            voltage_source: vs,
        }
    }

    // ── Configuration ─────────────────────────────────────────────────────

    /// Set the series resistance (Ω) and inductance (H).
    pub fn configure(&mut self, resistance: Real, inductance: Real) {
        assert!(inductance > 0.0, "Inductance must be positive");
        self.resistance = resistance;
        self.inductance = inductance;
    }

    /// Set the grid-side ideal voltage source frequency (Hz).
    pub fn set_grid_frequency(&mut self, hz: f64) {
        self.voltage_source.set_frequency(hz);
    }

    /// Set the grid-side ideal voltage source peak amplitude.
    pub fn set_grid_amplitude(&mut self, amplitude: f64) {
        self.voltage_source.set_amplitude(amplitude);
    }

    // ── Update ────────────────────────────────────────────────────────────

    /// Advance one simulation step.
    ///
    /// `v_inv` is the inverter output voltage in the stationary αβ frame.
    /// The method updates the internal grid voltage source and computes the
    /// new grid current using a forward-Euler discretisation of the RL
    /// circuit equation.
    pub fn update(&mut self, v_inv: AlphaBeta<Real>) {
        // Advance the ideal grid voltage source
        self.voltage_source.update();
        self.grid_voltage_ab = self.voltage_source.signal().to_alphabeta();

        // Forward Euler: i[k+1] = i[k] + (Ts/L) * (V_inv - R*i - V_grid)
        let ts_over_l = self.sampling_time / self.inductance;
        let di = (v_inv - self.current * self.resistance - self.grid_voltage_ab) * ts_over_l;
        self.current += di;
    }

    // ── Outputs ───────────────────────────────────────────────────────────

    /// Grid current in αβ (A).
    pub fn current(&self) -> AlphaBeta<Real> {
        self.current
    }

    /// Grid voltage from the ideal source in αβ (V).
    pub fn grid_voltage(&self) -> AlphaBeta<Real> {
        self.grid_voltage_ab
    }

    /// Immutable access to the internal voltage source.
    pub fn voltage_source(&self) -> &ThreePhaseGenerator {
        &self.voltage_source
    }

    /// Mutable access to the internal voltage source (for scheduling events).
    pub fn voltage_source_mut(&mut self) -> &mut ThreePhaseGenerator {
        &mut self.voltage_source
    }

    /// Series resistance (Ω).
    pub fn resistance(&self) -> Real {
        self.resistance
    }

    /// Series inductance (H).
    pub fn inductance(&self) -> Real {
        self.inductance
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TS: f64 = 100e-6;
    const EPS: f64 = 1e-4;

    fn approx(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn zero_voltage_zero_current() {
        let mut grid = GridModel::new(TS);
        grid.configure(0.1, 2e-3);
        grid.set_grid_amplitude(0.0);

        let v_inv = AlphaBeta::new(0.0, 0.0);
        for _ in 0..1000 {
            grid.update(v_inv);
        }

        assert!(
            approx(grid.current().alpha() as f64, 0.0, EPS),
            "Expected zero current, got α = {}",
            grid.current().alpha()
        );
        assert!(
            approx(grid.current().beta() as f64, 0.0, EPS),
            "Expected zero current, got β = {}",
            grid.current().beta()
        );
    }

    #[test]
    fn equal_voltage_sources_no_current() {
        // When V_inv == V_grid, no current flows (only if R > 0 to damp).
        let mut grid = GridModel::new(TS);
        grid.configure(1.0, 2e-3);
        grid.set_grid_frequency(50.0);
        grid.set_grid_amplitude(1.0);

        let mut inv = ThreePhaseGenerator::new(TS);
        inv.set_frequency(50.0);
        inv.set_amplitude(1.0);

        for _ in 0..50_000 {
            inv.update();
            let v_inv = inv.signal().to_alphabeta();
            grid.update(v_inv);
        }

        let i_mag = {
            let i = grid.current();
            ((i.alpha() * i.alpha() + i.beta() * i.beta()) as f64).sqrt()
        };
        assert!(
            i_mag < 0.05,
            "Current magnitude should be near zero when V_inv ≈ V_grid, got {i_mag:.4}"
        );
    }

    #[test]
    fn steady_state_current_magnitude() {
        // Sinusoidal steady state: |I| = |V_inv - V_grid| / |Z|
        // With V_inv amplitude = 1.1, V_grid = 1.0, ΔV = 0.1
        // Z = R + jωL, |Z| = sqrt(R² + (ωL)²)
        // At 50 Hz, ω = 2π·50 ≈ 314.16
        // R = 0.1, L = 2e-3 → ωL = 0.6283
        // |Z| = sqrt(0.01 + 0.3948) ≈ 0.636
        // Expected |I| ≈ 0.1 / 0.636 ≈ 0.157
        use std::f64::consts::PI;

        let r: Real = 0.1;
        let l: Real = 2e-3;
        let f = 50.0;
        let omega = 2.0 * PI * f;
        let z_mag = ((r as f64 * r as f64) + (omega * l as f64) * (omega * l as f64)).sqrt();
        let delta_v = 0.1;
        let expected_i = delta_v / z_mag;

        let mut grid = GridModel::new(TS);
        grid.configure(r, l);
        grid.set_grid_frequency(f);
        grid.set_grid_amplitude(1.0);

        let mut inv = ThreePhaseGenerator::new(TS);
        inv.set_frequency(f);
        inv.set_amplitude(1.1);

        // Run long enough to reach steady state
        for _ in 0..100_000 {
            inv.update();
            let v_inv = inv.signal().to_alphabeta();
            grid.update(v_inv);
        }

        let i = grid.current();
        let i_mag = ((i.alpha() * i.alpha() + i.beta() * i.beta()) as f64).sqrt();
        assert!(
            approx(i_mag, expected_i, 0.02),
            "Steady-state |I| = {i_mag:.4}, expected ≈ {expected_i:.4}"
        );
    }
}
