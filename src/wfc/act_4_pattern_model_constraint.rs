use std::cell::{RefCell, UnsafeCell};
use std::rc::Rc;
use crate::procedural_generation::debroglie::context::{Context, TopologyId, WavePropagatorId};
use crate::procedural_generation::debroglie::resolution::Resolution;
use crate::procedural_generation::debroglie::topology::direction::Direction;
use crate::procedural_generation::debroglie::topology::topology::Topology;
use crate::procedural_generation::debroglie::wfc::pattern_model::PatternModel;
use crate::procedural_generation::debroglie::wfc::pattern_model_constraint::PatternModelConstraint;
use crate::procedural_generation::debroglie::wfc::wave_propagator::{IndexPatternItem, WavePropagator};

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
    propagator: WavePropagatorId,
    topology: TopologyId,
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
    pub fn new(ctx: &Context<T>, propagator_id: WavePropagatorId, model: &PatternModel) -> Self {
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

        let propagator = ctx.wave_propagators().get(propagator_id).unwrap().clone();
        let topology = propagator.borrow().topology.clone();

        let (index_count, directions_count) = {
            let inner = propagator.borrow();
            let topology = ctx.topologies().get(inner.topology).unwrap().clone();
            let topology_inner = topology.borrow();
            (topology_inner.index_count(), topology_inner.directions_count())
        };

        Ac4PatternModelConstraint {
            propagator_array,
            pattern_count,
            propagator_array_dense,
            propagator: propagator_id.clone(),
            topology,
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
            propagator: WavePropagatorId::default(),
            topology: TopologyId::default(),
            directions_count: 0,
            index_count: 0,
            to_propagate: Vec::new(),
            compatible: Vec::new(),
            _phantom: std::marker::PhantomData,
            initialized: false
        }
    }

    fn get_wave_propagator(&self, ctx: &Context<T>) -> Rc<RefCell<WavePropagator<T>>> {
        ctx.wave_propagators().get(self.propagator).unwrap().clone()
    }

    fn get_topology(&self, ctx: &Context<T>) -> Rc<RefCell<dyn Topology>> {
        ctx.topologies().get(self.topology).unwrap().clone()
    }
}

