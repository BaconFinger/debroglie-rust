use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use crate::constraints::tile_constraint::TileConstraint;
use crate::tile::TileId;
use crate::tile_propagator::DummyTopoArray;
use crate::topology::topo_array::TopoArray;
use crate::topology::topology::Topology;
use crate::wfc::wave_propagator::ModelConstraintAlgorithm;

#[derive(Debug, Clone)]
pub struct PriorityAndWeight {
    pub weight: f64,
    pub priority: i32,
}

impl PriorityAndWeight {
    pub fn new(weight: f64, priority: i32) -> Self {
        Self { weight, priority }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IndexPickerType
{
    /// Use the most appropriate picker, usually MinEntropy
    Default,
    /// Pick the first available index.
    /// Uses IndexOrder if available, otherwise an arbitrary order.
    Ordered,
    /// Pick the index with the least entropy in the remaining tiles
    MinEntropy,
    /// As MinEntropy, but better optimized for large outputs
    HeapMinEntropy,
    /// Override frequencies on a per-index
    ArrayPriorityMinEntropy,
    /// Only pick indices that must deviate from a known clean value.
    /// This lets you regenerate an unknown subset of a much larger map,
    /// providing tiles are sufficiently stable.
    /// Experimental.
    Dirty,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TilePickerType
{
    /// Use the most appropriate picker, usually Weighted
    Default,
    /// Pick the first available tile.
    Ordered,
    /// Pick at random, based on frequencies supplied by the model
    Weighted,
    /// Use the provided weights.
    ArrayPriority,
}

#[derive(Debug, Clone)]
pub enum BacktrackType
{
    None,
    Backtrack,
    Backjump,
}

pub struct TilePropagatorOptions<V, T: Topology + Clone>
{
    pub backtrack: BacktrackType,

    /// Maximum number of steps to backtrack.
    /// 0 means disabled.
    pub max_backtrack_depth: i32,

    /// Extra constraints to control the generation process
    pub constraints: Vec<Box<dyn TileConstraint<T>>>,

    /// Source of randomness used by generation.
    /// A lot of randomness implementations that support seeding will mutate the underlying
    /// implementation in some way, so be mindful of this.
    pub random_double: Rc<dyn Fn() -> f64>,

    /// Controls which cells are selected during generation.
    pub index_picker_type: IndexPickerType,

    /// Controls which tiles are selected during generation.
    pub tile_picker_type: TilePickerType,

    /// Controls the algorithm used for enforcing the constraints of the model.
    pub model_constraint_algorithm: ModelConstraintAlgorithm,

    /// Overrides the weights set from the model, on a per-position basis.
    /// The integers correspond to entries in WeightSets
    /// Only used by <see cref="IndexPickerType.ArrayPriorityMinEntropy"/> and <see cref="TilePickerType.ArrayPriority"/>
    pub weight_set_by_index: Box<dyn TopoArray<V, T>>,

    /// The weights sets reference by WeightSetByIndex
    pub weight_sets: HashMap<i32, HashMap<TileId, PriorityAndWeight>>,

    /// Only used by <see cref="IndexPickerType.Dirty"/>
    pub clean_tiles: Box<dyn TopoArray<TileId, T>>,

    /// Only used by <see cref="IndexPickerType.Ordered"/>
    pub index_order: Vec<i32>,

    /// If true, the same indices will be retried after backtracking,
    /// otherwise a new choice of index will be made.
    pub memoize_indices: bool,
}

impl <V: Clone + 'static, T: Topology + Clone> Debug for TilePropagatorOptions<V, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<V: Clone + 'static, T: Topology + Clone + 'static> TilePropagatorOptions<V, T> {
    pub fn new(backtrack: bool, random_double: Option<Rc<dyn Fn() -> f64>>, constraints: Option<Vec<Box<dyn TileConstraint<T>>>>,) -> Self {
        Self {
            backtrack: if backtrack { BacktrackType::Backtrack } else { BacktrackType::None },
            max_backtrack_depth: 0,
            constraints: constraints.unwrap_or(Vec::new()),
            random_double: random_double.unwrap_or(Rc::new(|| {

                use std::hash::{Hash, Hasher};
                use std::time::{SystemTime, UNIX_EPOCH};

                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
                    .hash(&mut hasher);
                let hash = hasher.finish();
                (hash as f64) / (u64::MAX as f64)
            })),
            index_picker_type: IndexPickerType::Default,
            tile_picker_type: TilePickerType::Default,
            model_constraint_algorithm: ModelConstraintAlgorithm::Default,
            weight_set_by_index: Box::new(DummyTopoArray::new()),
            weight_sets: HashMap::new(),
            clean_tiles: Box::new(DummyTopoArray::new()),
            index_order: Vec::new(),
            memoize_indices: false,
        }
    }
}

mod tests {
    use crate::topology::grid_topology::GridTopology;
    use super::*;

    #[test]
    fn includes_default_random_func() {
        // Arrange
        let max_tries = 3;
        let mut last_random_double = 0.0;

        // Act
        let options: TilePropagatorOptions<i32, GridTopology> = TilePropagatorOptions::new(true, None, None);

        // Assert
        for i in 0..max_tries {
            if i >= max_tries - 1 {
                assert_eq!(false, true, "random func output has been the same for the last 3 tries.");
            }
            let output = options.random_double.clone()();
            if output != last_random_double {
                assert_eq!(true, true);
                break;
            }
            last_random_double = output;
        }
    }
}