#![no_std]

// Foundation modules
pub mod types;
pub mod constants;
pub mod angle;
pub mod frames;
pub mod limit;
pub mod per_unit;
pub mod precomputed_divisor;

// Control blocks
pub mod integrator;
pub mod derivative;
pub mod first_order_lpf;
pub mod second_order_filter;

// Timer and delay blocks
pub mod on_delay;
pub mod off_delay;
pub mod on_off_delay;
pub mod elapsed_timer;
pub mod interval_timer;

// Logic and limiter blocks
pub mod edge_detector;
pub mod boolean_debouncer;
pub mod hysteresis_limiter;
pub mod rate_of_change_limiter;

// Lookup tables and ramp
pub mod lookup_table_1d;
pub mod lookup_table_2d;
pub mod linear_ramp;