impl<T: Topology + Clone> PatternModelConstraint<T> for Ac4PatternModelConstraint<T> {
    fn clear(&mut self, ctx: &Context<T>) {
        if !self.initialized {
            panic!("Ac4PatternModelConstraint not initialized"); // TODO: Fix panic
        }

        self.to_propagate.clear();
        // let binding = self.propagator.lock().unwrap().inner().clone();
        // let inner = binding.lock().unwrap();
        // let topology = inner.get_topology();
        // let topology = topology.lock().unwrap();
        let topology = self.get_topology(ctx);

        // Initialize compatible array
        self.compatible = vec![
            vec![vec![0i32; self.directions_count]; self.pattern_count];
            self.index_count
        ];

        let mut edge_labels = vec![-1i32; self.directions_count];

        for index in 0..self.index_count {
            if !topology.borrow().contains_index(index) {
                continue;
            }

            // Cache edge_labels
            for d in 0..self.directions_count {
                let result = topology.borrow().try_move_full(index, Direction::from_index(d).unwrap());
                if result.is_err() {
                    panic!("try_move_full {}", result.unwrap_err());
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
                            let propagator = self.get_wave_propagator(ctx);
                            let wave_exists = {
                                let wave = propagator.borrow().get_wave(ctx).unwrap().clone();
                                let result = wave.borrow().get(index, pattern);
                                result
                            };
                            if wave_exists {
                                if propagator.borrow_mut().internal_ban(ctx, index, pattern).unwrap() {
                                    propagator.borrow_mut().set_contradiction();
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn do_ban(&mut self, ctx: &Context<T>, index: usize, pattern: i32) {
        if !self.initialized {
            panic!("Ac4PatternModelConstraint not initialized"); // TODO: Fix panic
        }

        // Update compatible (so that we never ban twice)
        for d in 0..self.directions_count {
            self.compatible[index][pattern as usize][d] -= self.pattern_count as i32;
        }

        // Queue any possible consequences of this changing
        self.to_propagate.push(IndexPatternItem::new(index as i32, pattern as i32));
    }

    fn undo_ban(&mut self, ctx: &Context<T>, index: usize, pattern: i32) {
        if !self.initialized {
            panic!("Ac4PatternModelConstraint not initialized"); // TODO: Fix panic
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
                return;
            }
        }

        // Not in to_propagate, therefore undo what was done in propagate
        for d in 0..self.directions_count {
            let topology = self.get_topology(ctx);

            let result = topology.borrow().try_move_full(index, Direction::from_index(d).unwrap());
            if result.is_err() {
                panic!("try_move_full {}", result.unwrap_err());
            }
            if let Some((i2, id, el)) = result.unwrap() {
                let patterns = &self.propagator_array[pattern as usize][el as usize];
                for &p in patterns {
                    self.compatible[i2][p][id as usize] += 1;
                }
            }
        }
    }

    fn do_select(&mut self, ctx: &Context<T>, index: usize, pattern: i32) {
        if !self.initialized {
            panic!("Ac4PatternModelConstraint not initialized"); // TODO: Fix panic
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
    }

    fn propagate(&mut self, ctx: &Context<T>) {
        if !self.initialized {
            panic!("Ac4PatternModelConstraint not initialized"); // TODO: Fix panic
        }

        while !self.to_propagate.is_empty() {
            let item = self.to_propagate.pop().unwrap();

            // Get coordinates first
            let (x, y, z) = {
                let topology = self.get_topology(ctx);

                let result = topology.borrow().get_coord(item.index as usize);
                if result.is_err() {
                    // TODO: Fix panic
                    panic!("get_coord failed {}", result.unwrap_err());
                }
                result.unwrap()
            };

            if item.pattern >= 0 {
                // Process a ban
                for d in 0..self.directions_count {
                    // Get movement result and patterns in separate scope
                    let (move_result, patterns) = {
                        let topology = self.get_topology(ctx);

                        let move_result = topology.borrow().try_move_coord_full(x, y, z, Direction::from_index(d).unwrap());
                        let patterns = if let Ok(Some((_, _, el))) = move_result {
                            self.propagator_array[item.pattern as usize][el as usize].clone()
                        } else {
                            Vec::new()
                        };
                        (move_result, patterns)
                    };

                    if let Ok(Some((i2, id, _el))) = move_result {
                        self.propagate_ban_core(ctx, &patterns, i2, id as usize);
                    }
                }
            } else {
                // Process a select
                let pattern = (!item.pattern) as usize;
                for d in 0..self.directions_count {
                    // Get movement result and patterns_dense in separate scope
                    let (move_result, patterns_dense) = {
                        let topology = self.get_topology(ctx);

                        let move_result = topology.borrow().try_move_coord_full(x, y, z, get_direction(d));
                        let patterns_dense = if let Ok(Some((_, _, el))) = move_result {
                            self.propagator_array_dense[pattern][el as usize].clone()
                        } else {
                            Vec::new()
                        };
                        (move_result, patterns_dense)
                    };

                    if let Ok(Some((i2, id, _el))) = move_result {
                        self.propagate_select_core(ctx, &patterns_dense, i2, id as usize);
                    }
                }
            }

            // Check status after processing
            // TODO: Fix this unsafe stuff
            let propagator_cell = self.get_wave_propagator(ctx);
            let propagator = propagator_cell.as_ptr();
            unsafe {
                // println!("Checking status");
                if (*propagator).status() == Resolution::Contradiction {
                    println!("Status checked");
                    return;
                }
            }
            // let propagator = self.get_wave_propagator(ctx);
            // if propagator.borrow().status() == Resolution::Contradiction {
            //     return;
            // }
        }
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
    fn propagate_ban_core(&mut self, ctx: &Context<T>, patterns: &[usize], i2: usize, d: usize) {
        if !self.initialized {
            panic!("Ac4PatternModelConstraint not initialized"); // TODO: Fix panic
        }

        // Hot loop
        for &p in patterns {
            self.compatible[i2][p][d] -= 1;
            let c = self.compatible[i2][p][d];
            // Have we just now ruled out this possible pattern?
            if c == 0 {
                let propagator = self.get_wave_propagator(ctx);
                // TODO: Fix the dependency chain so we don't have to do this unsafe stuff.
                let propagator_ptr = propagator.as_ptr();
                unsafe {
                    // println!("Calling internal_ban");
                    if (*propagator_ptr).internal_ban(ctx, i2, p).unwrap() {
                        println!("Calling set_contradiction");
                        (*propagator_ptr).set_contradiction();
                        println!("Called set_contradiction");
                    }
                }
                // let propagator = self.get_wave_propagator(ctx);
                // if propagator.borrow_mut().internal_ban(ctx, i2, p).unwrap() {
                //     propagator.borrow_mut().set_contradiction();
                // }
            }
        }
    }

    fn propagate_select_core(&mut self, ctx: &Context<T>, patterns_dense: &[bool], i2: usize, id: usize) {
        if !self.initialized {
            panic!("Ac4PatternModelConstraint not initialized"); // TODO: Fix panic
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
                let propagator_cell = self.get_wave_propagator(ctx);
                let propagator = propagator_cell.as_ptr();
                unsafe {
                    // println!("Calling internal_ban");
                    if (*propagator).internal_ban(ctx, i2, p).unwrap() {
                        println!("Calling set_contradiction");
                        (*propagator).set_contradiction();
                        println!("Called set_contradiction");
                    }
                }
                // let propagator = self.get_wave_propagator(ctx);
                // if propagator.borrow_mut().internal_ban(ctx, i2, p).unwrap() {
                //     propagator.borrow_mut().set_contradiction();
                // }
            }
        }
    }
}