use crate::procedural_generation::debroglie::context::{Context, TilePropagatorId};
use crate::procedural_generation::debroglie::topology::topology::Topology;

/// Interface for specifying non-local constraints to be respected during generation.
pub trait TileConstraint<T>
where T: Topology + Clone
{

    /// Called once when the propagator first initializes.
    /// The propagator to constrain
    fn init(&mut self, ctx: &Context<T>, propagator: TilePropagatorId);

    /// Called frequently during generation to help maintain the constraint.
    /// The propagator to constrain
    fn check(&mut self, ctx: &Context<T>, propagator: TilePropagatorId);
}