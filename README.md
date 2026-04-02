# pawer

[![CI](https://github.com/marciomoura/pawer/actions/workflows/ci.yml/badge.svg)](https://github.com/marciomoura/pawer/actions/workflows/ci.yml)

A Rust library that contains a collection of basic-blocks commonly used for
control of **power electronic converters** – low-pass filters, PI controllers,
integrators, and more.

---

## Modules

| Module | Contents |
|---|---|
| `filters` | `LowPassFilter` (1st-order), `SecondOrderLowPassFilter` (Butterworth) |
| `controllers` | `PiController` with anti-windup |
| `integrator` | `Integrator` with optional saturation limits |

All blocks are discrete-time and operate sample-by-sample.  
Filters use the **bilinear (Tustin) transform**; the integrator uses **forward Euler**.

---

## Quick start

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
pawer = { git = "https://github.com/marciomoura/pawer" }
```

### First-order low-pass filter

```rust
use pawer::filters::LowPassFilter;

let cutoff_rad = 200.0; // 200 rad/s ≈ 31.8 Hz
let sample_time = 1e-4; // 100 µs (10 kHz)

let mut lpf = LowPassFilter::new(cutoff_rad, sample_time);

loop {
    let raw_measurement = read_sensor();
    let filtered = lpf.update(raw_measurement);
}
```

### PI controller

```rust
use pawer::controllers::PiController;

let mut pi = PiController::new(
    1.0,    // Kp
    10.0,   // Ki  (rad/s)
    1e-4,   // Ts  (s)
    -100.0, // output min
    100.0,  // output max
);

let reference = 5.0;
loop {
    let feedback = read_feedback();
    let error    = reference - feedback;
    let duty     = pi.update(error);
    set_duty_cycle(duty);
}
```

### Integrator

```rust
use pawer::integrator::Integrator;

let mut integ = Integrator::new(1e-4, Some(-10.0), Some(10.0));
let accumulated = integ.update(signal);
```

---

## Running the filter example

```bash
cargo run --example filter_example
```

This prints the step response of both the first-order and second-order low-pass
filters over 50 ms at 10 kHz sampling.

---

## Running the tests

```bash
cargo test
```

---

## CI

GitHub Actions runs on every push and pull request:

- **fmt** – `cargo fmt --check`
- **clippy** – `cargo clippy -- -D warnings`
- **test** – `cargo test`
- **example** – `cargo run --example filter_example`
