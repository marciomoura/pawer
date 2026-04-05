#![no_std]

// Foundation modules
pub mod angle;
pub mod constants;
pub mod frames;
pub mod limit;
pub mod per_unit;
pub mod reciprocal;
pub mod types;

// Control blocks
pub mod derivative;
pub mod first_order_lpf;
pub mod integrator;
pub mod second_order_filter;

// Timer and delay blocks
pub mod elapsed_timer;
pub mod interval_timer;
pub mod off_delay;
pub mod on_delay;
pub mod on_off_delay;

// Logic and limiter blocks
pub mod boolean_debouncer;
pub mod edge_detector;
pub mod hysteresis_limiter;
pub mod rate_of_change_limiter;

// Lookup tables and ramp
pub mod linear_ramp;
pub mod lookup_table_1d;
pub mod lookup_table_2d;

// Controllers
pub mod pi_controller;
pub mod srf_pll;
