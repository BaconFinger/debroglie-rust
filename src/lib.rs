mod context;
mod tile;

#[cfg(test)]
mod full_test;
mod topology;
mod models;
mod tile_propagator;
mod resolution;
mod wfc;
mod point;
mod tile_propagator_tile_set;
mod trackers;
mod tile_propagator_options;
mod constraints;
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
