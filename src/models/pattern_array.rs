use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::Wrapping;
use serde_derive::Serialize;
use crate::tile::Tile;

pub(crate) struct PatternArray {
    values: Vec<Vec<Vec<Tile>>>,
}

impl PatternArray {

    pub fn default() -> Self {
        Self { values: Vec::new() }
    }
    pub fn new(values: Vec<Vec<Vec<Tile>>>) -> Self {
        Self { values }
    }

    pub fn get_values(&self) -> &Vec<Vec<Vec<Tile>>> {
        &self.values
    }

    pub fn width(&self) -> usize {
        self.values.len()
    }

    pub fn height(&self) -> usize {
        if self.width() == 0 {
            return 0;
        }
        self.values[0].len()
    }

    pub fn depth(&self) -> usize {
        if self.width() == 0 || self.height() == 0 {
            return 0;
        }
        self.values[0][0].len()
    }
    
    pub fn insert(&mut self, x: usize, y: usize, z: usize, tile: Tile) {
        self.values[x][y][z] = tile;
    }
}

impl PartialEq for PatternArray {
    fn eq(&self, other: &Self) -> bool {
        for x in 0..self.width() {
            for y in 0..self.height() {
                for z in 0..self.depth() {
                    if self.values[x][y][z] != other.values[x][y][z] {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl Eq for PatternArray {

}

impl Hash for PatternArray {
    // This is as direct of a port of the C# as I can get. Maybe overkill.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut hash_code: Wrapping<i32> = Wrapping(13);
        for x in 0..self.width() {
            for y in 0..self.height() {
                for z in 0..self.depth() {
                    let constant_value: Wrapping<i32> = Wrapping(397);
                    let mut h = DefaultHasher::new();
                    let tile_hash = self.values[x][y][z].get_value().hash(&mut h);
                    hash_code = (hash_code * constant_value) ^ Wrapping(h.finish() as i32);
                }
            }
        }
        hash_code.0.hash(state);
    }
}