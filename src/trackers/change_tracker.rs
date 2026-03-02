use std::fmt::Debug;
use std::marker::PhantomData;
use crate::models::tile_model_mapping::TileModelMapping;
use crate::topology::topology::{Topology, TopologyError};
use crate::trackers::index_picker::IndexPicker;
use crate::trackers::pattern_picker::PatternPicker;
use crate::trackers::tracker::Tracker;

pub struct ChangeTracker<T: Topology + Clone> {
    index_count: usize,
    // Using pattern topology
    changed_indices: Vec<usize>,
    // Double buffering
    changed_indices2: Vec<usize>,
    generation: i32,
    last_changed_generation: Vec<i32>,
    phantom_data: PhantomData<T>,
}

impl<T: Topology + Clone> Debug for ChangeTracker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T: Topology + Clone> ChangeTracker<T> {
    pub fn new(index_count: usize) -> Self {
        Self::with_index_count(index_count)
    }

    pub fn with_index_count(index_count: usize) -> Self {
        Self {
            index_count,
            changed_indices: Vec::new(),
            changed_indices2: Vec::new(),
            generation: 0,
            last_changed_generation: Vec::new(),
            phantom_data: PhantomData,
        }
    }

    pub fn changed_count(&self) -> usize {
        self.changed_indices.len()
    }

    /// Returns the set of indices that have been changed since the last call.
    pub fn get_changed_indices(&mut self, tile_model_mapping: &TileModelMapping<T>) -> Result<Vec<usize>, TopologyError> {
        let current_changed_indices = std::mem::take(&mut self.changed_indices);

        // Switch over double buffering
        std::mem::swap(&mut self.changed_indices, &mut self.changed_indices2);
        self.changed_indices.clear();
        self.generation += 1;

        if self.generation == i32::MAX {
            return Err(TopologyError::Other(
                format!("Change Tracker doesn't support more than {} executions", i32::MAX)
            ));
        }

        if tile_model_mapping.pattern_coord_to_tile_coord_index_and_offset.is_none() {
            Ok(current_changed_indices)
        } else {
            // Handle the overlapped case
            let mapping = tile_model_mapping.pattern_coord_to_tile_coord_index_and_offset
                .as_ref().ok_or(TopologyError::Other("unable to get mapping".to_string()))?;
            let mut result = Vec::new();

            for i in current_changed_indices {
                if let Ok(coord_data) = mapping.get_index(i) {
                    for (_, tile_index, _) in coord_data {
                        result.push(*tile_index as usize);
                    }
                }
            }

            Ok(result)
        }
    }
}

impl<T: Topology + Clone> Tracker<T> for ChangeTracker<T> {
    fn reset(&mut self) -> Result<(), String> {
        self.changed_indices = Vec::new();
        self.changed_indices2 = Vec::new();
        self.last_changed_generation = vec![0; self.index_count];
        self.generation = 1;
        Ok(())
    }

    fn do_ban(&mut self, index: usize, _pattern: usize) -> Result<(), String>{
        let g = self.last_changed_generation[index];
        if g != self.generation {
            self.last_changed_generation[index] = self.generation;
            self.changed_indices.push(index);
        }
        Ok(())
    }

    fn undo_ban(&mut self, index: usize, _pattern: usize) -> Result<(), String>{
        let g = self.last_changed_generation[index];
        if g != self.generation {
            self.last_changed_generation[index] = self.generation;
            self.changed_indices.push(index);
        }

        Ok(())
    }

    fn as_pattern_picker(&self) -> Option<&dyn PatternPicker<T>> {
        None
    }

    fn as_pattern_picker_mut(&mut self) -> Option<&mut dyn PatternPicker<T>> {
        None
    }

    fn as_index_picker(&self) -> Option<&dyn IndexPicker<T>> {
        None
    }

    fn as_index_picker_mut(&mut self) -> Option<&mut dyn IndexPicker<T>> {
        None
    }
}