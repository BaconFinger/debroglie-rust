use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use crate::constraints::tile_constraint::TileConstraint;
use crate::models::tile_model::TileModel;
use crate::models::tile_model_mapping::TileModelMapping;
use crate::resolution::Resolution;
use crate::tile::TileVisual;
use crate::tile_propagator_options::{BacktrackType, IndexPickerType, TilePickerType, TilePropagatorOptions};
use crate::topology::topo_array::TopoArray;
use crate::topology::topo_array_1d::TopoArray1D;
use crate::topology::topology::{Topology, TopologyError};
use crate::trackers::entropy_tracker::EntropyTracker;
use crate::trackers::heap_entropy_tracker::HeapEntropyTracker;
use crate::trackers::index_picker::IndexPicker;
use crate::trackers::tracker::SuperTracker;
use crate::trackers::weighted_random_pattern_picker::WeightedRandomPatternPicker;
use crate::wfc::backtrack_policy::{BacktrackPolicy, ConstantBacktrackPolicy, PatienceBackjumpPolicy};
use crate::wfc::wave_propagator::{ModelConstraintAlgorithm, WaveConstraint, WavePropagator, WavePropagatorOptions, WavePropagatorState};

/// TilePropagator is the main entrypoint to the DeBroglie library.
/// It takes a TileModel and an output Topology and generates
/// an output array using those parameters.
///
/// Implementation-wise, this wraps a WavePropagator to do the majority of the work.
/// The only thing this class handles is conversion of tile objects into sets of patterns
/// and coordinate conversion.
pub struct TilePropagator<T>
where T: Topology + Clone + 'static
{
    // topology: T,
    // tile_model: Box<dyn TileModel<T>>,
    // tile_model_mapping: TileModelMapping<T>,
    wave_propagator: WavePropagator<T>,
    state: TilePropagatorState<T>,
}

pub struct TilePropagatorState<T>
where T: Topology + Clone + 'static {
    topology: T,
    // wave_propagator: WavePropagator<T>,
    tile_model: Box<dyn TileModel<T>>,
    tile_model_mapping: TileModelMapping<T>,
}

impl<T> TilePropagatorState<T>
where T: Topology + Clone + 'static {
    pub fn get_topology(&self) -> &T {
        &self.topology
    }
}

