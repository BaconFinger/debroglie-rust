use std::cell::{RefCell};
use std::rc::Rc;
use slotmap::{new_key_type, Key, SlotMap};
use crate::constraints::tile_constraint::TileConstraint;
use crate::models::adjacent_model::AdjacentModel;
use crate::models::tile_model::TileModel;
use crate::models::tile_model_mapping::TileModelMapping;
use crate::tile::Tile;
use crate::tile_propagator::TilePropagator;
use crate::topology::grid_topology::GridTopology;
use crate::topology::topology::Topology;
use crate::trackers::tracker::{ChoiceObserver, SuperTracker};
use crate::wfc::backtrack_policy::BacktrackPolicy;
use crate::wfc::wave::Wave;
use crate::wfc::wave_propagator::{WavePropagator};

new_key_type! {
    pub struct TileId;
    pub struct ModelId;
    pub struct TileModelMappingId;
    pub struct GridTopologyId;
    pub struct TilePropagatorId;
    pub struct WavePropagatorId;
    pub struct TopologyId;
    pub struct ConstraintId;
    pub struct WaveId;
    pub struct WaveConstraintId;
    pub struct TrackerId;
    pub struct ChoiceObserverId;
    // pub struct IndexPickerId;
    // pub struct PatternPickerId;
    pub struct BacktrackPolicyId;
}

// Just a textbook example of RcRefCell (read: interior mutability).
// The Rc to ContextStorage is normally read-only, but because the data in the ContextStorage
// is in a RefCell, it can be mutated without having to explicitly use "mut" everywhere.

