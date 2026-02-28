use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;
use crate::models::tile_model_mapping::TileModelMapping;
use crate::resolution::Resolution;
use crate::tile_propagator::TilePropagatorState;
use crate::topology::topology::Topology;
use crate::trackers::entropy_tracker::EntropyTracker;
use crate::trackers::index_picker::IndexPicker;
use crate::trackers::pattern_picker::PatternPicker;
use crate::trackers::tracker::{ChoiceObserver, SuperTracker};
use crate::trackers::weighted_random_pattern_picker::WeightedRandomPatternPicker;
use crate::wfc::act_4_pattern_model_constraint::Ac4PatternModelConstraint;
use crate::wfc::backtrack_policy::BacktrackPolicy;
use crate::wfc::pattern_model::PatternModel;
use crate::wfc::pattern_model_constraint::{Ac3PatternModelConstraint, OneStepPatternModelConstraint, PatternModelConstraint};
use crate::wfc::wave::Wave;

// Compile time optimizations
pub struct Optimizations;

impl Optimizations {
    pub const QUICK_SELECT: bool = false;
}

pub struct WaveConstraint {} // TODO: implement

// TODO: implement
#[derive(Debug, Clone, Copy)]
pub struct IndexPatternItem {
    // Can also take value -1 in some circumstances to indicate that we've saved the state,
    // but no particular choice was made.
    pub index: i32,
    pub pattern: i32,
}
impl IndexPatternItem {
    pub fn new(index: i32, pattern: i32) -> Self {
        IndexPatternItem { index, pattern }
    }
}

pub enum ModelConstraintAlgorithm {
    Default,
    Ac4,
    Ac3,
    OneStep,
}

pub struct WavePropagator<T: Topology + Clone> {
    state: WavePropagatorState<T>,
    // Main data tracking what we've decided so far
    // wave: Option<Wave>,

    pattern_model_constraint: Box<dyn PatternModelConstraint<T>>,

    // From model
    pattern_count: usize,
    // frequencies: Vec<f64>,

    // Used for backtracking
    backtrack_items: Option<VecDeque<IndexPatternItem>>,
    backtrack_items_lengths: Option<VecDeque<usize>>,
    prev_choices: Option<VecDeque<IndexPatternItem>>,
    dropped_backtrack_items_count: usize,
    backtrack_count: i32,
    backjump_count: i32,

    // Basic parameters
    index_count: usize,
    backtrack: bool,
    max_backtrack_depth: i32,
    constraints: Vec<WaveConstraint>,
    // random_double: Rc<dyn Fn() -> f64>,

    // Deferred constraints
    deferred_constraints_step: bool,

    // The overall status
    // status: Resolution,
    contradiction_reason: Option<String>,
    contradiction_source: Option<String>,

    // pub topology: TopologyId, // was pub topology: Arc<Mutex<Box<dyn Topology>>>,
    directions_count: usize,

    trackers: Vec<Box<dyn SuperTracker<T>>>,
    choice_observers: Vec<Box<dyn ChoiceObserver>>,
    index_picker: Box<dyn IndexPicker<T>>,
    pattern_picker: Box<dyn PatternPicker<T>>,
    backtrack_policy: Option<Box<dyn BacktrackPolicy<T>>>,
}

// Options struct for WavePropagator
pub struct WavePropagatorOptions<T: Topology + Clone> {
    pub backtrack_policy: Option<Box<dyn BacktrackPolicy<T>>>,
    pub max_backtrack_depth: i32,
    pub constraints: Option<Vec<WaveConstraint>>,
    pub random_double: Option<Rc<dyn Fn() -> f64>>,
    pub index_picker: Option<Box<dyn IndexPicker<T>>>,
    pub pattern_picker: Option<Box<dyn PatternPicker<T>>>,
    pub clear: bool,
    pub model_constraint_algorithm: ModelConstraintAlgorithm,
}

pub struct WavePropagatorState<T: Topology + Clone> {
    wave: Option<Wave>,
    frequencies: Vec<f64>,
    random_double: Rc<dyn Fn() -> f64>,
    status: Resolution,

    phantom_data: PhantomData<T>,
}

