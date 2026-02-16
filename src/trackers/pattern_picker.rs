use std::rc::Rc;
use crate::context::{Context};
use crate::topology::topology::Topology;
use crate::wfc::wave_propagator::WavePropagator;

pub trait PatternPicker<T: Topology + Clone> {
    fn init(&mut self, wave_propagator: &WavePropagator<T>) -> Result<(), String>;
    fn get_random_possible_pattern_at(&mut self, ctx: &Context<T>, index: usize, random_double: Rc< dyn Fn() -> f64>) -> Option<usize>;
    // fn set_self_ref(&mut self, self_ref: TrackerId);
    // fn add_self(self, ctx: &Context<T>) -> TrackerId;
}