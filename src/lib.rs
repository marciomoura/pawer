//! # Pawer
//!
//! A collection of basic-blocks commonly used for control of power electronic converters.
//!
//! ## Modules
//!
//! - [`filters`] – Low-pass filters (first-order, second-order)
//! - [`controllers`] – PI controller with anti-windup
//! - [`integrator`] – Discrete-time integrator

pub mod controllers;
pub mod filters;
pub mod integrator;
