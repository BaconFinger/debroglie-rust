use std::rc::Rc;
use crate::procedural_generation::debroglie::context::{Context, TrackerId, WavePropagatorId};
use crate::procedural_generation::debroglie::topology::topology::Topology;
use crate::procedural_generation::debroglie::wfc::wave_propagator::WavePropagator;

pub trait PatternPicker<T: Topology + Clone> {
    fn init(&mut self, wave_propagator: &WavePropagator<T>) -> Result<(), String>;
    fn get_random_possible_pattern_at(&mut self, ctx: &Context<T>, index: usize, random_double: Rc< dyn Fn() -> f64>) -> Option<usize>;
    // fn set_self_ref(&mut self, self_ref: TrackerId);
    // fn add_self(self, ctx: &Context<T>) -> TrackerId;
}