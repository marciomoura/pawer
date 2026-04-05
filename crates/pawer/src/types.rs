/// Default floating-point type for all computations in the `pawer` crate.
///
/// Defaults to [`f32`], matching the single-precision default of the original
/// C++ codebase. Switch to [`f64`] by changing this alias if double-precision
/// is required.
pub type Real = f32;
