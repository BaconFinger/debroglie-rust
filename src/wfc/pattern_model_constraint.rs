use crate::topology::topology::Topology;
use crate::wfc::wave_propagator::{WavePropagator, WavePropagatorState};

pub trait PatternModelConstraint<T: Topology + Clone> {
    fn do_ban(&mut self, index: usize, pattern: i32) ->  Result<(), String>;
    fn undo_ban(&mut self, index: usize, pattern: i32, topology: &T) ->  Result<(), String>;
    fn do_select(&mut self, index: usize, pattern: i32) ->  Result<(), String>;
    fn propagate(&mut self, topology: &T, wave_propagator: &mut WavePropagator<T>) -> Result<(), String>;

    /// This method will clear the internal state of the Constraint, then return the index and pattern
    /// that should be banned by the caller of this method.
    fn clear(&mut self, topology: &T, wave_propagator_state: &WavePropagatorState<T>) -> Result<Option<(usize, usize)>, String>;
}

// Placeholder pattern model constraint implementations
pub struct OneStepPatternModelConstraint;
impl<T: Topology + Clone> PatternModelConstraint<T> for OneStepPatternModelConstraint {
    fn do_ban(&mut self, index: usize, pattern: i32) -> Result<(), String> {
        todo!()
    }

    fn undo_ban(&mut self, index: usize, pattern: i32, topology: &T) -> Result<(), String> {
        todo!()
    }

    fn do_select(&mut self, index: usize, pattern: i32) -> Result<(), String> {
        todo!()
    }

    fn propagate(&mut self, topology: &T, wave_propagator: &mut WavePropagator<T>) -> Result<(), String> {
        todo!()
    }

    fn clear(&mut self, topology: &T, wave_propagator_state: &WavePropagatorState<T>) -> Result<Option<(usize, usize)>, String> {
        todo!()
    }
}

// pub struct Ac4PatternModelConstraint;
// impl PatternModelConstraint for Ac4PatternModelConstraint {
//     fn do_ban(&mut self, _index: usize, _pattern: i32) {}
//     fn undo_ban(&mut self, _index: usize, _pattern: i32) {}
//     fn do_select(&mut self, _index: usize, _pattern: i32) {}
//     fn propagate(&mut self) {}
//     fn clear(&mut self) {}
// }

pub struct Ac3PatternModelConstraint;
impl<T: Topology + Clone> PatternModelConstraint<T> for Ac3PatternModelConstraint {
    fn do_ban(&mut self, index: usize, pattern: i32) -> Result<(), String> {
        todo!()
    }

    fn undo_ban(&mut self, index: usize, pattern: i32, topology: &T) -> Result<(), String> {
        todo!()
    }

    fn do_select(&mut self, index: usize, pattern: i32) -> Result<(), String> {
        todo!()
    }

    fn propagate(&mut self, topology: &T, wave_propagator: &mut WavePropagator<T>) -> Result<(), String> {
        todo!()
    }

    fn clear(&mut self, topology: &T, wave_propagator_state: &WavePropagatorState<T>) -> Result<Option<(usize, usize)>, String> {
        todo!()
    }
}