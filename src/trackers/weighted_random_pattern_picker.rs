use std::marker::PhantomData;
use crate::models::tile_model_mapping::TileModelMapping;
use crate::topology::topology::Topology;
use crate::trackers::index_picker::IndexPicker;
use crate::trackers::pattern_picker::PatternPicker;
use crate::trackers::random_picker_utils::RandomPickerUtils;
use crate::trackers::tracker::{Tracker};
use crate::wfc::wave_propagator::WavePropagatorState;

#[derive(Default)]
pub struct WeightedRandomPatternPicker<T: Topology + Clone> {
    frequencies: Option<Vec<f64>>,
    phantom_data: PhantomData<T>
}

impl<T: Topology + Clone> WeightedRandomPatternPicker<T> {
    pub fn new() -> Self {
        Self {
            frequencies: None,
            phantom_data: PhantomData
        }
    }
}

impl<T: Topology + Clone> PatternPicker<T> for WeightedRandomPatternPicker<T> {
    fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), String> {
        self.frequencies = Some(wave_propagator_state.get_frequencies());
        Ok(())
    }

    fn get_random_possible_pattern_at(&mut self, index: usize, wave_propagator_state: &WavePropagatorState<T>) -> Option<usize> {
        if wave_propagator_state.get_wave().is_none() {
            println!("No wave set for WeightedRandomPatternPicker");
            return None;
        }

        let frequencies = self.frequencies.as_ref()?;
        let pattern = RandomPickerUtils::get_random_possible_pattern::<T>(
            wave_propagator_state.get_wave().as_ref().unwrap(),
            wave_propagator_state.get_random_double(),
            index,
            frequencies,
        )?;

        #[allow(unused_comparisons)]
        if pattern < 0 {
            None
        } else {
            Some(pattern)
        }
    }

    fn as_tracker(&self) -> Option<&dyn Tracker<T>> {
        None
    }

    fn as_tracker_mut(&mut self) -> Option<&mut dyn Tracker<T>> {
        None
    }

    fn as_index_picker(&self) -> Option<&dyn IndexPicker<T>> {
        None
    }

    fn as_index_picker_mut(&mut self) -> Option<&mut dyn IndexPicker<T>> {
        None
    }
}

// impl<T: Topology + Clone> Tracker for WeightedRandomPatternPicker<T> {
//     fn reset(&mut self) -> Result<(), String> {
//         Err("WeightedRandomPatternPicker is not a Tracker".to_string())
//     }
//
//     fn do_ban(&mut self, index: usize, pattern: usize) -> Result<(), String> {
//         Err("WeightedRandomPatternPicker is not a Tracker".to_string())
//     }
//
//     fn undo_ban(&mut self, index: usize, pattern: usize) -> Result<(), String> {
//         Err("WeightedRandomPatternPicker is not a Tracker".to_string())
//     }
// }
//
// impl<T: Topology + Clone> IndexPicker<T> for WeightedRandomPatternPicker<T> {
//     fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), String> {
//         unimplemented!("WeightedRandomPatternPicker is not an IndexPicker")
//     }
//
//     fn get_random_index(&mut self, wave_propagator_state: &WavePropagatorState<T>, tile_model_mapping: &TileModelMapping<T>) -> Option<i32> {
//         unimplemented!("WeightedRandomPatternPicker is not an IndexPicker")
//     }
//
//     fn as_super_tracker(&self) -> Option<&dyn SuperTracker<T>> {
//         Some(self as &dyn SuperTracker<T>)
//     }
//
//     fn as_super_tracker_mut(&mut self) -> Option<&mut dyn SuperTracker<T>> {
//         Some(self as &mut dyn SuperTracker<T>)
//     }
// }
//
// impl<T: Topology + Clone> SuperTracker<T> for WeightedRandomPatternPicker<T> {
//     fn is_index_picker(&self) -> bool {
//         false
//     }
//
//     fn is_pattern_picker(&self) -> bool {
//         true
//     }
//
//     fn is_tracker(&self) -> bool {
//         false
//     }
// }