use crate::topology::topology::Topology;
use crate::trackers::tracker::SuperTracker;
use crate::wfc::wave_propagator::WavePropagatorState;

pub trait PatternPicker<T: Topology + Clone> {
    fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), String>;
    fn get_random_possible_pattern_at(&mut self, index: usize, wave_propagator_state: &WavePropagatorState<T>) -> Option<usize>;
    fn as_super_tracker(&self) -> Option<&dyn SuperTracker<T>>;
    fn as_super_tracker_mut(&mut self) -> Option<&mut dyn SuperTracker<T>>;
}