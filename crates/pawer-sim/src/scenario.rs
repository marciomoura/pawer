use crate::context::SimContext;

/// Trait that users implement to define a simulation scenario.
///
/// A scenario combines `pawer` control blocks (or any custom models) into
/// a test case that can be driven interactively through the CLI.
///
/// # Example
///
/// ```ignore
/// struct MyScenario { /* blocks, state, ... */ }
///
/// impl Scenario for MyScenario {
///     fn init(&mut self, ctx: &mut SimContext) {
///         ctx.set_param("setpoint", 0.0);
///     }
///
///     fn step(&mut self, ctx: &mut SimContext) {
///         let sp = ctx.param("setpoint");
///         ctx.log("output", sp * 2.0);
///     }
/// }
/// ```
pub trait Scenario {
    /// Called once before simulation starts and on every reset.
    ///
    /// Use this to register signals, set initial parameter values, and
    /// configure control blocks.
    fn init(&mut self, ctx: &mut SimContext);

    /// Called every simulation step.
    ///
    /// Read parameters with [`SimContext::param`], compute outputs using
    /// your blocks, and log signals with [`SimContext::log`].
    fn step(&mut self, ctx: &mut SimContext);
}
