# Copilot Instructions for `pawer`

## Project

This is a **Cargo workspace** containing:

| Crate | Path | Purpose |
|---|---|---|
| `pawer` | `crates/pawer` | `no_std` core library of control building blocks for power electronic converters |
| `pawer-sim` | `crates/pawer-sim` | Software-in-the-loop (SIL) simulation support — `std`-enabled, host-only |

`pawer` targets embedded systems with **no dynamic memory allocation**. `pawer-sim` depends on `pawer` and provides host-side simulation, waveform capture, and testing harness utilities.

## Build, Test, and Lint Commands

All commands run from the workspace root unless noted.

```bash
# Build all crates
cargo build

# Build a single crate
cargo build -p pawer
cargo build -p pawer-sim

# Run all tests across the workspace
cargo test

# Run all tests for a single crate
cargo test -p pawer

# Run a single test by name
cargo test -p pawer <test_name>

# Run a single test in a specific module
cargo test -p pawer <module>::<test_name>

# Lint
cargo clippy

# Format
cargo fmt

# Check formatting without modifying files
cargo fmt --check
```

## Architecture

### `crates/pawer` (no_std core)

- `#![no_std]` at the crate root — never import from `std`. Use `core::*` equivalents.
- No heap allocation: no `alloc` crate, no `Vec`/`Box`/`String`. All state is stack-allocated.
- Fixed-size buffers use **const generics** (e.g., `Buffer<T, const N: usize>`).
- Numeric types should be generic over floating-point via `num-traits` (`Float`, `NumCast`) to support both `f32` and `f64`.
- For transcendentals (`sin`, `cos`, `sqrt`, etc.) in `no_std`, use `libm` (add as a dependency when needed).
- Public API is re-exported from `lib.rs` — keep the module tree internal, surface a clean flat API.

### `crates/pawer-sim` (SIL simulation, std)

- `std` is available; dynamic allocation (`Vec`, etc.) is allowed here.
- **No dependency on `pawer`** — `pawer-sim` is generic over any discrete-time system. Users wire `pawer` blocks (or their own code) into `pawer-sim` harnesses themselves.
- Intended for: running simulation loops, capturing time-series waveforms, computing performance metrics (overshoot, settling time, rise time, control tracking error, fault detection response), and building closed-loop test harnesses.
- Not intended to be flashed to a target — host execution only.

## Domain Conventions

Typical building blocks expected in this library (non-exhaustive):

- **Filters**: first/second-order IIR (low-pass, high-pass, band-pass), moving average
- **Integrators**: forward Euler, backward Euler, bilinear (Tustin)
- **Controllers**: PID, PR (proportional-resonant)
- **Limiters**: saturator/clamp, rate limiter, dead-band, hysteresis
- **Transforms**: Clarke (αβ), Park (dq), inverse transforms, PLL helpers
- **Modulators**: carrier-based PWM helpers, space-vector

All blocks are **discrete-time** (sample-period `T_s` is a parameter, not a global). Each block owns its state; no shared mutable globals.

## Key Conventions

- Rust edition 2024 — use edition-2024-compatible idioms.
- Every block is a `struct` holding its state; update logic lives in an `update(&mut self, input: T) -> T` method (or equivalent).
- Tests use `#[cfg(test)]` inline modules. Use `approx` (or manual epsilon checks) for floating-point assertions.
- `Cargo.toml` dependencies stay minimal. Adding a dependency requires it to be `no_std`-compatible (check `[features] default = ["std"]` patterns).
- No `unsafe` unless absolutely necessary for hardware-level primitives, and then it must be documented and isolated.
