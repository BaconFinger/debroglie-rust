use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use image::{GenericImageView, ImageReader, Pixel, Rgba, RgbaImage};
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

#[test]
pub fn test_adjacent_image_from_file() {
    println!("{}", std::env::current_dir().unwrap().as_path().display());
    let img = ImageReader::open("tests/samples/pathway.png").unwrap().decode().unwrap().into_rgba8();
    let img_array = image_to_rgba8_array(&img);

    // Arrange
    let ctx: Context<GridTopology> = Context::new();
    let initial_vec = img_array
        .into_iter()
        .map(|x| x
            .clone()
            .iter()
            .map(|y| y.to_tile())
            .collect::<Vec<Tile>>()
        )
        .collect::<Vec<Vec<Tile>>>();

    let initial_vec = ctx.tiles().add_vec_2d(initial_vec);

    let topology = GridTopology::new_2d(48, 48, false);
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
    let output_visuals = output_to_visuals_2(&ctx, output.unwrap(), 48, 48);
    let expected_output = load_snapshot();
    // save_output_image(&output_visuals);

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
fn output_to_visuals_2(ctx: &Context<GridTopology>, output_result: Box<dyn TopoArray<TileVisual, GridTopology>>, w: usize, h: usize) -> Vec<Vec<TileVisual>> {
    let mut output_visuals: Vec<Vec<TileVisual>> = vec![Vec::new(); 0];
    for y in 0..h {
        if output_visuals.len() < y + 1 {
            output_visuals.push(Vec::new());
        }
        for x in 0..w {
            let output_visual = output_result.get_coord_2d(&ctx, x, y);
            // print!("{}", output_visual.unwrap());
            output_visuals[y].push(output_visual.unwrap().clone());
        }
        println!();
    }
    output_visuals
}

fn save_output_image(output_visuals: &Vec<Vec<TileVisual>>) {
    let output_y = output_visuals.len();
    let output_x = output_visuals[0].len(); // big assumption

    let mut output_img = RgbaImage::new(output_x as u32, output_y as u32); // could be sketchy

    for (y, row) in output_visuals.iter().enumerate() {
        for (x, col) in row.iter().enumerate() {
            let pixel = col.as_pixel().unwrap();
            output_img.put_pixel(x as u32, y as u32, pixel.clone());
        }
    }

    output_img.save("tests/samples/pathway_output.png").unwrap();
}

fn load_snapshot() -> Vec<Vec<Rgba<u8>>> {
    let snapshot = ImageReader::open("tests/snapshots/pathway_adjacent.png").unwrap().decode().unwrap().into_rgba8();
    image_to_rgba8_array(&snapshot)
}