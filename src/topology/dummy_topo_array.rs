use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::models::tile_model_mapping::TileModelMapping;
use crate::topology::topo_array::TopoArray;
use crate::topology::topology::{Topology, TopologyError};
use crate::trackers::{index_picker::IndexPicker, pattern_picker::PatternPicker, tracker::Tracker};
use crate::trait_error::TraitError;
use crate::wfc::wave_propagator::WavePropagatorState;

/// A dummy TopoArray implementation for cases where we need a placeholder
pub struct DummyTopoArray<T: Clone + 'static> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + 'static> DummyTopoArray<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Clone + 'static, TopoT: Topology + Clone + 'static> TopoArray<T, TopoT>
    for DummyTopoArray<T>
{
    fn topology(&self) -> Option<&TopoT> {
        unimplemented!();
    }

    fn get_coord(&self, x: usize, y: usize, z: usize) -> Result<&T, TopologyError> {
        unimplemented!();
    }

    fn get_index(&self, index: usize) -> Result<&T, TopologyError> {
        unimplemented!();
    }

    fn get_value_from_index(&self, index: usize) -> Option<&T> {
        unimplemented!();
    }

    fn get_value_from_coord(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        unimplemented!();
    }

    fn get_id_from_index(&self, index: usize) -> Option<usize> {
        unimplemented!();
    }

    fn get_id_from_coord(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        unimplemented!();
    }

    fn clone_box(&self) -> Option<Box<dyn TopoArray<T, TopoT>>> {
        unimplemented!();
    }
}

pub struct AsIndexPicker<T: Topology + Clone, R: IndexPicker<T> + 'static>(
    pub Rc<RefCell<R>>,
    PhantomData<T>,
);
impl<T: Topology + Clone, R: IndexPicker<T>> IndexPicker<T> for AsIndexPicker<T, R> {
    fn init(
        &mut self,
        wave_propagator_state: &WavePropagatorState<T>,
        topology: &T,
    ) -> Result<(), TraitError> {
        self.0.borrow_mut().init(wave_propagator_state, topology)
    }

    fn get_random_index(
        &mut self,
        wave_propagator_state: &WavePropagatorState<T>,
        tile_model_mapping: &TileModelMapping<T>,
    ) -> Option<i32> {
        self.0
            .borrow_mut()
            .get_random_index(wave_propagator_state, tile_model_mapping)
    }

    fn as_tracker(&self) -> Option<&dyn Tracker<T>> {
        None
    }

    fn as_tracker_mut(&mut self) -> Option<&mut dyn Tracker<T>> {
        None
    }

    fn as_pattern_picker(&self) -> Option<&dyn PatternPicker<T>> {
        None
    }

    fn as_pattern_picker_mut(&mut self) -> Option<&mut dyn PatternPicker<T>> {
        None
    }
}
pub fn to_index_picker<T: Topology + Clone + 'static, R: IndexPicker<T> + 'static>(
    index_picker: Rc<RefCell<R>>,
) -> Rc<RefCell<dyn IndexPicker<T>>> {
    Rc::new(RefCell::new(AsIndexPicker(
        index_picker.clone(),
        PhantomData,
    )))
}
