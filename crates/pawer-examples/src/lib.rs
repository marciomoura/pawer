// Reusable simulation building blocks and example scenarios for the pawer ecosystem.
//
// This crate provides:
// - `waveform_gen`: Three-phase sinusoidal signal generation with disturbance events.
// - `grid_model`: Equivalent grid model (series RL + ideal voltage source).
// - `grid_current_controller`: dq-frame current controller for grid-connected inverters.

pub mod grid_current_controller;
pub mod grid_model;
pub mod waveform_gen;
