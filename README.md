# pawer

A `no_std` Rust workspace for power electronic converter control algorithms.

## Crates

| Crate | Description |
|---|---|
| [`pawer`](crates/pawer) | `no_std` core library — foundational control building blocks (filters, integrators, transforms, limiters, controllers) |
| [`pawer-sim`](crates/pawer-sim) | Software-in-the-loop simulation utilities — `std`-enabled, host-only. Generic over any discrete-time system; works with `pawer` blocks or custom code |

## Design Goals

- **Embedded-first**: `pawer` has no dynamic memory allocation and no `std`. All state is stack-allocated.
- **Composable**: blocks are discrete-time, each owning its own state with a consistent `update` interface.
- **Testable**: `pawer-sim` runs simulation loops against any discrete-time system — `pawer` blocks, custom code, or a mix — enabling closed-loop performance evaluation before deployment.

## Getting Started

```toml
[dependencies]
pawer = "0.1"

# For simulation/testing only:
[dev-dependencies]
pawer-sim = "0.1"
```