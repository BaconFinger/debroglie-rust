use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use debroglie_rust::models::adjacent_model::AdjacentModel;
use debroglie_rust::resolution::Resolution;
use debroglie_rust::tile::{Tile, TileVisual, ToTile};
use debroglie_rust::tile_propagator::TilePropagator;
use debroglie_rust::tile_propagator_options::{BacktrackType, TilePropagatorOptions};
use debroglie_rust::topology::grid_topology::GridTopology;
use debroglie_rust::topology::ragged_topology_array_2d::RaggedTopoArray2D;
use debroglie_rust::topology::topo_array::TopoArray;

#[cfg(test)]

#[test]
pub fn full_test_refactor() {
    // Arrange
    let width = 5;
    let height = 5;
    let initial_data = vec![
        vec!['_', '_', '_'],
        vec!['_', '*', '_'],
        vec!['_', '_', '_'],
    ];
    let initial_vec = initial_data
        .into_iter()
        .map(|x| x
            .clone()
            .iter()
            .map(|y| y.to_tile())
            .collect::<Vec<Tile>>()
        )
        .collect::<Vec<Vec<Tile>>>();

    let topology = GridTopology::new_2d(width, height, false);

    let sample = RaggedTopoArray2D::new(initial_vec, false);

    let mut model = AdjacentModel::new();
    model.add_sample_simple::<GridTopology>(&sample).unwrap();

    let rng = Mutex::new(Pcg32::new(12345));
    let random_double = Rc::new(move || {
        let mut rng = rng.lock().unwrap();
        let next = rng.next_f64();
        next
    });

    let mut tile_propagator_options = TilePropagatorOptions::new(true, Some(random_double), None);
    tile_propagator_options.backtrack = BacktrackType::Backjump;
    let model = Box::new(model);

    // Act
    let mut tile_propagator = TilePropagator::with_options(model, topology.clone(), tile_propagator_options).unwrap();
    let status = tile_propagator.run().unwrap();
    assert_ne!(status, Resolution::Contradiction);

    let output = tile_propagator.to_value_array();
    assert!(output.is_ok());

    // Assert
    let output_visuals = output_to_visuals(output.unwrap());
    // debug_print_output(&output_visuals);

    let expected_output = vec![
        vec!['_', '_', '_', '_', '_'],
        vec!['*', '_', '_', '_', '_'],
        vec!['_', '_', '_', '_', '_'],
        vec!['_', '_', '_', '_', '_'],
        vec!['_', '_', '_', '_', '_'],
    ];

    assert_eq!(expected_output.len(), output_visuals.len());

    for (y, row) in expected_output.iter().enumerate() {
        for (x, col) in row.iter().enumerate() {
            assert_eq!(expected_output[y].len(), output_visuals[y].len());
            assert_eq!(format!("{}", *col), format!("{}", output_visuals[y][x]));
        }
    }
}

#[test]
pub fn speed_test() {
    // Arrange
    let start = std::time::Instant::now();
    let width = 200;
    let height = 200;

    let initial_data = vec![
        vec!['_', '_', '_'],
        vec!['_', '*', '_'],
        vec!['_', '_', '_'],
    ];
    let initial_vec = initial_data
        .into_iter()
        .map(|x| x
            .clone()
            .iter()
            .map(|y| y.to_tile())
            .collect::<Vec<Tile>>()
        )
        .collect::<Vec<Vec<Tile>>>();

    let topology = GridTopology::new_2d(width, height, false);

    let sample = RaggedTopoArray2D::new(initial_vec, false);

    let mut model = AdjacentModel::new();
    model.add_sample_simple::<GridTopology>(&sample).unwrap();

    let rng = Mutex::new(Pcg32::new(12345));
    let random_double = Rc::new(move || {
        let mut rng = rng.lock().unwrap();
        let next = rng.next_f64();
        next
    });

    let mut tile_propagator_options = TilePropagatorOptions::new(true, Some(random_double), None);
    tile_propagator_options.backtrack = BacktrackType::Backjump;
    let model = Box::new(model);

    // Act
    let mut tile_propagator = TilePropagator::with_options(model, topology.clone(), tile_propagator_options).unwrap();
    let status = tile_propagator.run().unwrap();
    assert_ne!(status, Resolution::Contradiction);

    let output = tile_propagator.to_value_array();
    assert!(output.is_ok());

    let end = std::time::Instant::now();
    println!("Time elapsed: {:?}", end - start);
}

fn output_to_visuals(output_result: Box<dyn TopoArray<TileVisual, GridTopology>>) -> Vec<Vec<TileVisual>> {
    let mut output_visuals: Vec<Vec<TileVisual>> = vec![Vec::new(); 0];
    for y in 0..5 {
        if output_visuals.len() < y + 1 {
            output_visuals.push(Vec::new());
        }
        for x in 0..5 {
            let output_visual = output_result.get_coord_2d(x, y);
            // print!("{}", output_visual.unwrap());
            output_visuals[y].push(output_visual.unwrap().clone());
        }
        println!();
    }
    output_visuals
}

/// Just for testing that the C# and Rust implementations produce the same output.
#[derive(Clone)]
pub struct Pcg32 {
    state: u64,
}

impl Pcg32 {
    const MULTIPLIER: u64 = 6364136223846793005;
    const INCREMENT:  u64 = 1442695040888963407;

    /// Create a new PCG32 RNG from a 64-bit seed.
    pub fn new(seed: u64) -> Self {
        // Same seeding scheme as the reference pcg32_init:
        // state = seed + increment; then advance once.
        let mut rng = Pcg32 {
            state: seed.wrapping_add(Self::INCREMENT),
        };
        rng.next_u32(); // warm up
        rng
    }

    /// Generate the next 32-bit output (pcg32()).
    pub fn next_u32(&mut self) -> u32 {
        let x = self.state;
        let count = (x >> 59) as u32; // 59 = 64 - 5

        // state = state * multiplier + increment (mod 2^64)
        self.state = x
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT);

        // xorshift high, then random rotate
        let xorshifted = (((x >> 18) ^ x) >> 27) as u32; // 18 = (64 - 27)/2, 27 = 32 - 5
        xorshifted.rotate_right(count)
    }

    /// Generate a uniform f64 in [0, 1) using 53 random bits.
    pub fn next_f64(&mut self) -> f64 {
        // Standard technique: combine 27 + 26 random bits into a 53-bit mantissa.
        let hi = (self.next_u32() >> 5) as u64; // 27 bits
        let lo = (self.next_u32() >> 6) as u64; // 26 bits
        let value = (hi << 26) | lo;           // 53 bits total

        (value as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}