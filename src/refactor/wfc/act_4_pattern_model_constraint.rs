use std::cell::{RefCell};
use std::rc::Rc;
use crate::refactor::resolution::Resolution;
use crate::refactor::topology::direction::Direction;
use crate::refactor::topology::topology::Topology;
use crate::refactor::wfc::pattern_model::PatternModel;
use crate::refactor::wfc::pattern_model_constraint::PatternModelConstraint;
use crate::refactor::wfc::wave_propagator::{IndexPatternItem, WavePropagator, WavePropagatorState};

/// Implements pattern adjacency propagation using the arc consistency 4 algorithm.
///
/// Roughly speaking, this algorithm keeps a count for each cell/pattern/direction of the "support",
/// i.e. how many possible cells could adjoin that particular pattern.
/// This count can be straightforwardly updated, and when it drops to zero, we know that that cell/pattern is not possible, and can be banned.
pub struct Ac4PatternModelConstraint<T: Topology + Clone> {
    // From model
    propagator_array: Vec<Vec<Vec<usize>>>,
    pattern_count: usize,

    // Re-organized propagator_array
    propagator_array_dense: Vec<Vec<Vec<bool>>>,

    // Useful values
    // propagator: WavePropagatorId,
    // topology: TopologyId,
    directions_count: usize,
    index_count: usize,

    // List of locations that still need to be checked against for fulfilling the model's conditions
    to_propagate: Vec<IndexPatternItem>,

    /// compatible[index][pattern][direction] contains the number of patterns present in the wave
    /// that can be placed in the cell next to index in direction without being
    /// in contradiction with pattern placed in index.
    /// If possibilities[index][pattern] is set to false, then compatible[index][pattern][direction] has every direction negative or null
    compatible: Vec<Vec<Vec<i32>>>,
    _phantom: std::marker::PhantomData<T>,
    initialized: bool,
}

impl<T: Topology + Clone> Ac4PatternModelConstraint<T> {
    pub fn new(topology: &T, model: &PatternModel) -> Self {
        let propagator_array = model.propagator().clone();
        let pattern_count = model.pattern_count();

        // Convert propagator array to dense bit representation
        let propagator_array_dense: Vec<Vec<Vec<bool>>> = propagator_array
            .iter()
            .map(|a1| {
                a1.iter()
                    .map(|x| {
                        let mut dense = vec![false; pattern_count];
                        for &p in x {
                            dense[p] = true;
                        }
                        dense
                    })
                    .collect()
            })
            .collect();

        let index_count = topology.index_count();
        let directions_count = topology.directions_count();

        Ac4PatternModelConstraint {
            propagator_array,
            pattern_count,
            propagator_array_dense,
            directions_count,
            index_count,
            to_propagate: Vec::new(),
            compatible: Vec::new(),
            _phantom: std::marker::PhantomData,
            initialized: true
        }
    }

    pub fn init() -> Self {
        Ac4PatternModelConstraint {
            propagator_array: Vec::new(),
            pattern_count: 0,
            propagator_array_dense: Vec::new(),
            directions_count: 0,
            index_count: 0,
            to_propagate: Vec::new(),
            compatible: Vec::new(),
            _phantom: std::marker::PhantomData,
            initialized: false
        }
    }
}