/// Context stores and owns **all** data which is shared between classes.
/// Some classes with tighter coupling ([`HeapEntropyTracker`] and [`SharedMutHeap`]) might
/// share data in a more traditional, Rc<RefCell<>> way.
/// All classes should store the SlotMap handle instead of references to their dependencies, and
/// use this Context object to get an RcRefCell reference to the actual dependency.
/// The original library was written in C#, so this is, perhaps, a ham-fisted attempt to mimic
/// the code there.
pub struct Context<T: Topology + Clone + 'static> {
    tiles_inner: Rc<ContextStorage<TileId, Tile>>,
    grid_models_inner: Rc<GridModels>,
    grid_topologies_inner: Rc<ContextStorage<GridTopologyId, GridTopology>>,
    topology_inner: Rc<TopologyStorage<T>>,
    tile_propagator_inner: Rc<ContextStorage<TilePropagatorId, TilePropagator<T>>>,
    wave_propagator_inner: Rc<ContextStorage<WavePropagatorId, WavePropagator<T>>>,
    tile_model_mapping_inner: Rc<ContextStorage<TileModelMappingId, TileModelMapping<T>>>,
    wave_inner: Rc<ContextStorage<WaveId, Wave>>,

    // Traits

    models_inner: Rc<TraitContextStorage<ModelId, Rc<RefCell<dyn TileModel<T>>>>>,
    index_picker_inner: Rc<TraitContextStorage<TrackerId, Rc<RefCell<dyn SuperTracker<T>>>>>,
    pattern_picker_inner: Rc<TraitContextStorage<TrackerId, Rc<RefCell<dyn SuperTracker<T>>>>>,
    choice_observers_inner: Rc<TraitContextStorage<ChoiceObserverId, Rc<RefCell<dyn ChoiceObserver>>>>,
    trackers_inner: Rc<TraitContextStorage<TrackerId, Rc<RefCell<dyn SuperTracker<T>>>>>,
    backtrack_policy_inner: Rc<TraitContextStorage<BacktrackPolicyId, Rc<RefCell<dyn BacktrackPolicy<T>>>>>,
    tile_constraint_inner: Rc<TraitContextStorage<ConstraintId, Rc<RefCell<dyn TileConstraint<T>>>>>
}
impl<T> Context<T> where T: Topology + Clone + 'static {
    pub fn new() -> Self {
        Self {
            tiles_inner: Rc::new(ContextStorage::new()),
            models_inner: Rc::new(TraitContextStorage::new()),
            grid_models_inner: Rc::new(GridModels::new()),
            grid_topologies_inner: Rc::new(ContextStorage::new()),
            topology_inner: Rc::new(TopologyStorage::new()),
            tile_propagator_inner: Rc::new(ContextStorage::new()),
            wave_propagator_inner: Rc::new(ContextStorage::new()),
            tile_model_mapping_inner: Rc::new(ContextStorage::new()),
            index_picker_inner: Rc::new(TraitContextStorage::new()),
            pattern_picker_inner: Rc::new(TraitContextStorage::new()),
            wave_inner: Rc::new(ContextStorage::new()),
            trackers_inner: Rc::new(TraitContextStorage::new()),
            backtrack_policy_inner: Rc::new(TraitContextStorage::new()),
            choice_observers_inner: Rc::new(TraitContextStorage::new()),
            tile_constraint_inner: Rc::new(TraitContextStorage::new()),
        }
    }

    pub fn tiles(&self) -> Rc<ContextStorage<TileId, Tile>> {
        self.tiles_inner.clone()
    }

    pub fn models(&self) -> Rc<TraitContextStorage<ModelId, Rc<RefCell<dyn TileModel<T>>>>> {
        self.models_inner.clone()
    }

    pub fn grid_models(&self) -> Rc<GridModels> {
        self.grid_models_inner.clone()
    }

    pub fn grid_topologies(&self) -> Rc<ContextStorage<GridTopologyId, GridTopology>> {
        self.grid_topologies_inner.clone()
    }

    pub fn topologies(&self) -> Rc<TopologyStorage<T>> {
        self.topology_inner.clone()
    }

    pub fn tile_propagators(&self) -> Rc<ContextStorage<TilePropagatorId, TilePropagator<T>>> {
        self.tile_propagator_inner.clone()
    }

    pub fn wave_propagators(&self) -> Rc<ContextStorage<WavePropagatorId, WavePropagator<T>>> {
        self.wave_propagator_inner.clone()
    }

    pub fn tile_model_mappings(&self) -> Rc<ContextStorage<TileModelMappingId, TileModelMapping<T>>> {
        self.tile_model_mapping_inner.clone()
    }

    pub fn index_pickers(&self) -> Rc<TraitContextStorage<TrackerId, Rc<RefCell<dyn SuperTracker<T>>>>> {
        self.index_picker_inner.clone()
    }

    pub fn pattern_pickers(&self) -> Rc<TraitContextStorage<TrackerId, Rc<RefCell<dyn SuperTracker<T>>>>> {
        self.pattern_picker_inner.clone()
    }

    pub fn wave(&self) -> Rc<ContextStorage<WaveId, Wave>> {
        self.wave_inner.clone()
    }

    pub fn trackers(&self) -> Rc<TraitContextStorage<TrackerId, Rc<RefCell<dyn SuperTracker<T>>>>> {
        self.trackers_inner.clone()
    }

    pub fn backtrack_policy(&self) -> Rc<TraitContextStorage<BacktrackPolicyId, Rc<RefCell<dyn BacktrackPolicy<T>>>>> {
        self.backtrack_policy_inner.clone()
    }

    pub fn choice_observers(&self) -> Rc<TraitContextStorage<ChoiceObserverId, Rc<RefCell<dyn ChoiceObserver>>>> {
        self.choice_observers_inner.clone()
    }

    pub fn tile_constraints(&self) -> Rc<TraitContextStorage<ConstraintId, Rc<RefCell<dyn TileConstraint<T>>>>> {
        self.tile_constraint_inner.clone()
    }
}

pub struct GridModels {
    data: RefCell<SlotMap<ModelId, Rc<RefCell<dyn TileModel<GridTopology>>>>>
}

impl GridModels where {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(SlotMap::with_key())
        }
    }

    pub fn add_adjacent_model(&self, model: AdjacentModel) -> ModelId {
        self.data.borrow_mut().insert(Rc::new(RefCell::new(model)))
    }

    pub fn get(&self, id: ModelId) -> Option<Rc<RefCell<dyn TileModel<GridTopology>>>> {
        let temp = self.data.borrow();
        let val = temp.get(id);
        match val {
            Some(val) => Some(val.clone()),
            None => None
        }
    }
}

pub struct ContextStorage<TId: Key, T> {
    data: RefCell<SlotMap<TId, Rc<RefCell<T>>>>
}

