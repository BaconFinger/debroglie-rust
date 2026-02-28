use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use image::{ImageReader, Pixel, Rgba, RgbaImage};
use debroglie_rust::context::Context;
use debroglie_rust::refactor::models::adjacent_model::AdjacentModel;
use debroglie_rust::refactor::resolution::Resolution;
use debroglie_rust::refactor::tile::{Tile, ToTile};
use debroglie_rust::refactor::tile_propagator::TilePropagator;
use debroglie_rust::refactor::tile_propagator_options::{BacktrackType, TilePropagatorOptions};
use debroglie_rust::refactor::topology::grid_topology::GridTopology;
use debroglie_rust::refactor::topology::ragged_topology_array_2d::RaggedTopoArray2D;
use debroglie_rust::refactor::tile::TileVisual;
use debroglie_rust::refactor::topology::topo_array::TopoArray;

#[cfg(test)]

#[test]
pub fn adjacent_image_refactor() {
    // Arrange
    let width = 5;
    let height = 5;
    let prpl: Rgba<u8> = Rgba([100, 0, 255, 255]); // Shortening name to visually align with 'blue' in arrays.
    let blue: Rgba<u8> = Rgba([0, 0, 255, 255]);

    let initial_data = vec![
        vec![Rgba([100, 0, 255, 255]), Rgba([100, 0, 255, 255]), Rgba([100, 0, 255, 255])],
        vec![Rgba([100, 0, 255, 255]), Rgba([000, 0, 255, 255]), Rgba([100, 0, 255, 255])],
        vec![Rgba([100, 0, 255, 255]), Rgba([100, 0, 255, 255]), Rgba([100, 0, 255, 255])],
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
    let output_visuals = output_to_visuals_2(output.unwrap(), width, height);
    // debug_print_output(&output_visuals);

    let expected_output = vec![
        vec![prpl.clone(), prpl.clone(), prpl.clone(), prpl.clone(), prpl.clone()],
        vec![blue.clone(), prpl.clone(), prpl.clone(), prpl.clone(), prpl.clone()],
        vec![prpl.clone(), prpl.clone(), prpl.clone(), prpl.clone(), prpl.clone()],
        vec![prpl.clone(), prpl.clone(), prpl.clone(), prpl.clone(), prpl.clone()],
        vec![prpl.clone(), prpl.clone(), prpl.clone(), prpl.clone(), prpl.clone()],
    ];

    assert_eq!(expected_output.len(), output_visuals.len());

    for (y, row) in expected_output.iter().enumerate() {
        for (x, col) in row.iter().enumerate() {
            assert_eq!(expected_output[y].len(), output_visuals[y].len());
            let actual = output_visuals[y][x].as_pixel().unwrap();
            assert_pixel_eq(col, actual);
        }
    }
}

#[test]
pub fn from_file_refactor() {
    println!("{}", std::env::current_dir().unwrap().as_path().display());
    let img = ImageReader::open("tests/samples/pathway.png").unwrap().decode().unwrap().into_rgba8();
    let img_array = image_to_rgba8_array(&img);

    // Arrange
    let width = 48;
    let height = 48;

    let initial_vec = img_array
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
    let output_visuals = output_to_visuals_2(output.unwrap(), width, height);
    let expected_output = load_snapshot();

    assert_eq!(expected_output.len(), output_visuals.len());
    for (y, row) in expected_output.iter().enumerate() {
        for (x, col) in row.iter().enumerate() {
            assert_eq!(expected_output[y].len(), output_visuals[y].len());
            let actual = output_visuals[y][x].as_pixel().unwrap();
            assert_pixel_eq(col, actual);
        }
    }
}

fn assert_pixel_eq(expected: &Rgba<u8>, actual: &Rgba<u8>) {
    let expected_channels = expected.channels();
    let actual_channels = actual.channels();
    assert_eq!(expected_channels.len(), actual_channels.len());
    assert_eq!(expected_channels[0], actual_channels[0], "R");
    assert_eq!(expected_channels[1], actual_channels[1], "G");
    assert_eq!(expected_channels[2], actual_channels[2], "B");
    assert_eq!(expected_channels[3], actual_channels[3], "A");
}

fn output_to_visuals_2(output_result: Box<dyn TopoArray<TileVisual, GridTopology>>, w: usize, h: usize) -> Vec<Vec<TileVisual>> {
    let mut output_visuals: Vec<Vec<TileVisual>> = vec![Vec::new(); 0];
    for y in 0..h {
        if output_visuals.len() < y + 1 {
            output_visuals.push(Vec::new());
        }
        for x in 0..w {
            let output_visual = output_result.get_coord_2d(x, y);
            // print!("{}", output_visual.unwrap());
            output_visuals[y].push(output_visual.unwrap().clone());
        }
        println!();
    }
    output_visuals
}

fn image_to_rgba8_array(img: &RgbaImage) -> Vec<Vec<Rgba<u8>>> {
    let w = img.width() as usize;
    let h = img.height() as usize;

    (0..h)
        .map(|y| {
            (0..w)
                .map(|x| img.get_pixel(x as u32, y as u32).clone())
                .collect()
        })
        .collect()
}

fn load_snapshot() -> Vec<Vec<Rgba<u8>>> {
    let snapshot = ImageReader::open("tests/snapshots/pathway_adjacent.png").unwrap().decode().unwrap().into_rgba8();
    image_to_rgba8_array(&snapshot)
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