use std::rc::Rc;
use crate::refactor::topology::topology::Topology;
use crate::refactor::wfc::wave_propagator::{WavePropagator, WavePropagatorState};

pub trait PatternPicker<T: Topology + Clone> {
    fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), String>;
    fn get_random_possible_pattern_at(&mut self, index: usize, random_double: Rc< dyn Fn() -> f64>) -> Option<usize>;
    // fn set_self_ref(&mut self, self_ref: TrackerId);
    // fn add_self(self, ctx: &Context<T>) -> TrackerId;
}