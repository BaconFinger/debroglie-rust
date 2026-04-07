use bitflags::bitflags;

use crate::{
    constraints::tile_constraint::{Constraint, ConstraintDependencies, ConstraintError},
    tile::Tile,
    topology::topology::Topology,
};

bitflags! {
    /// Used by BorderConstraint to indicate what area affected.
    #[derive(Debug, Clone, Copy)]
    pub struct BorderSides: u32 {
        const None = 0;
        const XMin = 0x01;
        const XMax = 0x02;
        const YMin = 0x04;
        const YMax = 0x08;
        const ZMin = 0x10;
        const ZMax = 0x20;
        const All = 0x3F;
    }
}

/// BorderConstraint class restricts what tiles can be selected in various regions of the output.
///
/// For each affected location, BorderConstratin calls Select with the Tile specified.
/// If the Ban field is set, then it calls Ban instead of Select.
pub struct BorderConstraint {
    /// The tiles to select or ban fromthe  border area.
    tiles: Vec<Tile>,

    /// A set of flags specifying which sides of the output are affected by the constraint.
    sides: BorderSides,

    /// These locations are subtracted from the ones specified in <see cref="Sides"/>. Defaults to empty.
    exclude_sides: BorderSides,

    /// Inverts the area specified by <see cref="Sides"/> and <see cref="ExcludeSides"/>
    invert_area: bool,

    /// If true, ban Tile from the area. Otherwise, select it (i.e. ban every other tile).
    ban: bool,
}

impl BorderConstraint {
    fn do_match(
        &self,
        sides: BorderSides,
        xmin: bool,
        xmax: bool,
        ymin: bool,
        ymax: bool,
        zmin: bool,
        zmax: bool,
    ) -> bool {
        return xmin && sides.contains(BorderSides::XMin)
            || xmax && sides.contains(BorderSides::XMax)
            || ymin && sides.contains(BorderSides::YMin)
            || ymax && sides.contains(BorderSides::YMax)
            || zmin && sides.contains(BorderSides::ZMin)
            || zmax && sides.contains(BorderSides::ZMax);
    }
}

impl<T: Topology + Clone + 'static> Constraint<T> for BorderConstraint {
    /// Called once when the propagator first initializes.
    /// The propagator to constrain
    fn init(
        &mut self,
        dependencies: &mut ConstraintDependencies<T>,
    ) -> Result<(), ConstraintError> {
        let (width, height, depth) = {
            let topology = dependencies.tile_propagator_state.get_topology();
            (topology.width(), topology.height(), topology.depth())
        };

        for x in 0..width {
            let xmin = x == 0;
            let xmax = x == width - 1;

            for y in 0..height {
                let ymin = y == 0;
                let ymax = y == height - 1;

                for z in 0..depth {
                    let zmin = z == 0;
                    let zmax = z == depth - 1;

                    let is_match = self.do_match(self.sides, xmin, xmax, ymin, ymax, zmin, zmax)
                        && !self.do_match(self.exclude_sides, xmin, xmax, ymin, ymax, zmin, zmax)
                            == self.invert_area;

                    if is_match {
                        if self.ban {
                            dependencies.ban(x as i32, y as i32, z as i32, &self.tiles)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Called frequently during generation to help maintain the constraint.
    /// The propagator to constrain
    fn check(&mut self, dependencies: &ConstraintDependencies<T>) -> Result<(), ConstraintError> {
        // Does nothing
        Ok(())
    }
}
