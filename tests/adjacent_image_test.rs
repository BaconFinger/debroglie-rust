use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use image::{Pixel, Rgba};
use debroglie_rust::context::Context;
use debroglie_rust::models::adjacent_model::AdjacentModel;
use debroglie_rust::resolution::Resolution;
use debroglie_rust::tile::{Tile, TileVisual, ToTile};
use debroglie_rust::tile_propagator::TilePropagator;
use debroglie_rust::tile_propagator_options::{BacktrackType, TilePropagatorOptions};
use debroglie_rust::topology::grid_topology::GridTopology;
use debroglie_rust::topology::ragged_topology_array_2d::RaggedTopoArray2D;
use debroglie_rust::topology::topo_array::TopoArray;
use crate::full_test::{debug_print_output, output_to_visuals, Pcg32};

#[cfg(test)]

const width: usize = 5;
const height: usize = 5;

#[test]
pub fn test_simple_adjacent_image() {
    // Arrange
    let ctx: Context<GridTopology> = Context::new();
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
    let initial_vec = ctx.tiles().add_vec_2d(initial_vec);

    let topology = GridTopology::new_2d(width, height, false);
    let topology = ctx.topologies().add(topology);

    let sample = RaggedTopoArray2D::new(&ctx, initial_vec, false);

    let mut model = AdjacentModel::new();
    model.add_sample_simple(&ctx, &sample).unwrap();
    let model = ctx.models().add(Rc::new(RefCell::new(model)));

    let rng = Mutex::new(Pcg32::new(12345));
    let random_double = Rc::new(move || {
        let mut rng = rng.lock().unwrap();
        let next = rng.next_f64();
        next
    });

    let mut tile_propagator_options = TilePropagatorOptions::new(true, Some(random_double), None);
    tile_propagator_options.backtrack = BacktrackType::Backjump;

    // Act
    let tile_propagator = TilePropagator::with_options(&ctx, model.clone(), topology.clone(), tile_propagator_options).unwrap();
    let tile_propagator = ctx.tile_propagators().get(tile_propagator).unwrap();
    let status = tile_propagator.borrow_mut().run(&ctx).unwrap();
    assert_ne!(status, Resolution::Contradiction);

    let output = tile_propagator.borrow().to_value_array(&ctx);
    assert!(output.is_ok());

    // Assert
    let output_visuals = output_to_visuals(&ctx, output.unwrap());
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

fn assert_pixel_eq(expected: &Rgba<u8>, actual: &Rgba<u8>) {
    let expected_channels = expected.channels();
    let actual_channels = actual.channels();
    assert_eq!(expected_channels.len(), actual_channels.len());
    assert_eq!(expected_channels[0], actual_channels[0], "R");
    assert_eq!(expected_channels[1], actual_channels[1], "G");
    assert_eq!(expected_channels[2], actual_channels[2], "B");
    assert_eq!(expected_channels[3], actual_channels[3], "A");
}