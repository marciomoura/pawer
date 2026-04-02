//! Example: Low-pass filter step response
//!
//! Simulates a first-order and a second-order low-pass filter driven by a
//! unit-step input and prints the time-domain output to stdout.
//!
//! Run with:
//! ```text
//! cargo run --example filter_example
//! ```

use pawer::filters::{LowPassFilter, SecondOrderLowPassFilter};

fn main() {
    // Parameters
    let cutoff_rad = 200.0_f64; // 200 rad/s ≈ 31.8 Hz
    let sample_time = 1e-4_f64; // 100 µs (10 kHz sampling)
    let duration = 0.05_f64; // 50 ms simulation

    let mut lpf1 = LowPassFilter::new(cutoff_rad, sample_time);
    let mut lpf2 = SecondOrderLowPassFilter::new(cutoff_rad, sample_time);

    let steps = (duration / sample_time) as usize;

    println!(
        "{:<12} {:<18} {:<18}",
        "time [s]", "1st-order LPF", "2nd-order LPF"
    );
    println!("{}", "-".repeat(50));

    for i in 0..=steps {
        let t = i as f64 * sample_time;
        // Unit step starts at t = 0
        let input = if i == 0 { 0.0 } else { 1.0 };

        let out1 = lpf1.update(input);
        let out2 = lpf2.update(input);

        // Print every 50th sample to keep output readable.
        if i % 50 == 0 {
            println!("{:<12.5} {:<18.6} {:<18.6}", t, out1, out2);
        }
    }

    println!();
    println!(
        "Final values: 1st-order = {:.6}, 2nd-order = {:.6}",
        lpf1.update(1.0),
        lpf2.update(1.0)
    );
}
