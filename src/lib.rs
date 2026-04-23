#![allow(unused_variables, dead_code)] // TODO: Reconsider this once we've ported most everything.
pub mod tile;

// #[cfg(test)]
// mod full_test;
pub mod constraints;
pub mod heap;
pub mod models;
pub mod point;
pub mod resolution;
pub mod shared_mut_heap;
pub mod tile_propagator;
pub mod tile_propagator_options;
pub mod tile_propagator_tile_set;
pub mod topology;
pub mod trackers;
pub mod trait_error;
pub mod wfc;
mod rot;

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
