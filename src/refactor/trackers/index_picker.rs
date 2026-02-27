use std::rc::Rc;
use crate::refactor::topology::topology::Topology;
use crate::refactor::trackers::tracker::SuperTracker;
use crate::refactor::wfc::wave_propagator::{WavePropagator, WavePropagatorState};

pub trait IndexPicker<T: Topology + Clone> {
    fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), String>;
    fn get_random_index(&mut self, random_double: Rc<dyn Fn() -> f64>) -> Option<i32>;
    // fn set_self_ref(&mut self, self_ref: TrackerId);
    // fn add_self(self, ctx: &Context<T>) -> TrackerId;
}