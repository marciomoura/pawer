# pawer

A `no_std` Rust workspace for power electronic converter control algorithms.

## Crates

| Crate | Description |
|---|---|
| [`pawer`](crates/pawer) | `no_std` core library — foundational control building blocks (filters, integrators, transforms, limiters, controllers) |
| [`pawer-sim`](crates/pawer-sim) | Software-in-the-loop simulation support — host-side waveform capture, performance metrics, and test harnesses |

## Design Goals

- **Embedded-first**: `pawer` has no dynamic memory allocation and no `std`. All state is stack-allocated.
- **Composable**: blocks are discrete-time, each owning its own state with a consistent `update` interface.
- **Testable**: `pawer-sim` runs the same algorithm code on a host, enabling closed-loop simulation before deployment.

## Getting Started

```toml
[dependencies]
pawer = "0.1"

# For simulation/testing only:
[dev-dependencies]
pawer-sim = "0.1"
```