impl<TId: Key, T> ContextStorage<TId, T> {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(SlotMap::with_key())
        }
    }

    pub fn add(&self, element: T) -> TId {
        self.data.borrow_mut().insert(Rc::new(RefCell::new(element)))
    }

    pub fn add_vec(&self, elements: Vec<T>) -> Vec<TId> {
        let mut ids = Vec::new();
        for element in elements {
            ids.push(self.add(element));
        }
        ids
    }

    pub fn add_vec_2d(&self, elements: Vec<Vec<T>>) -> Vec<Vec<TId>> {
        let mut ids = Vec::new();
        for element in elements {
            ids.push(self.add_vec(element));
        }
        ids
    }

    pub fn add_vec_3d(&self, elements: Vec<Vec<Vec<T>>>) -> Vec<Vec<Vec<TId>>> {
        let mut ids = Vec::new();
        for element in elements {
            ids.push(self.add_vec_2d(element));
        }
        ids
    }

    pub fn get(&self, id: TId) -> Option<Rc<RefCell<T>>> {
        let temp = self.data.borrow();
        let val = temp.get(id);
        match val {
            Some(val) => Some(val.clone()),
            None => None
        }
    }
}

/// TraitContextStorage is a specialization of ContextStorage for traits, since they have to be wrapped in an RcRefCell.
/// This means that the get method does not force the value into an RcRefCell, so be careful!
pub struct TraitContextStorage<TId: Key, T: Clone> {
    data: RefCell<SlotMap<TId, T>>
}

impl<TId: Key, T: Clone> TraitContextStorage<TId, T> {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(SlotMap::with_key())
        }
    }

    pub fn add(&self, element: T) -> TId {
        self.data.borrow_mut().insert(element)
    }

    pub fn add_vec(&self, elements: Vec<T>) -> Vec<TId> {
        let mut ids = Vec::new();
        for element in elements {
            ids.push(self.add(element));
        }
        ids
    }

    pub fn add_vec_2d(&self, elements: Vec<Vec<T>>) -> Vec<Vec<TId>> {
        let mut ids = Vec::new();
        for element in elements {
            ids.push(self.add_vec(element));
        }
        ids
    }

    pub fn add_vec_3d(&self, elements: Vec<Vec<Vec<T>>>) -> Vec<Vec<Vec<TId>>> {
        let mut ids = Vec::new();
        for element in elements {
            ids.push(self.add_vec_2d(element));
        }
        ids
    }

    pub fn get(&self, id: TId) -> Option<T> {
        let temp = self.data.borrow();
        let val = temp.get(id);
        match val {
            Some(val) => Some(val.clone()),
            None => None
        }
    }
}

pub struct TopologyStorage<T: Topology + Clone + 'static> {
    data: RefCell<SlotMap<TopologyId, Rc<RefCell<T>>>>
}

impl<T> TopologyStorage<T> where T: Topology + Clone + 'static {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(SlotMap::with_key())
        }
    }

    /// This also sets the TopologyId to the Topology.
    pub fn add(&self, element: T) -> TopologyId {
        let id = self.data.borrow_mut().insert(Rc::new(RefCell::new(element)));
        self.data.borrow_mut().get(id).unwrap().borrow_mut().set_key(id.clone()); // Safe since we just added it

        id
    }

    pub fn get(&self, id: TopologyId) -> Option<Rc<RefCell<T>>> {
        let temp = self.data.borrow();
        let val = temp.get(id);
        match val {
            Some(val) => Some(val.clone()),
            None => None
        }
    }
}

mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::context::{Context, TileId};
    use crate::tile::Tile;
    use crate::topology::grid_topology::GridTopology;

    #[test]
    fn test_it() {
        let ctx: Context<GridTopology> = Context::new();
        let value = '*';
        let tile = Tile::from_char(value);
        let handle = ctx.tiles().add(tile);

        let fetched_value = ctx.tiles().get(handle).and_then(|rc| {
            let t = rc.borrow();
            Some(t.get_value().clone())
        });

        assert_eq!(fetched_value.unwrap(), value);
    }

    #[test]
    fn breaking_maybe() {
        let ctx: Context<GridTopology> = Context::new();
        let value = '*';
        let tile = Tile::from_char(value.clone());
        let handle = ctx.tiles().add(tile);

        let fetched_tile = get_tile(&ctx, handle);
        let fetched_value = fetched_tile.and_then(|rc| {
            let t = rc.borrow();
            Some(t.get_value().clone())
        });

        assert_eq!(fetched_value.unwrap(), value);
    }

    fn get_tile(ctx: &Context<GridTopology>, id: TileId) -> Option<Rc<RefCell<Tile>>> {
        let fetched_tile = ctx.tiles().get(id);
        return fetched_tile;
    }
}