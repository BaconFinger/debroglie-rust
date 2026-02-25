pub(crate) mod tile;

// #[cfg(test)]
// mod full_test;
pub(crate) mod topology;
pub(crate) mod models;
pub(crate) mod tile_propagator;
pub(crate) mod resolution;
pub(crate) mod wfc;
pub(crate) mod point;
pub(crate) mod tile_propagator_tile_set;
pub(crate) mod trackers;
pub(crate) mod tile_propagator_options;
pub(crate) mod constraints;
pub(crate) mod shared_mut_heap;

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
