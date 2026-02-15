use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::procedural_generation::debroglie::context::{Context, TopologyId};
use crate::procedural_generation::debroglie::topology::grid_topology::GridTopology;
use crate::procedural_generation::debroglie::topology::topo_array::TopoArray;
use crate::procedural_generation::debroglie::topology::topology::{Topology, TopologyError};

/// A 2D array with potentially different row lengths, coupled with a topology
#[derive(Debug, Clone)]
pub struct RaggedTopoArray2D<T> {
    values: Vec<Vec<T>>,
    topology: TopologyId,
}

impl<T> RaggedTopoArray2D<T> {
    /// Create a new RaggedTopoArray2D from a jagged array and periodicity.
    /// This also creates its own output topology and stores that in the Context.
    pub fn new(ctx: &Context<GridTopology>, values: Vec<Vec<T>>, periodic: bool) -> Self {
        let height = values.len();
        let width = values.iter().map(|row| row.len()).max().unwrap_or(0);

        // Create the output topology
        let topology = GridTopology::new_2d(width, height, periodic);
        let topology = ctx.topologies().add(topology);

        Self { values, topology }
    }

    /// Create a new RaggedTopoArray2D from a jagged array and explicit topology
    pub fn with_topology<TopologyT>(values: Vec<Vec<T>>, topology: TopologyId) -> RaggedTopoArray2DGeneric<T, TopologyT>
    where
        TopologyT: Topology + Clone,
    {
        RaggedTopoArray2DGeneric::new(values, topology)
    }

    /// Get the width of the widest row
    pub fn max_width(&self) -> usize {
        self.values.iter().map(|row| row.len()).max().unwrap_or(0)
    }

    /// Get the number of rows
    pub fn height(&self) -> usize {
        self.values.len()
    }

    /// Get the width of a specific row
    pub fn row_width(&self, y: usize) -> Option<usize> {
        self.values.get(y).map(|row| row.len())
    }

    /// Check if coordinates are valid for the ragged array
    pub fn is_valid_coord(&self, x: usize, y: usize) -> bool {
        if y >= self.values.len() {
            return false;
        }
        x < self.values[y].len()
    }
}

impl<T> TopoArray<T, GridTopology> for RaggedTopoArray2D<T> {
    fn topology(&self, ctx: &Context<GridTopology>) -> Option<Rc<RefCell<GridTopology>>> {
        ctx.topologies().clone().get(self.topology)
    }

    fn get_coord(&self, _ctx: &Context<GridTopology>, x: usize, y: usize, _z: usize) -> Result<&T, TopologyError> {
        if y >= self.values.len() {
            return Err(TopologyError::CoordinateOutOfBounds { x, y, z: 0 });
        }

        self.values[y].get(x).ok_or(TopologyError::CoordinateOutOfBounds { x, y, z: 0 })
    }

    fn get_index(&self, ctx: &Context<GridTopology>, index: usize) -> Result<&T, TopologyError> {
        let result = self
            .topology(ctx)
            .ok_or(TopologyError::Other("unable to get topology".to_string()))
            ?.borrow()
            .get_coord(index);
        if result.is_err() {
            return Err(TopologyError::Other(format!("GridTopologyError: {}", result.unwrap_err().to_string())));
        }
        let (x, y, z) = result.unwrap();
        self.get_coord(ctx, x, y, z)
    }
}

/// Generic version that works with any topology type
#[derive(Debug, Clone)]
pub struct RaggedTopoArray2DGeneric<T, TopologyT>
where
    TopologyT: Topology + Clone,
{
    values: Vec<Vec<T>>,
    topology: TopologyId,

    _phantom: PhantomData<TopologyT>,
}

impl<T, TopologyT> RaggedTopoArray2DGeneric<T, TopologyT>
where
    TopologyT: Topology + Clone,
{
    /// Create a new RaggedTopoArray2DGeneric from a jagged array and topology
    pub fn new(values: Vec<Vec<T>>, topology: TopologyId) -> Self {
        Self { values, topology, _phantom: PhantomData }
    }

    /// Get the width of the widest row
    pub fn max_width(&self) -> usize {
        self.values.iter().map(|row| row.len()).max().unwrap_or(0)
    }

    /// Get the number of rows
    pub fn height(&self) -> usize {
        self.values.len()
    }

    /// Get the width of a specific row
    pub fn row_width(&self, y: usize) -> Option<usize> {
        self.values.get(y).map(|row| row.len())
    }

    /// Check if coordinates are valid for the ragged array
    pub fn is_valid_coord(&self, x: usize, y: usize) -> bool {
        if y >= self.values.len() {
            return false;
        }
        x < self.values[y].len()
    }
}

impl<T, TopologyT> TopoArray<T, TopologyT> for RaggedTopoArray2DGeneric<T, TopologyT>
where
    TopologyT: Topology + Clone,
{

    fn topology(&self, ctx: &Context<TopologyT>) -> Option<Rc<RefCell<TopologyT>>> {
        ctx.topologies().clone().get(self.topology)
    }

    fn get_coord(&self, _ctx: &Context<TopologyT>, x: usize, y: usize, _z: usize) -> Result<&T, TopologyError> {
        if y >= self.values.len() {
            return Err(TopologyError::CoordinateOutOfBounds { x, y, z: 0 });
        }

        self.values[y].get(x).ok_or(TopologyError::CoordinateOutOfBounds { x, y, z: 0 })
    }

    fn get_index(&self, ctx: &Context<TopologyT>, index: usize) -> Result<&T, TopologyError> {
        let (x, y, z) = self
            .topology(ctx)
            .ok_or(TopologyError::Other("unable to get topology".to_string()))
            ?.borrow()
            .get_coord(index)?;
        self.get_coord(ctx, x, y, z)
    }
}