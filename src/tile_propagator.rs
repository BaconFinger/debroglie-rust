use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use bevy::render::render_resource::encase::private::RuntimeSizedArray;
use bevy_rapier2d::na::DimAdd;
use crate::procedural_generation::debroglie::context::{ConstraintId, Context, ModelId, TileId, TileModelMappingId, TilePropagatorId, TopologyId, TrackerId, WaveConstraintId, WavePropagatorId};
use crate::procedural_generation::debroglie::models::tile_model::TileModel;
use crate::procedural_generation::debroglie::models::tile_model_mapping::TileModelMapping;
use crate::procedural_generation::debroglie::resolution::Resolution;
use crate::procedural_generation::debroglie::tile::{Tile, TileVisual};
use crate::procedural_generation::debroglie::tile_propagator_options::{BacktrackType, IndexPickerType, TilePickerType, TilePropagatorOptions};
use crate::procedural_generation::debroglie::topology::topo_array::TopoArray;
use crate::procedural_generation::debroglie::topology::topo_array_1d::TopoArray1D;
use crate::procedural_generation::debroglie::topology::topology::{Topology, TopologyError};
use crate::procedural_generation::debroglie::trackers::entropy_tracker::EntropyTracker;
use crate::procedural_generation::debroglie::trackers::heap_entropy_tracker::HeapEntropyTracker;
use crate::procedural_generation::debroglie::trackers::index_picker::IndexPicker;
use crate::procedural_generation::debroglie::trackers::pattern_picker::PatternPicker;
use crate::procedural_generation::debroglie::trackers::tracker::SuperTracker;
use crate::procedural_generation::debroglie::trackers::weighted_random_pattern_picker::WeightedRandomPatternPicker;
use crate::procedural_generation::debroglie::wfc::backtrack_policy::{BacktrackPolicy, ConstantBacktrackPolicy, PatienceBackjumpPolicy};
use crate::procedural_generation::debroglie::wfc::wave_propagator::{ModelConstraintAlgorithm, WavePropagator, WavePropagatorOptions};

/// TilePropagator is the main entrypoint to the DeBroglie library.
/// It takes a TileModel and an output Topology and generates
/// an output array using those parameters.
///
/// Implementation wise, this wraps a WavePropagator to do the majority of the work.
/// The only thing this class handles is conversion of tile objects into sets of patterns
/// and coordinate conversion.
pub struct TilePropagator<T>
where T: Topology + Clone + 'static
{
    wave_propagator: Option<WavePropagatorId>,
    topology: TopologyId,
    tile_model: ModelId,
    tile_model_mapping: TileModelMappingId,
    self_ref: Option<TilePropagatorId>,
    // wave_propagator: Option<Arc<Mutex<crate::procedural_generation::debroglie_scuffed::wfc::wave_propagator::WavePropagator>>>,
    // tile_model: Arc<Mutex<Box<dyn crate::procedural_generation::debroglie_scuffed::models::tile_model::TileModel<T>>>>,
    // tile_model_mapping: Arc<Mutex<crate::procedural_generation::debroglie_scuffed::models::tile_model_mapping::TileModelMapping<T>>>,
    // self_ref: Option<Arc<Mutex<Self>>>,

    phantom_data: PhantomData<T>
}

