/// Rising and falling edge detector for boolean signals.
///
/// Detects transitions in a boolean input signal. A **rising edge** occurs when
/// the input changes from `false` to `true`; a **falling edge** occurs on the
/// opposite transition. The detector must be called once per control cycle via
/// [`update`](EdgeDetector::update).
#[derive(Default)]
pub struct EdgeDetector {
    rising_edge: bool,
    falling_edge: bool,
    previous_input: bool,
}

impl EdgeDetector {
    /// Creates a new [`EdgeDetector`] with no prior input history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Processes the current `input` and returns `true` when a rising edge is
    /// detected.
    pub fn update(&mut self, input: bool) -> bool {
        self.rising_edge = !self.previous_input && input;
        self.falling_edge = self.previous_input && !input;
        self.previous_input = input;
        self.rising_edge
    }

    /// Returns `true` if the last [`update`](Self::update) detected a rising
    /// edge.
    pub fn is_rising_edge(&self) -> bool {
        self.rising_edge
    }

    /// Returns `true` if the last [`update`](Self::update) detected a falling
    /// edge.
    pub fn is_falling_edge(&self) -> bool {
        self.falling_edge
    }

    /// Resets the detector to its initial state (no edges, previous input
    /// `false`).
    pub fn reset(&mut self) {
        self.rising_edge = false;
        self.falling_edge = false;
        self.previous_input = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_has_no_edges() {
        let det = EdgeDetector::new();
        assert!(!det.is_rising_edge());
        assert!(!det.is_falling_edge());
    }

    #[test]
    fn false_to_true_is_rising_edge() {
        let mut det = EdgeDetector::new();
        let rising = det.update(true);
        assert!(rising);
        assert!(det.is_rising_edge());
        assert!(!det.is_falling_edge());
    }

    #[test]
    fn true_to_true_no_edge() {
        let mut det = EdgeDetector::new();
        det.update(true);
        det.update(true);
        assert!(!det.is_rising_edge());
        assert!(!det.is_falling_edge());
    }

    #[test]
    fn true_to_false_is_falling_edge() {
        let mut det = EdgeDetector::new();
        det.update(true);
        det.update(false);
        assert!(!det.is_rising_edge());
        assert!(det.is_falling_edge());
    }

    #[test]
    fn false_to_false_no_edge() {
        let mut det = EdgeDetector::new();
        det.update(false);
        assert!(!det.is_rising_edge());
        assert!(!det.is_falling_edge());
    }

    #[test]
    fn reset_clears_state() {
        let mut det = EdgeDetector::new();
        det.update(true);
        assert!(det.is_rising_edge());

        det.reset();
        assert!(!det.is_rising_edge());
        assert!(!det.is_falling_edge());

        // After reset, previous_input is false, so true is a rising edge again.
        let rising = det.update(true);
        assert!(rising);
    }

    #[test]
    fn sequence_detects_correct_edges() {
        let mut det = EdgeDetector::new();
        let inputs = [false, true, true, false, true];
        let expected_rising = [false, true, false, false, true];
        let expected_falling = [false, false, false, true, false];

        for (i, &input) in inputs.iter().enumerate() {
            det.update(input);
            assert_eq!(
                det.is_rising_edge(),
                expected_rising[i],
                "rising mismatch at index {i}"
            );
            assert_eq!(
                det.is_falling_edge(),
                expected_falling[i],
                "falling mismatch at index {i}"
            );
        }
    }

    #[test]
    fn update_returns_rising_edge_flag() {
        let mut det = EdgeDetector::new();
        assert!(!det.update(false));
        assert!(det.update(true));
        assert!(!det.update(true));
        assert!(!det.update(false));
        assert!(det.update(true));
    }

    #[test]
    fn default_matches_new() {
        let det = EdgeDetector::default();
        assert!(!det.is_rising_edge());
        assert!(!det.is_falling_edge());
    }
}
