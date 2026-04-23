use std::collections::HashMap;
use crate::rot::rotation_group::RotationGroup;
use crate::rot::rotations::Rotation;
use crate::tile::Tile;

pub enum TileRotationTreatment
{
    Missing,
    Unchanged,

    /// Experimental, this doesn't work properly yet.
    Generated,
}

/// Describes which rotations and reflections are allowed, and
/// and stores how to process each tile during a rotation.
/// These are constructed with a TileRotationBuilder
pub struct TileRotation {
    rotation_group: RotationGroup,
    default_treatment: TileRotationTreatment,
    treatments: HashMap<Tile, TileRotationTreatment>,
    rotations: HashMap<Tile, HashMap<Rotation, Tile>>,
}

impl TileRotation {
    pub fn new(rotational_symmetry: i32, reflectional_symmetry: bool) -> Self {
        Self {
            treatments: HashMap::new(),
            rotations: HashMap::new(),
            default_treatment: TileRotationTreatment::Unchanged,
            rotation_group: RotationGroup::new(rotational_symmetry, reflectional_symmetry),
        }
    }

    pub fn default() -> Self {
        Self::new(1, false)
    }

    pub fn get_rotation_group(&self) -> &RotationGroup {
        &self.rotation_group
    }

    /// Attempts to reflect, then rotate clockwise, a given Tile.
    /// If there is a corresponding tile (possibly the same one), then it is set to result.
    /// Otherwise, false is returned.
    pub fn rotate(&self, tile: &Tile, rotation: &mut Rotation) -> (bool, Option<Tile>) {
        if let Some(tile_rotation) = tile.rotation {
            *rotation = tile_rotation * rotation.clone();
        }

        if let Some(d) = self.rotations.get(tile) {
            if let Some(result) = d.get(rotation) {
                return (true, Some(result.clone()));
            }
        }

        // Transform not found, apply treatment
        let treatment = self.treatments.get(tile).unwrap_or(&self.default_treatment);
        match treatment {
            TileRotationTreatment::Missing => {
                (false, Some(Tile::default()))
            }
            TileRotationTreatment::Unchanged => {
                (true, Some(tile.clone()))
            }
            TileRotationTreatment::Generated => {
                if rotation.is_identity() {
                    return (true, Some(tile.clone()))
                }
                let mut result = tile.clone();
                result.rotation = Some(rotation.clone());
                (true, Some(result))
            }
        }
    }
}