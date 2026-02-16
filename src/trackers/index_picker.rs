use std::rc::Rc;
use crate::context::{Context};
use crate::topology::topology::Topology;
use crate::wfc::wave_propagator::WavePropagator;

pub trait IndexPicker<T: Topology + Clone> {
    fn init(&mut self, ctx: &Context<T>, wave_propagator: &mut WavePropagator<T>) -> Result<(), String>;
    fn get_random_index(&mut self, ctx: &Context<T>, random_double: Rc<dyn Fn() -> f64>) -> Option<i32>;
    // fn set_self_ref(&mut self, self_ref: TrackerId);
    // fn add_self(self, ctx: &Context<T>) -> TrackerId;
}