// Software-in-the-loop simulation support for pawer control algorithms.
// This crate is std-only and intended for host-side simulation and testing.

pub mod cli;
pub mod context;
pub mod engine;
pub mod export;
pub mod logger;
pub mod plotter;
pub mod scenario;

/// Convenience re-exports for typical scenario definitions.
pub mod prelude {
    pub use crate::cli::SimCli;
    pub use crate::context::{SignalId, SimContext};
    pub use crate::scenario::Scenario;
}
