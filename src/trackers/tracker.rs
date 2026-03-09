use crate::topology::topology::Topology;
use crate::trackers::index_picker::IndexPicker;
use crate::trackers::pattern_picker::PatternPicker;
use crate::trait_error::TraitError;

/// Callback for when choices/backtracks occur on WavePropagator
/// TODO: Move this trait elsewhere?
pub trait ChoiceObserver {
    /// Called before the wave propagator is updated for the choice
    fn make_choice(&mut self, index: usize, pattern: usize);

    /// Called after the wave propagator is backtracked
    fn backtrack(&mut self);
}

/// Trackers are objects that maintain state that is a summary of the current state of the propagator.
/// By updating that state as the propagator changes, they can give a significant performance benefit
/// over calculating the value from scratch each time it is needed.
pub trait Tracker<T: Topology + Clone> {
    /// Reset the tracker to its initial state
    fn reset(&mut self) -> Result<(), TraitError>;

    /// Called when a pattern is banned at a specific index
    fn do_ban(&mut self, index: usize, pattern: usize) -> Result<(), TraitError>;

    /// Called when a pattern ban is undone at a specific index
    fn undo_ban(&mut self, index: usize, pattern: usize) -> Result<(), TraitError>;

    fn as_pattern_picker(&self) -> Option<&dyn PatternPicker<T>>;
    fn as_pattern_picker_mut(&mut self) -> Option<&mut dyn PatternPicker<T>>;

    fn as_index_picker(&self) -> Option<&dyn IndexPicker<T>>;
    fn as_index_picker_mut(&mut self) -> Option<&mut dyn IndexPicker<T>>;
}
