#![allow(unused_variables)]
pub mod context;
pub mod tile;

// #[cfg(test)]
// mod full_test;
pub mod topology;
pub mod models;
pub mod tile_propagator;
pub mod resolution;
pub mod wfc;
pub mod point;
pub mod tile_propagator_tile_set;
pub mod trackers;
pub mod tile_propagator_options;
pub mod constraints;
pub mod shared_mut_heap;
pub mod heap;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
