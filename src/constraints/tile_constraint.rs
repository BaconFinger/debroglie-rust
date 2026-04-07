use std::sync::{Arc, Mutex};

use crate::resolution::Resolution;
use crate::tile::Tile;
use crate::tile_propagator::TilePropagatorState;
use crate::topology::topology::Topology;
use crate::wfc::pattern_model_constraint::PatternModelConstraint;
use crate::wfc::wave_propagator::{WavePropagator, WavePropagatorError};

#[derive(Debug, thiserror::Error)]
pub enum ConstraintError {
    #[error(transparent)]
    WavePropagatorError(#[from] WavePropagatorError),
}

/// Mainly, this class exists so the Constraint doesn't have to keep these dependencies as fields,
/// since these objects aren't really used outside the Trait calls.
/// This has a side benefit of containing lifetime specifications.
pub struct ConstraintDependencies<'a, 'b, 'c, T: Topology + Clone + 'static> {
    pub tile_propagator_state: &'a TilePropagatorState<T>,
    pub wave_propagator: &'b mut WavePropagator<T>,
    pub pattern_model_constraint: &'c mut dyn PatternModelConstraint<T>,
}

impl<'a, 'b, 'c, T: Topology + Clone + 'static> ConstraintDependencies<'a, 'b, 'c, T> {
    pub fn new(
        tile_propagator_state: &'a TilePropagatorState<T>,
        wave_propagator: &'b mut WavePropagator<T>,
        pattern_model_constraint: &'c mut dyn PatternModelConstraint<T>,
    ) -> Self {
        Self {
            tile_propagator_state,
            wave_propagator,
            pattern_model_constraint,
        }
    }

    /// Bans the specified tiles through the wave propagator.
    /// In the original C# implementation, this was a method on TilePropagator.
    pub fn ban(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        tiles: &Vec<Tile>,
    ) -> Result<Resolution, ConstraintError> {
        let tile_set = self
            .tile_propagator_state
            .get_tile_model_mapping()
            .create_tile_set(tiles);

        let (px, py, pz, o) = self
            .tile_propagator_state
            .get_tile_model_mapping()
            .get_tile_coord_to_pattern_coord(x, y, z);

        let patterns = self
            .tile_propagator_state
            .get_tile_model_mapping()
            .get_patterns_from_tile_set(Arc::new(Mutex::new(tile_set)), o);

        for p in patterns {
            let status = self.wave_propagator.ban(
                px,
                py,
                pz,
                p as i32,
                self.tile_propagator_state,
                self.pattern_model_constraint,
            )?;
            if status != Resolution::Undecided {
                return Ok(status);
            }
        }

        Ok(Resolution::Undecided)
    }

    /// Selects the specified tiles through the wave propagator.
    /// In the original C# implementation, this was a method on TilePropagator.
    pub fn select(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        tiles: &Vec<Tile>,
    ) -> Result<Resolution, ConstraintError> {
        let tile_set = self
            .tile_propagator_state
            .get_tile_model_mapping()
            .create_tile_set(tiles);

        let (px, py, pz, o) = self
            .tile_propagator_state
            .get_tile_model_mapping()
            .get_tile_coord_to_pattern_coord(x, y, z);

        let patterns = self
            .tile_propagator_state
            .get_tile_model_mapping()
            .get_patterns_from_tile_set(Arc::new(Mutex::new(tile_set)), o);

        for p in 0..self.wave_propagator.get_pattern_count() {
            if patterns.contains(&p) {
                continue;
            };

            let status = self.wave_propagator.ban(
                px,
                py,
                pz,
                p as i32,
                self.tile_propagator_state,
                self.pattern_model_constraint,
            )?;
            if status != Resolution::Undecided {
                return Ok(status);
            }
        }

        Ok(Resolution::Undecided)
    }
}

/// Interface for specifying non-local constraints to be respected during generation.
pub trait Constraint<T>
where
    T: Topology + Clone + 'static,
{
    /// Called once when the propagator first initializes.
    /// The propagator to constrain
    fn init(&mut self, dependencies: &mut ConstraintDependencies<T>)
    -> Result<(), ConstraintError>;

    /// Called frequently during generation to help maintain the constraint.
    /// The propagator to constrain
    fn check(&mut self, dependencies: &ConstraintDependencies<T>) -> Result<(), ConstraintError>;
}