impl<T> WavePropagatorState<T> where T: Topology + Clone {
    pub fn get_random_double(&self) -> Rc<dyn Fn() -> f64> {
        self.random_double.clone()
    }
    pub fn get_frequencies(&self) -> Vec<f64> {
        self.frequencies.clone()
    }

    pub fn status(&self) -> Resolution {
        self.status
    }

    pub fn get_wave(&self) -> &Option<Wave> {
        &self.wave // TODO: Refactor to return &self.wave.as_ref()?
    }
}

impl<T: Topology + Clone + 'static> WavePropagator<T> {
    pub fn new(
        model: PatternModel,
        options: WavePropagatorOptions<T>,
        tile_propagator_state: &TilePropagatorState<T>,
    ) -> Result<Self, String> {
        let pattern_count = model.pattern_count();
        let frequencies = model.frequencies().clone();
        let index_count = tile_propagator_state.get_topology().index_count();
        let backtrack = options.backtrack_policy.is_some();
        let directions_count = tile_propagator_state.get_topology().directions_count();

        let random_double = options.random_double.unwrap_or(Rc::new(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            use std::time::{SystemTime, UNIX_EPOCH};

            let mut hasher = DefaultHasher::new();
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
            let hash = hasher.finish();
            (hash as f64) / (u64::MAX as f64)
        }));

        let index_picker = options.index_picker.unwrap_or_else(|| {
            Box::new(EntropyTracker::new())
        });
        let pattern_picker = options.pattern_picker.unwrap_or_else(|| {
            Box::new(WeightedRandomPatternPicker::new())
        });

        let propagator_state = WavePropagatorState {
            wave: None,
            frequencies,
            random_double: random_double.clone(),
            status: Resolution::Undecided,
            phantom_data: PhantomData,
        };

        let mut wave_propagator = Self {
            state: propagator_state,
            // wave: None,
            pattern_model_constraint: Box::new(OneStepPatternModelConstraint),
            pattern_count,
            // frequencies,
            backtrack_items: if backtrack { Some(VecDeque::new()) } else { None },
            backtrack_items_lengths: if backtrack { Some(VecDeque::new()) } else { None },
            prev_choices: if backtrack { Some(VecDeque::new()) } else { None },
            dropped_backtrack_items_count: 0,
            backtrack_count: 0,
            backjump_count: 0,
            index_count,
            backtrack,
            max_backtrack_depth: options.max_backtrack_depth,
            constraints: options.constraints.unwrap_or_else(Vec::new),
            // random_double,
            deferred_constraints_step: false,
            // status: Resolution::Undecided,
            contradiction_reason: None,
            contradiction_source: None,
            directions_count,
            trackers: Vec::new(),
            // index_picker_trackers: Vec::new(),
            // pattern_picker_trackers: Vec::new(),
            choice_observers: Vec::new(),
            index_picker,
            pattern_picker,
            backtrack_policy: options.backtrack_policy,
        };


        let pattern_model_constraint: Box<dyn PatternModelConstraint<T>> = match options.model_constraint_algorithm {
            ModelConstraintAlgorithm::OneStep => Box::new(OneStepPatternModelConstraint),
            ModelConstraintAlgorithm::Default | ModelConstraintAlgorithm::Ac4 => Box::new(Ac4PatternModelConstraint::new(tile_propagator_state.get_topology(), &model)),
            ModelConstraintAlgorithm::Ac3 => Box::new(Ac3PatternModelConstraint),
        };

        wave_propagator.pattern_model_constraint = pattern_model_constraint;
        if options.clear {
            wave_propagator.clear(tile_propagator_state)?;
        }

        Ok(wave_propagator)
    }

    /// Repeatedly step until the status is Decided or Contradiction
    pub fn run(&mut self, tile_model_mapping: &TileModelMapping<T>, topology: &T) -> Result<Resolution, String> {
        loop {
            let status = self.step(tile_model_mapping, topology)?;
            if status != Resolution::Undecided {
                return Ok(status);
            }
        }
    }

    pub fn clear(&mut self, tile_propagator_state: &TilePropagatorState<T>) -> Result<Resolution, String> {
        let wave = Wave::new(self.pattern_count, self.index_count);
        self.state.wave = Some(wave);

        if self.backtrack {
            self.backtrack_items = Some(VecDeque::new());
            self.backtrack_items_lengths = Some(VecDeque::new());
            if let Some(ref mut lengths) = self.backtrack_items_lengths {
                lengths.push_back(0);
            }
            self.prev_choices = Some(VecDeque::new());
        }

        self.state.status = Resolution::Undecided;
        self.contradiction_reason = None;
        self.contradiction_source = None;
        self.trackers.clear();
        self.choice_observers.clear();

        // Initialize pickers
        let picker_ref: &mut dyn IndexPicker<T> = &mut *self.index_picker;
        IndexPicker::init(picker_ref, &self.state, tile_propagator_state.get_topology())?;

        let picker_ref = &mut *self.pattern_picker;
        PatternPicker::init(picker_ref, &self.state, tile_propagator_state.get_topology())?;

        if let Some(mut policy) = self.backtrack_policy.take() {
            let res = policy.init(self);
            self.backtrack_policy = Some(policy);
            res?
        }
        // if let Some(ref mut policy) = self.backtrack_policy {
        //     policy.init(self)?;
        // }

        let mut pattern_model_constraint_owner = mem::replace(&mut self.pattern_model_constraint, Box::new(OneStepPatternModelConstraint));;
        let pattern_model_constraint: &mut dyn PatternModelConstraint<T> = &mut *pattern_model_constraint_owner;
        let result = pattern_model_constraint.clear(tile_propagator_state.get_topology(), &self.state)?;
        if result.is_some() {
            let (idx_to_ban, pattern_to_ban) = result.unwrap(); // Safe because checked above
            if self.internal_ban(idx_to_ban, pattern_to_ban, pattern_model_constraint).unwrap() { // TODO: Does this catch the panic?
                self.set_contradiction();
            }
        }

        if self.state.status == Resolution::Contradiction {
            self.pattern_model_constraint = pattern_model_constraint_owner;
            return Ok(self.state.status);
        }

        self.init_constraints(tile_propagator_state.get_topology(), pattern_model_constraint).ok();

        self.pattern_model_constraint = pattern_model_constraint_owner;
        Ok(self.state.status)
    }

    fn init_constraints(&mut self, topology: &T, pattern_model_constraint: &mut dyn PatternModelConstraint<T>) -> Result<(), String> {
        // Note: constraints would need to be handled differently in Rust
        // This is a simplified version
        let constraints_len = self.constraints.len();

        for _i in 0..constraints_len {
            // constraint.init(self);
            {
                if self.state.status != Resolution::Undecided {
                    return Ok(());
                }
            }

            pattern_model_constraint.propagate(topology, self);

            {
                if self.state.status != Resolution::Undecided {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub fn step(&mut self, tile_model_mapping: &TileModelMapping<T>, topology: &T) -> Result<Resolution, String> {
        let mut pattern_model_constraint_owner = mem::replace(&mut self.pattern_model_constraint, Box::new(OneStepPatternModelConstraint));;
        let pattern_model_constraint: &mut dyn PatternModelConstraint<T> = &mut *pattern_model_constraint_owner;
        // println!("wave_propagator.step");
        // Check if we need to step constraints
        if self.deferred_constraints_step {
            self.step_constraints(topology, pattern_model_constraint);
        }

        // If we're already in a final state, skip making an observation
        if self.state.status != Resolution::Undecided {
            self.try_backtrack_until_no_contradiction(topology, pattern_model_constraint)?;
            self.pattern_model_constraint = pattern_model_constraint_owner;
            return Ok(self.state.status);
        }

        let (index, pattern) = {
            // Pick an index to use
            let index = {
                self.index_picker.get_random_index(&self.state, tile_model_mapping)
                    .ok_or("unable to get random index from pattern picker")?
            };

            if index == -1 {
                // No more indices to process
                if self.state.status == Resolution::Undecided {
                    self.state.status = Resolution::Decided;
                }
                self.pattern_model_constraint = pattern_model_constraint_owner;
                return Ok(self.state.status);
            }

            let mut pattern: Option<usize> = None;
            if index != -1 {
                // Pick a pattern to select at that index
                pattern = {
                    let random_fn = self.state.random_double.clone();
                    self.pattern_picker.get_random_possible_pattern_at(index as usize, &self.state)
                };
            }

            (index, pattern)
        };

        if let Some(pattern) = pattern {
            self.record_backtrack(index, pattern as i32)?;

            // Use the pick
            if self.internal_select(index as usize, pattern, pattern_model_constraint).ok_or("unable to internal_select")? {
                self.state.status = Resolution::Contradiction;
            }
        }

        /*
            let mut pattern_model_constraint = mem::replace(&mut self.pattern_model_constraint, Box::new(OneStepPatternModelConstraint));
            pattern_model_constraint.propagate(topology, self)?;
            self.pattern_model_constraint = pattern_model_constraint;
         */
        // Re-evaluate status
        if self.state.status == Resolution::Undecided {
            pattern_model_constraint.propagate(topology, self)?;
        }
        if self.state.status == Resolution::Undecided {
            self.step_constraints(topology, pattern_model_constraint);
        }

        self.try_backtrack_until_no_contradiction(topology, pattern_model_constraint)?;
        self.pattern_model_constraint = pattern_model_constraint_owner;
        Ok(self.state.status)
    }

    pub fn step_constraints(&mut self, topology: &T, pattern_model_constraint: &mut dyn PatternModelConstraint<T>) {
        let constraints_len = self.constraints.len();

        for _i in 0..constraints_len {
            // constraint.check(self);
            {
                if self.state.status != Resolution::Undecided {
                    return;
                }
            }

            pattern_model_constraint.propagate(topology, self);
            {
                if self.state.status != Resolution::Undecided {
                    return;
                }
            }
        }

        self.deferred_constraints_step = false;
    }

    fn record_backtrack(&mut self, index: i32, pattern: i32) -> Result<(), String> {
        // Extract everything we need in one lock acquisition
        let (backtrack_enabled, current_length, max_depth) = {
            let current_length = self.dropped_backtrack_items_count +
                self.backtrack_items.as_ref().map_or(0, |items| items.len());

            (self.backtrack, current_length, self.max_backtrack_depth)
        };

        if !backtrack_enabled {
            return Ok(());
        }

        // Record the backtrack point with the pre-calculated length
        {
            if let Some(ref mut lengths) = self.backtrack_items_lengths {
                lengths.push_back(current_length);
            }

            if let Some(ref mut choices) = self.prev_choices {
                choices.push_back(IndexPatternItem::new(index, pattern));
            }
        }

        // Notify choice observers using the cloned vector
        for co in &mut self.choice_observers {
            co.make_choice(index as usize, pattern as usize);
        }

        // Clean up backtracks if they are too long
        while max_depth > 0 {
            let lengths_count = {
                self.backtrack_items_lengths.as_ref().map_or(0, |l| l.len())
            };

            if lengths_count <= max_depth as usize {
                break;
            }

            // Extract the dropped count before mutable borrow
            let dropped_count = self.dropped_backtrack_items_count;
            if let Some(ref mut lengths) = self.backtrack_items_lengths {
                let new_dropped_count = lengths.pop_front().ok_or("unable to pop front from backtrack_items_lengths")?;
                if let Some(ref mut choices) = self.prev_choices {
                    choices.pop_front().ok_or("unable to pop front from prev_choices")?;
                }
                if let Some(ref mut items) = self.backtrack_items {
                    items.drain(..(new_dropped_count - dropped_count).min(items.len()));
                }
                self.dropped_backtrack_items_count = new_dropped_count;
            }
        }

        Ok(())
    }

    // Internal API for constraints
    pub fn internal_ban(&mut self, index: usize, pattern: usize, pattern_model_constraint: &mut dyn PatternModelConstraint<T>) -> Option<bool> {
        if self.state.wave.is_none() {
            println!("WavePropagator.internal_ban: wave is None!");
            return None;
        }

        // Record information for backtracking
        if self.backtrack {
            if let Some(ref mut backtrack_items) = self.backtrack_items {
                backtrack_items.push_back(IndexPatternItem::new(index as i32, pattern as i32));
            }
        }

        pattern_model_constraint.do_ban(index, pattern as i32);

        // Update the wave
        let is_contradiction = self.state.wave.as_mut().unwrap().remove_possibility(index, pattern); // Safe because checked above

        // Update trackers
        for tracker in &mut self.trackers {
            let res = tracker.do_ban(index, pattern);
            Self::process_tracker_result(res, "do_ban");
        }
        if self.index_picker.as_super_tracker().is_some() {
            let res = self.index_picker.as_super_tracker_mut().unwrap().do_ban(index, pattern); // Safe because checked above
            Self::process_tracker_result(res, "do_ban");
        }
        if self.pattern_picker.as_super_tracker().is_some() {
            let res = self.pattern_picker.as_super_tracker_mut().unwrap().do_ban(index, pattern); // Safe because checked above
            Self::process_tracker_result(res, "do_ban");
        }

        Some(is_contradiction)
    }

    fn process_tracker_result(res: Result<(), String>, method_name: &str) {
        if res.is_err() {
            let err = res.err().unwrap(); // Safe because checked above
            if !err.contains("is not a Tracker") {
                panic!("Unable to {} on super tracker: {}", method_name, err);
            }
        }
    }

    pub fn internal_select(&mut self, index: usize, chosen_pattern: usize, pattern_model_constraint: &mut dyn PatternModelConstraint<T>) -> Option<bool> {
        if self.state.wave.is_none() {
            println!("WavePropagator.internal_select: wave is None!");
            return None;
        }

        if !Optimizations::QUICK_SELECT {
            let pattern_count = self.pattern_count;
            for pattern in 0..pattern_count {
                if pattern == chosen_pattern {
                    continue;
                }
                // Check if pattern is available
                let pattern_available = {
                    self.state.wave.as_ref().unwrap().get(index, pattern) // Safe because checked above
                };

                if pattern_available {
                    if self.internal_ban(index, pattern, pattern_model_constraint)? {
                        return Some(true);
                    }
                }
            }
            return Some(false);
        }

        // Quick select path - collect all patterns to ban first
        let patterns_to_ban: Vec<usize> = {
            let mut patterns = Vec::new();
            for pattern in 0..self.pattern_count {
                if pattern != chosen_pattern {
                    if self.state.wave.as_mut().unwrap().get(index, pattern) { // Safe because checked above
                        patterns.push(pattern);
                    }
                }
            }
            patterns
        };

        // Now ban each pattern (this will acquire locks individually)
        for pattern in patterns_to_ban {
            if self.internal_ban(index, pattern, pattern_model_constraint)? {
                return Some(true);
            }
        }

        // Do the select operation on the constraint
        {
            pattern_model_constraint.do_select(index, chosen_pattern as i32);
        }

        Some(false)
    }

    fn try_backtrack_until_no_contradiction(&mut self, topology: &T, pattern_model_constraint: &mut dyn PatternModelConstraint<T>) -> Result<(), String> {
        if !self.backtrack {
            return Ok(());
        }

        while self.state.status == Resolution::Contradiction {
            let backjump_amount = self.backtrack_policy
                .as_mut()
                .ok_or("unable to get backtrack policy")
                ?.get_backjump()
                .ok_or("unable to get backjump")?;

            for _i in 0..backjump_amount {
                let lengths_count = {
                    self.backtrack_items_lengths.as_ref().map_or(0, |l| l.len())
                };

                if lengths_count == 1 {
                    // We've backtracked as much as we can
                    return Ok(());
                }

                // Actually undo various bits of state
                let item = {
                    self.do_backtrack(topology)?;
                    let item = if let Some(ref mut choices) = self.prev_choices {
                        choices.pop_back() // Or pop_front?
                    } else {
                        None
                    };

                    self.state.status = Resolution::Undecided;
                    self.contradiction_reason = None;
                    self.contradiction_source = None;
                    item
                };

                // Update choice observers
                {
                    for co in &mut self.choice_observers {
                        co.backtrack();
                    }
                }

                if backjump_amount == 1 {
                    self.backtrack_count += 1;

                    // Mark the given choice as impossible
                    if let Some(item) = item {
                        if item.index >= 0 {
                            if self.internal_ban(item.index as usize, item.pattern as usize, pattern_model_constraint).ok_or("unable to internal_ban")? {
                                self.state.status = Resolution::Contradiction;
                            }
                        }
                    }
                }
            }

            if backjump_amount > 1 {
                self.backjump_count += 1;
            }

            // Revalidate status
            if self.state.status == Resolution::Undecided {
                // let mut pattern_model_constraint = mem::replace(&mut self.pattern_model_constraint, Box::new(OneStepPatternModelConstraint));
                pattern_model_constraint.propagate(topology, self);
                // self.pattern_model_constraint = pattern_model_constraint;
            }
            if self.state.status == Resolution::Undecided {
                self.step_constraints(topology, pattern_model_constraint);
            }
        }

        Ok(())
    }

    // Undoes any work that was done since the last backtrack point
    fn do_backtrack(&mut self, topology: &T) -> Result<(), String> {
        let target_length = if let Some(ref mut lengths) = self.backtrack_items_lengths {
            lengths.pop_back().unwrap_or(0) - self.dropped_backtrack_items_count
        } else {
            0
        };

        // Collect all items to undo first
        let mut items_to_undo = Vec::new();
        if let Some(ref mut items) = self.backtrack_items {
            while items.len() > target_length {
                if let Some(item) = items.pop_back() {
                    items_to_undo.push(item);
                }
            }
        }

        // Clone tracker references to avoid borrowing conflicts
        // let tracker_refs: Vec<_> = self.trackers.iter().cloned().collect();

        // Now process each item
        for item in &items_to_undo {
            let index = item.index as usize;
            let pattern = item.pattern as usize;

            // Add the possibility back
            self.state.wave.as_mut().ok_or("unable to get wave")?.add_possibility(index, pattern);

            // Undo the pattern model constraint
            self.pattern_model_constraint.undo_ban(index, pattern as i32, topology)?;

            // Update trackers
            for tracker in &mut self.trackers {
                let res = tracker.do_ban(index, pattern);
                Self::process_tracker_result(res, "do_ban");
            }
            if self.index_picker.as_super_tracker().is_some() {
                let res = self.index_picker.as_super_tracker_mut().unwrap().do_ban(index, pattern); // Safe because checked above
                Self::process_tracker_result(res, "do_ban");
            }
            if self.pattern_picker.as_super_tracker().is_some() {
                let res = self.pattern_picker.as_super_tracker_mut().unwrap().do_ban(index, pattern); // Safe because checked above
                Self::process_tracker_result(res, "do_ban");
            }
        }

        Ok(())
    }

    /// Returns the only possible value of a cell if there is only one,
    /// otherwise returns -1 (multiple possible) or -2 (none possible)
    pub fn get_decided_pattern(&self, index: usize) -> Option<i32> {
        if self.state.wave.is_none() {
            return Some(Resolution::Contradiction as i32);
        }
        let mut decided_pattern = Resolution::Contradiction as i32;
        for pattern in 0..self.pattern_count {
            if self.state.wave.as_ref()?.get(index, pattern) {
                if decided_pattern == Resolution::Contradiction as i32 {
                    decided_pattern = pattern as i32;
                } else {
                    return Some(Resolution::Undecided as i32);
                }
            }
        }
        Some(decided_pattern)
    }

    pub fn add_choice_observer(&mut self, observer: Box<dyn ChoiceObserver>) {
        self.choice_observers.push(observer);
    }

    pub fn add_tracker(&mut self, tracker: Box<dyn SuperTracker<T>>) {
        // if tracker.is_index_picker() {
        //     self.index_picker_trackers.push(tracker);
        // } else {
        //     self.pattern_picker_trackers.push(tracker);
        // }
        self.trackers.push(tracker);
    }

    pub fn get_random_double(&self) -> Rc<dyn Fn() -> f64> {
        self.state.random_double.clone()
    }

    pub fn get_state(&self) -> &WavePropagatorState<T> {
        &self.state
    }
}

// getters
impl<T: Topology + Clone> WavePropagator<T> {
    pub fn get_frequencies(&self) -> Vec<f64> {
        self.state.frequencies.clone()
    }

    pub fn status(&self) -> Resolution {
        self.state.status
    }

    pub fn set_contradiction(&mut self) {
        self.state.status = Resolution::Contradiction;
    }

    pub fn get_wave(&self) -> &Option<Wave> {
        &self.state.wave
    }
}