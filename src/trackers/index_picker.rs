use crate::models::tile_model_mapping::TileModelMapping;
use crate::topology::topology::Topology;
use crate::trackers::pattern_picker::PatternPicker;
use crate::trackers::tracker::Tracker;
use crate::trait_error::TraitError;
use crate::wfc::wave_propagator::WavePropagatorState;

pub trait IndexPicker<T: Topology + Clone> {
    fn init(
        &mut self,
        wave_propagator_state: &WavePropagatorState<T>,
        topology: &T,
    ) -> Result<(), TraitError>;
    fn get_random_index(
        &mut self,
        wave_propagator_state: &WavePropagatorState<T>,
        tile_model_mapping: &TileModelMapping<T>,
    ) -> Option<i32>;
    // fn as_super_tracker(&self) -> Option<&dyn SuperTracker<T>>;
    // fn as_super_tracker_mut(&mut self) -> Option<&mut dyn SuperTracker<T>>;

    fn as_tracker(&self) -> Option<&dyn Tracker<T>>;
    fn as_tracker_mut(&mut self) -> Option<&mut dyn Tracker<T>>;

    fn as_pattern_picker(&self) -> Option<&dyn PatternPicker<T>>;
    fn as_pattern_picker_mut(&mut self) -> Option<&mut dyn PatternPicker<T>>;
}
