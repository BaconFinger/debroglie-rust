use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::procedural_generation::debroglie::context::{Context, TopologyId};
use crate::procedural_generation::debroglie::topology::topo_array::TopoArray;
use crate::procedural_generation::debroglie::topology::topology::{Topology, TopologyError};

/// A 1D topological array implementation
pub struct TopoArray1D<T, TopologyT: Topology + Clone> {
    values: Vec<T>,
    topology: TopologyId,

    _phantom_data: PhantomData<TopologyT>
}

impl<T, TopologyT: Topology + Clone> TopoArray1D<T, TopologyT> {
    /// Creates a new TopoArray1D with the given values and topology
    pub fn new(values: Vec<T>, topology: TopologyId) -> Self {
        Self { values, topology, _phantom_data: PhantomData }
    }
}

impl<T, TopologyT: Topology + Clone> TopoArray<T, TopologyT> for TopoArray1D<T, TopologyT> {
    /// Gets the topology associated with this array
    fn topology(&self, ctx: &Context<TopologyT>) -> Option<Rc<RefCell<TopologyT>>> {
        ctx.topologies().get(self.topology)
    }

    /// Gets the value at the specified index
    fn get_index(&self, _ctx: &Context<TopologyT>, index: usize) -> Result<&T, TopologyError> {
        self.values.get(index)
            .ok_or(TopologyError::IndexOutOfBounds {
                index,
                max: self.values.len()
            })
    }
}