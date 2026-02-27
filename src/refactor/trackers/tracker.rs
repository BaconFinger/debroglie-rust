use crate::refactor::topology::topology::Topology;
use crate::refactor::trackers::heap_entropy_tracker::HeapEntropyTracker;
use crate::refactor::trackers::index_picker::IndexPicker;
use crate::refactor::trackers::pattern_picker::PatternPicker;

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
pub trait Tracker {
    /// Reset the tracker to its initial state
    fn reset(&mut self) -> Result<(), String>;

    /// Called when a pattern is banned at a specific index
    fn do_ban(&mut self, index: usize, pattern: usize);

    /// Called when a pattern ban is undone at a specific index
    fn undo_ban(&mut self, index: usize, pattern: usize);
}

/// A Tracker that also implements IndexPicker and PatternPicker.
/// TODO: Figure this out for real.
/// The problem is that in the init of TilePropagator, some
/// classes like HeapEntropyTracker are treated as IndexPickers, but in the run() they are
/// actually trackers.
/// So just implement this trait and mark unused stuff with not_implemented!() so once everything
/// is ported I can figure it out.
pub trait SuperTracker<T: Topology + Clone>: Tracker + IndexPicker<T> + PatternPicker<T> {
    fn is_index_picker(&self) -> bool;
    fn is_pattern_picker(&self) -> bool;
    fn is_tracker(&self) -> bool;
}