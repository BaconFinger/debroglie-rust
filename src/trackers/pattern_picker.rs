use std::error::Error;
use crate::topology::topology::Topology;
use crate::trackers::index_picker::IndexPicker;
use crate::trackers::tracker::{Tracker};
use crate::trait_error::TraitError;
use crate::wfc::wave_propagator::WavePropagatorState;

pub trait PatternPicker<T: Topology + Clone> {

    fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), TraitError>;
    fn get_random_possible_pattern_at(&mut self, index: usize, wave_propagator_state: &WavePropagatorState<T>) -> Option<usize>;
    // fn as_super_tracker(&self) -> Option<&dyn SuperTracker<T>>;
    // fn as_super_tracker_mut(&mut self) -> Option<&mut dyn SuperTracker<T>>;

    fn as_tracker(&self) -> Option<&dyn Tracker<T>>;
    fn as_tracker_mut(&mut self) -> Option<&mut dyn Tracker<T>>;
    
    fn as_index_picker(&self) -> Option<&dyn IndexPicker<T>>;
    fn as_index_picker_mut(&mut self) -> Option<&mut dyn IndexPicker<T>>;
}