impl<T> TilePropagator<T>
where T: Topology + Clone + 'static
{
    /// Creates a new TilePropagator with default options, and returns a reference to it under an RcRefCell.
    /// TilePropagator must be under an RcRefCell, as its numerous dependencies require a reference
    /// to itself.
    pub fn init(
        tile_model: Box<dyn TileModel<T>>,
        topology: T,
        backtrack: bool,
        constraints: Option<Vec<Box<dyn TileConstraint<T>>>>,
    ) -> Result<Self, String> {
        let options = TilePropagatorOptions {
            backtrack: if backtrack { BacktrackType::Backtrack } else { BacktrackType::None },
            max_backtrack_depth: 0,
            constraints: constraints.unwrap_or(Vec::new()),
            random_double: Rc::new(|| {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                use std::time::{SystemTime, UNIX_EPOCH};

                let mut hasher = DefaultHasher::new();
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
                    .hash(&mut hasher);
                let hash = hasher.finish();
                (hash as f64) / (u64::MAX as f64)
            }),
            index_picker_type: IndexPickerType::Default,
            tile_picker_type: TilePickerType::Default,
            model_constraint_algorithm: ModelConstraintAlgorithm::Default,
            weight_set_by_index: Arc::new(Mutex::new(Box::new(DummyTopoArray::new()))),
            weight_sets: std::collections::HashMap::new(),
            clean_tiles: Arc::new(Mutex::new(Box::new(DummyTopoArray::new()))),
            index_order: Vec::new(),
            memoize_indices: false,
        };

        Self::with_options(tile_model, topology, options)
    }

    /// Creates a new TilePropagator with the given options, and returns a reference to it under an RcRefCell.
    /// TilePropagator must be under an RcRefCell, as its numerous dependencies require a reference
    /// to itself.
    pub fn with_options(
        mut tile_model: Box<dyn TileModel<T>>,
        topology: T,
        options: TilePropagatorOptions<i32, T>,
    ) -> Result<Self, String> {
        let tile_model_mapping = tile_model.get_tile_model_mapping(&topology)?;
        let pattern_model = tile_model_mapping.pattern_model.clone();

        // WavePropagator
        let wave_constraints = Self::convert_constraints(&options.constraints)?;
        let (index_picker, pattern_picker) = Self::make_pickers(&options, &tile_model_mapping)?;
        let backtrack_policy = Self::make_backtrack_policy(&options)?;

        let wave_propagator_options = WavePropagatorOptions {
            backtrack_policy,
            max_backtrack_depth: options.max_backtrack_depth,
            random_double: Some(options.random_double),
            constraints: Some(wave_constraints),
            index_picker: Some(index_picker),
            pattern_picker: Some(pattern_picker),
            clear: false,
            model_constraint_algorithm: options.model_constraint_algorithm,
        };

        let state = TilePropagatorState {
            // wave_propagator,
            topology,
            tile_model,
            tile_model_mapping,
        };

        let mut wave_propagator = WavePropagator::new(
            pattern_model,
            wave_propagator_options,
            &state,
        )?;

        wave_propagator.clear(&state)?;

        Ok(Self {
            wave_propagator,
            state,
        })
    }

    fn make_backtrack_policy(options: &TilePropagatorOptions<i32, T>) -> Result<Option<Box<dyn BacktrackPolicy<T>>>, String> {
        match options.backtrack {
            BacktrackType::None => Ok(None),
            BacktrackType::Backtrack => Ok(Some(Box::new(ConstantBacktrackPolicy::new(1)))),
            BacktrackType::Backjump => Ok(Some(Box::new(PatienceBackjumpPolicy::new()))),
        }
    }

    fn make_pickers(
        options: &TilePropagatorOptions<i32, T>,
        tile_model_mapping: &TileModelMapping<T>,
    ) -> Result<(Box<dyn SuperTracker<T>>, Box<dyn SuperTracker<T>>), String> {
        let connected_constraint = options.constraints
            .iter()
            .find(|c| {
                // Check if the constraint is a ConnectedConstraint
                // This is a simplified check - will need to implement proper type checking
                false // For now, assume no connected constraints
            });

        let connected_pick_heuristic = connected_constraint.is_some();

        if connected_pick_heuristic {
            match options.index_picker_type {
                IndexPickerType::Default | IndexPickerType::MinEntropy | IndexPickerType::Ordered => {},
                _ => return Err(format!("Connected Pick Heuristic is incompatible with the selected IndexPickerType {:?}", options.index_picker_type)),
            }
        }

        let mut index_picker: Option<Box<dyn SuperTracker<T>>> = None;
        let mut pattern_picker: Option<Box<dyn SuperTracker<T>>> = None;

        match options.index_picker_type {
            IndexPickerType::Ordered => {
                todo!("implement");
                // if !options.index_order.is_empty() {
                //     println!("OrderedIndexPicker"); // TODO: Remove
                //     index_picker = Some(Arc::new(Mutex::new(Box::new(OrderedIndexPicker::new(options.index_order.clone())))));
                // } else {
                //     println!("SimpleOrderedIndexPicker"); // TODO: Remove
                //     let inner_picker = SimpleOrderedIndexPicker::new();
                //     let inner_picker = Self::move_out(inner_picker);
                //     index_picker = Some(Arc::new(Mutex::new(Box::new(inner_picker))));
                // }
            },
            IndexPickerType::ArrayPriorityMinEntropy => {
                todo!("implement");
                // if options.weight_set_by_index.lock().is_err() {
                //     return Err("Expected WeightSetByIndex and WeightSets to be set".to_string());
                // }
                // if options.tile_picker_type != TilePickerType::ArrayPriority && options.tile_picker_type != TilePickerType::Default {
                //     return Err("ArrayPriorityMinEntropy only works with Default tile picker".to_string());
                // }
                //
                // let weight_set_collection = WeightSetCollection::new(
                //     options.weight_set_by_index.clone(),
                //     options.weight_sets.clone(),
                //     tile_model_mapping.clone(),
                // );
                // let entropy_tracker = Arc::new(Mutex::new(ArrayPriorityEntropyTracker::new(weight_set_collection)));
                //
                // println!("ArrayPriorityEntropyTracker"); // TODO: Remove
                // index_picker = Some(to_index_picker(entropy_tracker.clone()));
                // pattern_picker = Some(to_pattern_picker(entropy_tracker.clone()));
            },
            IndexPickerType::MinEntropy => {
                // println!("EntropyTracker"); // TODO: Remove
                let ip = EntropyTracker::new();
                index_picker = Some(Box::new(ip) as Box<dyn SuperTracker<T>>);
            },
            IndexPickerType::Default | IndexPickerType::HeapMinEntropy => {
                // println!("HeapEntropyTracker"); // TODO: Remove
                let ip = HeapEntropyTracker::new_empty();
                index_picker = Some(Box::new(ip) as Box<dyn SuperTracker<T>>);
            },
            IndexPickerType::Dirty => {
                todo!("implement");
                // let mapping = ctx.tile_model_mappings().get(tile_model_mapping).unwrap();
                // if mapping.borrow().tile_coord_to_pattern_coord_index_and_offset.is_some() {
                //     return Err("Dirty index picker not supported with overlapping models".to_string());
                // }
                // let clean_tiles = options.clean_tiles.lock().unwrap();
                // let clean_patterns = clean_tiles.as_ref(); // Convert to pattern representation
                // let dirty_index_picker: Rc<RefCell<DirtyIndexPicker<T, SimpleOrderedIndexPicker>>> = DirtyIndexPicker::new(
                //     SimpleOrderedIndexPicker::default(),
                //     // This is a simplified conversion - you'll need proper tile-to-pattern mapping
                //     Box::new(DummyTopoArray::new()),
                // );
                // println!("DirtyIndexPicker"); // TODO: Remove
                //
                // index_picker = Some(to_index_picker(dirty_index_picker.clone()));
            },
        }

        if pattern_picker.is_none() {
            match options.tile_picker_type {
                TilePickerType::Default | TilePickerType::Weighted => {
                    // println!("WeightedRandomPatternPicker"); // TODO: Remove
                    let pp = WeightedRandomPatternPicker::new();
                    pattern_picker = Some(Box::new(pp) as Box<dyn SuperTracker<T>>);
                },
                TilePickerType::Ordered => {
                    // println!("SimplePatternPicker"); // TODO: Remove
                    todo!("implement");
                    // pattern_picker = Some(to_pattern_picker(SimplePatternPicker::new()));
                },
                TilePickerType::ArrayPriority => {
                    todo!("implement");
                    // let weight_set_collection = WeightSetCollection::new(
                    //     options.weight_set_by_index.clone(),
                    //     options.weight_sets.clone(),
                    //     tile_model_mapping.clone(),
                    // );
                    // let pp = ArrayPriorityEntropyTracker::new(weight_set_collection);
                    // let pp = Arc::new(Mutex::new(pp));
                    // println!("ArrayPriorityEntropyTracker"); // TODO: Remove
                    // pattern_picker = Some(to_pattern_picker(pp));
                },
            }
        }

        if connected_pick_heuristic {
            // Apply connected constraint heuristic to index picker
            // This is simplified - will need to implement the actual heuristic wrapper
        }

        if options.memoize_indices {
            if let Some(picker) = index_picker {
                // println!("MemoizeIndexPicker"); // TODO: Remove
                todo!("implement");
                // let mip = MemoizeIndexPicker::new(picker.clone());
                // index_picker = Some(to_index_picker(mip));
            }
        }

        if index_picker.is_none() {
            return Err("No index picker was created".to_string());
        }
        if pattern_picker.is_none() {
            return Err("No pattern picker was created".to_string());
        }

        Ok((
            index_picker.unwrap(),
            pattern_picker.unwrap(),
        ))
    }

    fn convert_constraints(constraints: &Vec<Box<dyn TileConstraint<T>>>) -> Result<Vec<WaveConstraint>, String> {
        // println!("Have {} constraints", constraints.len());
        return Ok(Vec::new());
        todo!("convert constraints");
        // let mut wave_constraints = Vec::new();
        //
        // for constraint in constraints {
        //     let adaptor = TileConstraintAdaptor::new(
        //         constraint.clone(),
        //         self_ref.clone(),
        //     );
        //     wave_constraints.push(Box::new(adaptor) as Box<dyn WaveConstraint>);
        // }
        //
        // Ok(wave_constraints)
    }

    /// Repeatedly Steps until the status is Decided or Contradiction.
    pub fn run(&mut self) -> Result<Resolution, String> {
        self.wave_propagator.run(&self.state.tile_model_mapping, self.state.get_topology())
    }

    /// Converts the generated results to an array of values.
    pub fn to_value_array(&self) -> Result<Box<dyn TopoArray<TileVisual, T>>, String> {
        self.to_value_array_with_defaults()
    }

    /// Converts the generated results to an array of values with defaults.
    pub fn to_value_array_with_defaults(
        &self,
    ) -> Result<Box<dyn TopoArray<TileVisual, T>>, String> {
        let index_count = self.state.topology.index_count();
        let mut values = Vec::with_capacity(index_count);

        for i in 0..index_count {
            values.push(self.get_value_with_defaults(i)
                .unwrap_or_else(|| TileVisual::Default));
        }

        // Create a simple 1D topology array wrapper
        Ok(Box::new(TopoArray1D::new(values, self.state.topology.clone())))
        // Ok(Box::new(SimpleTopoArray::new(values, self.topology.clone())))
    }

    /// Gets the value of a Tile that has been decided at a given index with defaults.
    pub fn get_value_with_defaults(&self, index: usize) -> Option<TileVisual> {
        let (pattern_index, o) = self.state.tile_model_mapping
            .get_tile_coord_to_pattern_coord_by_index(index);
        let pattern = self.wave_propagator.get_decided_pattern(pattern_index)?;

        match pattern as i8 {
            -1 => None, // Resolution::Undecided
            -2 => None, // Resolution::Contradiction
            _ => {
                if let Some(patterns_to_tiles) = self.state.tile_model_mapping.patterns_to_tiles_by_offset.get(&o) {
                    if let Some(tile_id) = patterns_to_tiles.get(&(pattern as usize)) {
                        let tile = self.state.tile_model.get_tile(tile_id.0);
                        let mut visual: Option<TileVisual> = None;
                        if tile.is_some() {
                            visual = Some(tile.unwrap().get_value().clone());
                        }
                        visual
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
}

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

impl<T: Clone + 'static, TopoT: Topology + Clone + 'static> TopoArray<T, TopoT> for DummyTopoArray<T> {
    fn topology(&self) -> Option<&TopoT> {
        todo!()
    }

    fn get_coord(&self, x: usize, y: usize, z: usize) -> Result<&T, TopologyError> {
        todo!()
    }

    fn get_index(&self, index: usize) -> Result<&T, TopologyError> {
        todo!()
    }

    fn get_value_from_index(&self, index: usize) -> Option<&T> {
        todo!()
    }

    fn get_value_from_coord(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        todo!()
    }

    fn get_id_from_index(&self, index: usize) -> Option<usize> {
        todo!()
    }

    fn get_id_from_coord(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        todo!()
    }

    fn clone_box(&self) -> Option<Box<dyn TopoArray<T, TopoT>>> {
        todo!()
    }
}

pub struct AsIndexPicker<T: Topology + Clone, R: IndexPicker<T> + 'static>(pub Rc<RefCell<R>>, PhantomData<T>);
impl<T: Topology + Clone, R: IndexPicker<T>> IndexPicker<T> for AsIndexPicker<T, R> {
    fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), String> {
        self.0.borrow_mut().init(wave_propagator_state, topology)
    }

    fn get_random_index(&mut self, wave_propagator_state: &WavePropagatorState<T>, tile_model_mapping: &TileModelMapping<T>) -> Option<i32> {
        self.0.borrow_mut().get_random_index(wave_propagator_state, tile_model_mapping)
    }

    fn as_super_tracker(&self) -> Option<&dyn SuperTracker<T>> {
        None
    }

    fn as_super_tracker_mut(&mut self) -> Option<&mut dyn SuperTracker<T>> {
        None
    }
}
pub fn to_index_picker<T: Topology + Clone + 'static, R: IndexPicker<T> +'static>(index_picker: Rc<RefCell<R>>) -> Rc<RefCell<dyn IndexPicker<T>>> {
    Rc::new(RefCell::new(AsIndexPicker(index_picker.clone(), PhantomData)))
}