impl<T: Topology + Clone + 'static> PatternModelConstraint<T> for Ac4PatternModelConstraint<T> {
    fn clear(&mut self, topology: &T, wave_propagator_state: &WavePropagatorState<T>) -> Result<Option<(usize, usize)>, String> {
        if !self.initialized {
            return Err("Ac4PatternModelConstraint not initialized".to_string());
        }

        self.to_propagate.clear();
        // let binding = self.propagator.lock().unwrap().inner().clone();
        // let inner = binding.lock().unwrap();
        // let topology = inner.get_topology();
        // let topology = topology.lock().unwrap();
        // let topology = self.get_topology(ctx);

        // Initialize compatible array
        self.compatible = vec![
            vec![vec![0i32; self.directions_count]; self.pattern_count];
            self.index_count
        ];

        let mut edge_labels = vec![-1i32; self.directions_count];

        for index in 0..self.index_count {
            if !topology.contains_index(index) {
                continue;
            }

            // Cache edge_labels
            for d in 0..self.directions_count {
                let result = topology.try_move_full(index, Direction::from_index(d).unwrap());
                if result.is_err() {
                    return Err(format!("try_move_full {}", result.unwrap_err()));
                }
                edge_labels[d] = if let Some((_dest, _id, el)) =
                    result.unwrap() {
                    el as i32
                } else {
                    -1
                };
            }

            for pattern in 0..self.pattern_count {
                for d in 0..self.directions_count {
                    let el = edge_labels[d];
                    if el >= 0 {
                        let compatible_patterns = self.propagator_array[pattern][el as usize].len() as i32;
                        self.compatible[index][pattern][d] = compatible_patterns;

                        if compatible_patterns == 0 {
                            let wave_exists = {
                                let wave = wave_propagator_state.get_wave().as_ref().ok_or("No wave")?;
                                let result = wave.get(index, pattern);
                                result
                            };
                            if wave_exists {
                                return Ok(Some((index, pattern)));
                                // if propagator.borrow_mut().internal_ban(ctx, index, pattern).unwrap() {
                                //     propagator.borrow_mut().set_contradiction();
                                // }
                                // break;
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn do_ban(&mut self, index: usize, pattern: i32) -> Result<(), String> {
        if !self.initialized {
            return Err("Ac4PatternModelConstraint not initialized".to_string());
        }

        // Update compatible (so that we never ban twice)
        for d in 0..self.directions_count {
            self.compatible[index][pattern as usize][d] -= self.pattern_count as i32;
        }

        // Queue any possible consequences of this changing
        self.to_propagate.push(IndexPatternItem::new(index as i32, pattern as i32));

        Ok(())
    }

    fn undo_ban(&mut self, index: usize, pattern: i32, topology: &T) -> Result<(), String> {
        if !self.initialized {
            return Err("Ac4PatternModelConstraint not initialized".to_string());
        }

        // Undo what was done in do_ban

        // First restore compatible for this cell
        // As it is set a negative value in internal_ban
        for d in 0..self.directions_count {
            self.compatible[index][pattern as usize][d] += self.pattern_count as i32;
        }

        // As we always Undo in reverse order, if item is in to_propagate, it'll
        // be at the top of the stack.
        // If item is in to_propagate, then we haven't got round to processing yet, so there's nothing to undo.
        if !self.to_propagate.is_empty() {
            let top = self.to_propagate.last().unwrap();
            if top.index == index as i32 && top.pattern == pattern as i32 {
                self.to_propagate.pop();
                return Ok(());
            }
        }

        // Not in to_propagate, therefore undo what was done in propagate
        for d in 0..self.directions_count {

            let result = topology.try_move_full(index, Direction::from_index(d).unwrap());
            if result.is_err() {
                return Err(format!("try_move_full {}", result.unwrap_err()));
            }
            if let Some((i2, id, el)) = result.unwrap() {
                let patterns = &self.propagator_array[pattern as usize][el as usize];
                for &p in patterns {
                    self.compatible[i2][p][id as usize] += 1;
                }
            }
        }

        Ok(())
    }

    fn do_select(&mut self, index: usize, pattern: i32) -> Result<(), String> {
        if !self.initialized {
            return Err("Ac4PatternModelConstraint not initialized".to_string());
        }

        // Update compatible (so that we never ban twice)
        for p in 0..self.pattern_count {
            if p == pattern as usize {
                continue;
            }
            for d in 0..self.directions_count {
                if self.compatible[index][p][d] > 0 {
                    self.compatible[index][p][d] -= self.pattern_count as i32;
                }
            }
        }

        // Queue any possible consequences of this changing
        self.to_propagate.push(IndexPatternItem::new(index as i32, !(pattern as i32)));

        Ok(())
    }

    fn propagate(&mut self, topology: &T, wave_propagator: &mut WavePropagator<T>) -> Result<(), String> {
        if !self.initialized {
            return Err("Ac4PatternModelConstraint not initialized".to_string());
        }

        while !self.to_propagate.is_empty() {
            let item = self.to_propagate.pop().unwrap();

            // Get coordinates first
            let (x, y, z) = {
                let result = topology.get_coord(item.index as usize);
                if result.is_err() {
                    return Err(format!("get_coord failed {}", result.unwrap_err()));
                }
                result.unwrap()
            };

            if item.pattern >= 0 {
                // Process a ban
                for d in 0..self.directions_count {
                    // Get movement result and patterns in separate scope
                    let (move_result, patterns) = {
                        let move_result = topology.try_move_coord_full(x, y, z, Direction::from_index(d).unwrap());
                        let patterns = if let Ok(Some((_, _, el))) = move_result {
                            self.propagator_array[item.pattern as usize][el as usize].clone()
                        } else {
                            Vec::new()
                        };
                        (move_result, patterns)
                    };

                    if let Ok(Some((i2, id, _el))) = move_result {
                        let result = self.propagate_ban_core(&patterns, i2, id as usize)?;
                        if result.is_some() {
                            let (idx_to_ban, pattern_to_ban) = result.unwrap(); // safe because checked above
                            if wave_propagator.internal_ban(idx_to_ban, pattern_to_ban).unwrap() {
                                wave_propagator.set_contradiction();
                            }
                        }
                    }
                }
            } else {
                // Process a select
                let pattern = (!item.pattern) as usize;
                for d in 0..self.directions_count {
                    // Get movement result and patterns_dense in separate scope
                    let (move_result, patterns_dense) = {
                        let move_result = topology.try_move_coord_full(x, y, z, get_direction(d));
                        let patterns_dense = if let Ok(Some((_, _, el))) = move_result {
                            self.propagator_array_dense[pattern][el as usize].clone()
                        } else {
                            Vec::new()
                        };
                        (move_result, patterns_dense)
                    };

                    if let Ok(Some((i2, id, _el))) = move_result {
                        let result = self.propagate_select_core(&patterns_dense, i2, id as usize)?;
                        if result.is_some() {
                            let (idx_to_ban, pattern_to_ban) = result.unwrap(); // safe because checked above
                            if wave_propagator.internal_ban(idx_to_ban, pattern_to_ban).unwrap() {
                                wave_propagator.set_contradiction();
                            }
                        }
                    }
                }
            }

            if wave_propagator.get_state().status() == Resolution::Contradiction {
                println!("Status checked");
                return Ok(());
            }
            // Check status after processing
            // TODO: Fix this unsafe stuff
            // let propagator_cell = self.get_wave_propagator(ctx);
            // let propagator = propagator_cell.as_ptr();
            // unsafe {
            //     // println!("Checking status");
            //     if (*propagator).status() == Resolution::Contradiction {
            //         println!("Status checked");
            //         return;
            //     }
            // }
            // let propagator = self.get_wave_propagator(ctx);
            // if propagator.borrow().status() == Resolution::Contradiction {
            //     return;
            // }
        }
        Ok(())
    }
}

pub fn get_direction(d: usize) -> Direction {
    let result = Direction::from_index(d);
    if result.is_none() {
        panic!("get_direction failed");
    }
    return result.unwrap();
}

impl<T: Topology + Clone> Ac4PatternModelConstraint<T> {
    fn propagate_ban_core(&mut self, patterns: &[usize], i2: usize, d: usize) -> Result<Option<(usize, usize)>, String> {
        if !self.initialized {
            return Err("Ac4PatternModelConstraint not initialized".to_string());
        }

        // Hot loop
        for &p in patterns {
            self.compatible[i2][p][d] -= 1;
            let c = self.compatible[i2][p][d];
            // Have we just now ruled out this possible pattern?
            if c == 0 {
                return Ok(Some((i2, p)));
                // let propagator = self.get_wave_propagator(ctx);
                // // TODO: Fix the dependency chain so we don't have to do this unsafe stuff.
                // let propagator_ptr = propagator.as_ptr();
                // unsafe {
                //     // println!("Calling internal_ban");
                //     if (*propagator_ptr).internal_ban(ctx, i2, p).unwrap() {
                //         println!("Calling set_contradiction");
                //         (*propagator_ptr).set_contradiction();
                //         println!("Called set_contradiction");
                //     }
                // }
                // let propagator = self.get_wave_propagator(ctx);
                // if propagator.borrow_mut().internal_ban(ctx, i2, p).unwrap() {
                //     propagator.borrow_mut().set_contradiction();
                // }
            }
        }
        Ok(None)
    }

    fn propagate_select_core(&mut self, patterns_dense: &[bool], i2: usize, id: usize) -> Result<Option<(usize, usize)>, String> {
        if !self.initialized {
            return Err("Ac4PatternModelConstraint not initialized".to_string());
        }

        for p in 0..self.pattern_count {
            let patterns_contains_p = patterns_dense[p];

            // Sets the value of compatible, triggering internal bans
            let prev_compatible = self.compatible[i2][p][id];
            let currently_possible = prev_compatible > 0;
            let new_compatible = if currently_possible { 0 } else { -(self.pattern_count as i32) }
                + if patterns_contains_p { 1 } else { 0 };
            self.compatible[i2][p][id] = new_compatible;

            // Have we just now ruled out this possible pattern?
            if new_compatible == 0 {
                return Ok(Some((i2, p)));
                // let propagator_cell = self.get_wave_propagator(ctx);
                // let propagator = propagator_cell.as_ptr();
                // unsafe {
                //     // println!("Calling internal_ban");
                //     if (*propagator).internal_ban(ctx, i2, p).unwrap() {
                //         println!("Calling set_contradiction");
                //         (*propagator).set_contradiction();
                //         println!("Called set_contradiction");
                //     }
                // }
                // let propagator = self.get_wave_propagator(ctx);
                // if propagator.borrow_mut().internal_ban(ctx, i2, p).unwrap() {
                //     propagator.borrow_mut().set_contradiction();
                // }
            }
        }
        Ok(None)
    }
}