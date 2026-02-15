use crate::procedural_generation::debroglie::context::Context;
use crate::procedural_generation::debroglie::topology::topology::Topology;

pub trait PatternModelConstraint<T: Topology + Clone> {
    fn do_ban(&mut self, ctx: &Context<T>, index: usize, pattern: i32);
    fn undo_ban(&mut self, ctx: &Context<T>, index: usize, pattern: i32);
    fn do_select(&mut self, ctx: &Context<T>, index: usize, pattern: i32);
    fn propagate(&mut self, ctx: &Context<T>);
    fn clear(&mut self, ctx: &Context<T>);
}

// Placeholder pattern model constraint implementations
pub struct OneStepPatternModelConstraint;
impl<T: Topology + Clone> PatternModelConstraint<T> for OneStepPatternModelConstraint {
    fn do_ban(&mut self, ctx: &Context<T>, index: usize, pattern: i32) {
        todo!()
    }

    fn undo_ban(&mut self, ctx: &Context<T>, index: usize, pattern: i32) {
        todo!()
    }

    fn do_select(&mut self, ctx: &Context<T>, index: usize, pattern: i32) {
        todo!()
    }

    fn propagate(&mut self, ctx: &Context<T>) {
        todo!()
    }

    fn clear(&mut self, ctx: &Context<T>) {
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
    fn do_ban(&mut self, ctx: &Context<T>, index: usize, pattern: i32) {
        todo!()
    }

    fn undo_ban(&mut self, ctx: &Context<T>, index: usize, pattern: i32) {
        todo!()
    }

    fn do_select(&mut self, ctx: &Context<T>, index: usize, pattern: i32) {
        todo!()
    }

    fn propagate(&mut self, ctx: &Context<T>) {
        todo!()
    }

    fn clear(&mut self, ctx: &Context<T>) {
        todo!()
    }
}