impl<T> TilePropagator<T>
where T: Topology + Clone + 'static
{
    pub fn new(
        ctx: &Context<T>,
        tile_model: ModelId,
        topology: TopologyId,
        backtrack: bool,
        constraints: Option<Vec<ConstraintId>>,
    ) -> Result<Rc<RefCell<Self>>, String> {
        let options = TilePropagatorOptions {
            backtrack: if backtrack { BacktrackType::Backtrack } else { BacktrackType::None },
            max_backtrack_depth: 0,
            constraints: constraints.unwrap_or(Vec::new()),
            // TODO: make customizable and seedable
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

        let self_id = Self::with_options(ctx, tile_model, topology, options)?;
        Ok(ctx.tile_propagators().get(self_id).ok_or("unable to get self")?.clone())
    }

    pub fn with_options(
        ctx: &Context<T>,
        tile_model: ModelId,
        topology: TopologyId,
        options: TilePropagatorOptions<i32, T>,
    ) -> Result<TilePropagatorId, String> {
        let model = ctx.models().get(tile_model).clone().ok_or("No tile model")?;
        let mut model_val = model.borrow_mut();
        let tile_model_mapping = model_val.get_tile_model_mapping(ctx, topology)?;
        let pattern_topology = tile_model_mapping.pattern_topology.as_ref()
            .ok_or("Pattern topology is required")?
            .clone();
        let pattern_model = tile_model_mapping.pattern_model.clone();
        let tile_model_mapping = ctx.tile_model_mappings().add(tile_model_mapping);

        let me = Self {
            wave_propagator: None,
            topology,
            tile_model,
            tile_model_mapping,
            self_ref: None,
            phantom_data: PhantomData,
        };
        let self_ref = ctx.tile_propagators().add(me);
        let me_ref = ctx.tile_propagators().get(self_ref).unwrap(); // Safe since we just added
        me_ref.borrow_mut().self_ref = Some(self_ref);

        let wave_constraints = Self::convert_constraints(ctx, &options.constraints, self_ref.clone())?;
        let (index_picker, pattern_picker) = Self::make_pickers(ctx, &options, tile_model_mapping.clone())?;
        let backtrack_policy = Self::make_backtrack_policy(&options)?;
        let backtrack_policy = if let Some(actual_bt_policy) = backtrack_policy {
            let id = ctx.backtrack_policy().add(actual_bt_policy);
            Some(id)
        } else {
            None
        };

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

        let mut wave_propagator = WavePropagator::new(
            ctx,
            pattern_model,
            pattern_topology.clone(),
            wave_propagator_options,
        )?;

        let wave_propagator_ref = ctx.wave_propagators().get(wave_propagator).ok_or("unable to get wave propagator")?;
        wave_propagator_ref.borrow_mut().clear(ctx)?;

        me_ref.borrow_mut().wave_propagator = Some(wave_propagator);

        Ok(self_ref)
    }

    fn make_backtrack_policy(options: &TilePropagatorOptions<i32, T>) -> Result<Option<Rc<RefCell<dyn BacktrackPolicy<T>>>>, String> {
        match options.backtrack {
            BacktrackType::None => Ok(None),
            BacktrackType::Backtrack => Ok(Some(Rc::new(RefCell::new(ConstantBacktrackPolicy::new(1))))),
            BacktrackType::Backjump => Ok(Some(Rc::new(RefCell::new(PatienceBackjumpPolicy::new())))),
        }
    }

    fn make_pickers(
        ctx: &Context<T>,
        options: &TilePropagatorOptions<i32, T>,
        tile_model_mapping: TileModelMappingId,
    ) -> Result<(TrackerId, TrackerId), String> {
        let connected_constraint = options.constraints
            .iter()
            .find(|c| {
                // Check if the constraint is a ConnectedConstraint
                // This is a simplified check - you may need to implement proper type checking
                false // For now, assume no connected constraints
            });

        let connected_pick_heuristic = connected_constraint.is_some();

        if connected_pick_heuristic {
            match options.index_picker_type {
                IndexPickerType::Default | IndexPickerType::MinEntropy | IndexPickerType::Ordered => {},
                _ => return Err(format!("Connected Pick Heuristic is incompatible with the selected IndexPickerType {:?}", options.index_picker_type)),
            }
        }

        // let mut index_picker: Option<Rc<RefCell<dyn IndexPicker<T>>>> = None;
        // let mut pattern_picker: Option<Rc<RefCell<dyn PatternPicker<T>>>> = None;
        let mut index_picker: Option<TrackerId> = None;
        let mut pattern_picker: Option<TrackerId> = None;

        match options.index_picker_type {
            IndexPickerType::Ordered => {
                panic!("NYI!");
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
                panic!("NYI");
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
                index_picker = Some(ip.add_self(ctx));
            },
            IndexPickerType::Default | IndexPickerType::HeapMinEntropy => {
                // println!("HeapEntropyTracker"); // TODO: Remove
                let ip = HeapEntropyTracker::init();
                index_picker = Some(ip.add_self(ctx));
            },
            IndexPickerType::Dirty => {
                panic!("NYI");
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
                    pattern_picker = Some(pp.add_self(ctx));
                },
                TilePickerType::Ordered => {
                    // println!("SimplePatternPicker"); // TODO: Remove
                    panic!("NYI");
                    todo!("implement");
                    // pattern_picker = Some(to_pattern_picker(SimplePatternPicker::new()));
                },
                TilePickerType::ArrayPriority => {
                    panic!("NYI");
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
            // This is simplified - you'll need to implement the actual heuristic wrapper
        }

        if options.memoize_indices {
            if let Some(picker) = index_picker {
                // println!("MemoizeIndexPicker"); // TODO: Remove
                panic!("NYI");
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

    fn convert_constraints(ctx: &Context<T>, constraints: &[ConstraintId], self_ref: TilePropagatorId) -> Result<Vec<WaveConstraintId>, String> {
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
    pub fn run(&self, ctx: &Context<T>) -> Result<Resolution, String> {
        self.get_wave_propagator(ctx).ok_or("unable to get wave propagator")?.borrow_mut().run(ctx)
    }

    /// Converts the generated results to an array of values.
    pub fn to_value_array(&self, ctx: &Context<T>) -> Result<Box<dyn TopoArray<TileVisual, T>>, String> {
        self.to_value_array_with_defaults(ctx)
    }

    /// Converts the generated results to an array of values with defaults.
    pub fn to_value_array_with_defaults(
        &self,
        ctx: &Context<T>,
    ) -> Result<Box<dyn TopoArray<TileVisual, T>>, String> {
        let topology = self.get_topology(ctx)
            .ok_or_else(|| format!("Topology {:?} not found", self.topology))?;
        let index_count = topology.borrow().index_count();
        let mut values = Vec::with_capacity(index_count);

        for i in 0..index_count {
            values.push(self.get_value_with_defaults(ctx, i)
                .unwrap_or_else(|| TileVisual::Default));
        }

        // Create a simple 1D topology array wrapper
        Ok(Box::new(TopoArray1D::new(values, self.topology.clone())))
        // Ok(Box::new(SimpleTopoArray::new(values, self.topology.clone())))
    }

    /// Gets the value of a Tile that has been decided at a given index with defaults.
    pub fn get_value_with_defaults(&self, ctx: &Context<T>, index: usize) -> Option<TileVisual> {
        let (pattern_index, o) = self.get_tile_model_mapping(ctx)
            ?.borrow()
            .get_tile_coord_to_pattern_coord_by_index(ctx, index);
        let pattern = self.get_wave_propagator(ctx)?.borrow().get_decided_pattern(ctx, pattern_index)?;

        match pattern as i8 {
            -1 => None, // Resolution::Undecided
            -2 => None, // Resolution::Contradiction
            _ => {
                if let Some(patterns_to_tiles) = self.get_tile_model_mapping(ctx)?.borrow().patterns_to_tiles_by_offset.get(&o) {
                    if let Some(tile) = patterns_to_tiles.get(&(pattern as usize)) {
                        let tile_ref = self.get_tile(ctx, tile.clone())?;
                        let tile = tile_ref.borrow();
                        Some(tile.get_value().clone())
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

// Getters
impl<T> TilePropagator<T>
where T: Topology + Clone + 'static
{
    fn get_topology(&self, ctx: &Context<T>) -> Option<Rc<RefCell<T>>> {
        ctx
            .topologies()
            .clone()
            .get(self.topology.clone())
    }

    fn get_tile_model_mapping(&self, ctx: &Context<T>) -> Option<Rc<RefCell<TileModelMapping<T>>>> {
        ctx
            .tile_model_mappings()
            .clone()
            .get(self.tile_model_mapping.clone())
    }

    fn get_wave_propagator(&self, ctx: &Context<T>) -> Option<Rc<RefCell<WavePropagator<T>>>> {
        ctx
            .wave_propagators()
            .clone()
            .get(self.wave_propagator?.clone())
    }

    fn get_tile(&self, ctx: &Context<T>, tile: TileId) -> Option<Rc<RefCell<Tile>>> {
        ctx
            .tiles()
            .clone()
            .get(tile)
    }

    fn get_model(&self, ctx: &Context<T>) -> Option<Rc<RefCell<dyn TileModel<T>>>> {
        ctx
            .models()
            .clone()
            .get(self.tile_model.clone())
    }
}

/// A dummy TopoArray implementation for cases where we need a placeholder
pub struct DummyTopoArray<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> DummyTopoArray<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Clone, TopoT: Topology + Clone> TopoArray<T, TopoT> for DummyTopoArray<T> {
    fn topology(&self, ctx: &Context<TopoT>) -> Option<Rc<RefCell<TopoT>>> {
        todo!()
    }

    fn get_index(&self, ctx: &Context<TopoT>, index: usize) -> Result<&T, TopologyError> {
        todo!()
    }
}

pub struct AsIndexPicker<T: Topology + Clone, R: IndexPicker<T> + 'static>(pub Rc<RefCell<R>>, PhantomData<T>);
impl<T: Topology + Clone, R: IndexPicker<T>> IndexPicker<T> for AsIndexPicker<T, R> {
    fn init(&mut self, ctx: &Context<T>, wave_propagator: &mut WavePropagator<T>) -> Result<(), String> {
        self.0.borrow_mut().init(ctx, wave_propagator)
    }

    fn get_random_index(&mut self, ctx: &Context<T>, random_double: Rc<dyn Fn() -> f64>) -> Option<i32> {
        self.0.borrow_mut().get_random_index(ctx, random_double)
    }

    // fn set_self_ref(&mut self, self_ref: TrackerId) {
    //     todo!()
    // }
    //
    // fn add_self(self, ctx: &Context<T>) -> TrackerId {
    //     todo!()
    // }
}
pub fn to_index_picker<T: Topology + Clone + 'static, R: IndexPicker<T> +'static>(index_picker: Rc<RefCell<R>>) -> Rc<RefCell<dyn IndexPicker<T>>> {
    Rc::new(RefCell::new(AsIndexPicker(index_picker.clone(), PhantomData)))
}