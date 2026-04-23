use std::borrow::BorrowMut;
use std::cmp::{max, min};
use crate::rot::rotations::Rotation;
use crate::rot::tile_rotation::TileRotation;
use crate::tile::Tile;
use crate::topology::direction::DirectionSetType;
use crate::topology::grid_topology::GridTopology;
use crate::topology::topo_array::TopoArray;
use crate::topology::topo_array_3d::TopoArray3D;
use crate::topology::topology::Topology;

pub struct TopoArrayUtils {}

impl TopoArrayUtils {
    pub fn rotate<'a>(original: &'a dyn TopoArray<Tile, GridTopology>, rotation: &Rotation, tile_rotation: &TileRotation ) -> Box<dyn TopoArray<Tile, GridTopology>> {
        if original.topology().is_none() {
            // Look, we've done this check so many times already.
            // If the topology is none, then we genuinely need to panic.
            panic!("Original topology is none");
        }
        let topology = original.topology().unwrap();
        let direction_type = topology.directions().direction_type();
        match direction_type {
            DirectionSetType::Unknown => { panic!("Unknown direction type") } // This is actually unrecoverable.
            DirectionSetType::Cartesian2d => {Self::square_rotate(original, rotation, tile_rotation)}
            DirectionSetType::Cartesian3d => {Self::square_rotate(original, rotation, tile_rotation)}
            DirectionSetType::Hexagonal2d => {Self::hex_rotate(original, rotation, tile_rotation)}
            DirectionSetType::Hexagonal3d => {Self::hex_rotate(original, rotation, tile_rotation)}
        }
    }

    pub fn square_rotate<'a>(original: &'a dyn TopoArray<Tile, GridTopology>, rotation: &Rotation, tile_rotation: &TileRotation) -> Box<dyn TopoArray<Tile, GridTopology>> {

        if rotation.is_identity() {
            return original.clone_box().unwrap();
        }

        Self::rotate_inner(original, rotation, tile_rotation)
    }

    pub fn square_rotate_vector(mut x: i32, y: i32, rotation: &Rotation) -> (i32, i32) {
        if rotation.get_reflect_x() {
            x = -x;
        }

        let rotation_cw = rotation.get_rotate_cw();
        if rotation_cw == 0 {
            return (x, y);
        }
        if rotation_cw == 90 {
            return (-y, x);
        }
        if rotation_cw == 180 {
            return (-x, -y);
        }
        if rotation_cw == 270 {
            return (y, -x);
        }

        // TODO: Ensure panic cannot happen
        panic!("Unexpected rotation angle {}", rotation_cw); // Something went wrong with the calculations.
    }

    fn rotate_inner<'a>(original: &dyn TopoArray<Tile, GridTopology>, rotation: &Rotation, tile_rotation: &TileRotation) -> Box<dyn TopoArray<Tile, GridTopology>> {
        if original.topology().is_none() {
            // Look, we've done this check so many times already.
            // If the topology is none, then we genuinely need to panic.
            panic!("Original topology is none");
        }
        let topology = original.topology().unwrap();
        let map_coord: Box<dyn Fn(i32, i32) -> (i32, i32)> = Box::new(|x, y| {
            return Self::square_rotate_vector(x, y, rotation);
        });

        // Find new bounds
        let (x1, y1) = map_coord(0, 0);
        let (x2, y2) = map_coord((topology.width() - 1) as i32, 0);
        let (x3, y3) = map_coord((topology.width() - 1) as i32, (topology.height() - 1) as i32);
        let (x4, y4) = map_coord(0, (topology.height() - 1) as i32);

        let min_x = min(min(x1, x2), min(x3, x4));
        let max_x = max(max(x1, x2), max(x3, x4));
        let min_y = min(min(y1, y2), min(y3, y4));
        let max_y = max(max(y1, y2), max(y3, y4));

        // Arrange so that co-ordinate transfer is into the rect bounced by width, height
        let offset_x = -min_x;
        let offset_y = -min_y;
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let depth = topology.depth();
        let mask: Vec<bool> = vec![false; width as usize * height as usize * depth as usize];
        let mut new_topology = GridTopology::new(
            topology.directions().clone(),
            width as usize,
            height as usize,
            depth,
            false,
            false,
            false,
            Some(mask),
        );
        let mut values = vec![vec![vec![Tile::default(); depth]; height as usize]; width as usize];

        // Copy from original to values based on the rotation, setting up the mask as we go.
        for z in 0..depth {
            for y in topology.height()..0 {
                for x in topology.width()..0 {
                    let (mut new_x, mut new_y) = map_coord(x as i32, y as i32);
                    new_x += offset_x;
                    new_y += offset_y;

                    let new_index = new_topology.get_index(new_x as usize, new_y as usize, z).unwrap(); // TODO: Handle error
                    let mut new_value = original.get_value_from_coord(x, y, z).unwrap(); // TODO: Handle error
                    let (has_new_value, new_new_value) = tile_rotation.rotate(new_value, rotation);

                    values[new_x as usize][new_y as usize][z] = new_new_value.unwrap().clone();

                    let to_find = topology.get_index(x, y, z).unwrap(); // TODO: Handle error
                    // mask[new_index] = has_new_value && topology.contains_index(to_find);
                    new_topology.set_mask_value(new_index, topology.contains_index(to_find));
                }
            }
        }

        Box::new(TopoArray3D::with_topology(values, new_topology))
    }

    pub fn hex_rotate<'a>(original: &dyn TopoArray<Tile, GridTopology>, rotation: &Rotation, tile_rotation: &TileRotation ) -> Box<dyn TopoArray<Tile, GridTopology>> {
        unimplemented!("TopoArrayUtils::hex_rotate")
    }
}