use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Instant};
use crate::refactor::resolution::Resolution;
use crate::refactor::topology::topology::Topology;
use crate::refactor::trackers::entropy_tracker::EntropyTracker;
use crate::refactor::trackers::index_picker::IndexPicker;
use crate::refactor::trackers::pattern_picker::PatternPicker;
use crate::refactor::trackers::tracker::{ChoiceObserver, SuperTracker};
use crate::refactor::trackers::weighted_random_pattern_picker::WeightedRandomPatternPicker;
use crate::refactor::wfc::act_4_pattern_model_constraint::Ac4PatternModelConstraint;
use crate::refactor::wfc::backtrack_policy::BacktrackPolicy;
use crate::refactor::wfc::pattern_model::PatternModel;
use crate::refactor::wfc::pattern_model_constraint::{Ac3PatternModelConstraint, OneStepPatternModelConstraint, PatternModelConstraint};
use crate::refactor::wfc::wave::Wave;

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
    // Main data tracking what we've decided so far
    wave: Option<WaveId>,

    pattern_model_constraint: Box<dyn PatternModelConstraint<T>>,

    // From model
    pattern_count: usize,
    frequencies: Vec<f64>,

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
    constraints: Vec<WaveConstraintId>, // was constraints: Vec<Box<dyn WaveConstraint>>,
    random_double: Rc<dyn Fn() -> f64>,

    // Deferred constraints
    deferred_constraints_step: bool,

    // The overall status
    status: Resolution,
    contradiction_reason: Option<String>,
    contradiction_source: Option<String>,

    pub topology: TopologyId, // was pub topology: Arc<Mutex<Box<dyn Topology>>>,
    directions_count: usize,

    trackers: Vec<TrackerId>, // was trackers: Vec<Arc<Mutex<dyn Tracker>>>,
    choice_observers: Vec<ChoiceObserverId>, // was Vec<Arc<Mutex<dyn ChoiceObserver>>>,

    index_picker: TrackerId, // was Arc<Mutex<Box<dyn IndexPicker>>>,
    pattern_picker: TrackerId, // was Arc<Mutex<Box<dyn PatternPicker>>>,
    backtrack_policy: Option<BacktrackPolicyId>, // was Option<Box<dyn BacktrackPolicy>>,

    self_ref: Option<WavePropagatorId>
}

// Options struct for WavePropagator
pub struct WavePropagatorOptions {
    pub backtrack_policy: Option<BacktrackPolicyId>, // was Option<Box<dyn BacktrackPolicy>>,
    pub max_backtrack_depth: i32,
    pub constraints: Option<Vec<WaveConstraintId>>, // was Option<Vec<Box<dyn WaveConstraint>>>,
    pub random_double: Option<Rc<dyn Fn() -> f64>>,
    pub index_picker: Option<TrackerId>, // was Option<Arc<Mutex<Box<dyn IndexPicker>>>>,
    pub pattern_picker: Option<TrackerId>, // was Option<Arc<Mutex<Box<dyn PatternPicker>>>>,
    pub clear: bool,
    pub model_constraint_algorithm: ModelConstraintAlgorithm,
}

impl<T: Topology + Clone> WavePropagator<T> {
    pub fn new(
        model: PatternModel,
        topology: TopologyId,
        options: WavePropagatorOptions,
    ) -> Result<WavePropagatorId, String> {
        let topology_ref = get_topology(ctx, topology.clone()).ok_or("unable to get topology")?;
        let pattern_count = model.pattern_count();
        let frequencies = model.frequencies().clone();
        let index_count = topology_ref.borrow().index_count();
        let backtrack = options.backtrack_policy.is_some();
        let directions_count = topology_ref.borrow().directions_count();

        // TODO: make more configurable and ideally seedable
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
            let default_tracker = EntropyTracker::new();
            ctx.index_pickers().add(Rc::new(RefCell::new(default_tracker)))
        });
        let pattern_picker = options.pattern_picker.unwrap_or_else(|| {
            let default_picker = WeightedRandomPatternPicker::new();
            ctx.pattern_pickers().add(Rc::new(RefCell::new(default_picker)))
        });

        let wave_propagator = Self {
            wave: None,
            pattern_model_constraint: Box::new(OneStepPatternModelConstraint),
            pattern_count,
            frequencies,
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
            random_double,
            deferred_constraints_step: false,
            status: Resolution::Undecided,
            contradiction_reason: None,
            contradiction_source: None,
            topology,
            directions_count,
            trackers: Vec::new(),
            choice_observers: Vec::new(),
            index_picker,
            pattern_picker,
            backtrack_policy: options.backtrack_policy,
            self_ref: None
        };

        let id = ctx.wave_propagators().add(wave_propagator);
        let wave_propagator = get_wave_propagator(ctx, id).unwrap(); // Safe since we just added it
        let pattern_model_constraint: Box<dyn PatternModelConstraint<T>> = match options.model_constraint_algorithm {
            ModelConstraintAlgorithm::OneStep => Box::new(OneStepPatternModelConstraint),
            ModelConstraintAlgorithm::Default | ModelConstraintAlgorithm::Ac4 => Box::new(Ac4PatternModelConstraint::new(ctx, id, &model)),
            ModelConstraintAlgorithm::Ac3 => Box::new(Ac3PatternModelConstraint),
        };
        let mut wave_propagator_ref = wave_propagator.borrow_mut();
        wave_propagator_ref.set_self_ref(id);
        wave_propagator_ref.pattern_model_constraint = pattern_model_constraint;
        if options.clear {
            wave_propagator_ref.clear(ctx)?;
        }

        Ok(id)
    }

    /// Repeatedly step until the status is Decided or Contradiction
    pub fn run(&mut self) -> Result<Resolution, String> {
        let now = Instant::now();
        loop {
            let status = self.step(ctx)?;
            if status != Resolution::Undecided {
                return Ok(status);
            }
        }
    }

    pub fn clear(&mut self, ctx: &Context<T>) -> Result<Resolution, String> {
        if self.self_ref.is_none() {
            return Err("WavePropagator::clear called without self_ref set! It must be added to the Context first!".to_string());
        }

        let wave = Wave::new(self.pattern_count, self.index_count);
        self.wave = Some(ctx.wave().add(wave));

        if self.backtrack {
            self.backtrack_items = Some(VecDeque::new());
            self.backtrack_items_lengths = Some(VecDeque::new());
            if let Some(ref mut lengths) = self.backtrack_items_lengths {
                lengths.push_back(0);
            }
            self.prev_choices = Some(VecDeque::new());
        }

        self.status = Resolution::Undecided;
        self.contradiction_reason = None;
        self.contradiction_source = None;
        self.trackers.clear();
        self.choice_observers.clear();

        // Initialize pickers
        let index_picker_rc = self.get_index_picker(ctx).ok_or("unable to get index picker")?;
        let mut index_picker = index_picker_rc.borrow_mut();
        let picker_ref = &mut *index_picker;
        IndexPicker::init(picker_ref, ctx, self)?;

        let pattern_picker_rc = self.get_pattern_picker(ctx).ok_or("unable to get index picker")?;
        let mut pattern_picker = pattern_picker_rc.borrow_mut();
        let picker_ref = &mut *pattern_picker;
        PatternPicker::init(picker_ref, self)?;

        if let Some(ref mut policy) = self.backtrack_policy {
            self.get_backtrack_policy(ctx).ok_or("unable to get backtrack policy")?.borrow_mut().init(ctx, self)?;
        }

        self.pattern_model_constraint.clear(ctx);

        if self.status == Resolution::Contradiction {
            return Ok(self.status);
        }

        self.init_constraints(ctx).ok();

        Ok(self.status)
    }

    fn init_constraints(&mut self, ctx: &Context<T>) -> Result<(), String> {
        // Note: constraints would need to be handled differently in Rust
        // This is a simplified version
        let constraints_len = self.constraints.len();

        for _i in 0..constraints_len {
            // constraint.init(self);
            {
                if self.status != Resolution::Undecided {
                    return Ok(());
                }
            }

            self.pattern_model_constraint.propagate(ctx);

            {
                if self.status != Resolution::Undecided {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub fn step(&mut self, ctx: &Context<T>) -> Result<Resolution, String> {
        // println!("wave_propagator.step");
        // Check if we need to step constraints
        if self.deferred_constraints_step {
            self.step_constraints(ctx);
        }

        // If we're already in a final state, skip making an observation
        if self.status != Resolution::Undecided {
            self.try_backtrack_until_no_contradiction(ctx)?;
            return Ok(self.status);
        }

        let (index, pattern) = {
            // Pick an index to use
            let index = {
                let random_fn = self.random_double.clone();
                self.get_index_picker(ctx)
                    .ok_or("unable to get index picker")
                    ?.borrow_mut()
                    .get_random_index(ctx, random_fn)
                    .ok_or("unable to get random index from pattern picker")?
            };

            if index == -1 {
                // No more indices to process
                if self.status == Resolution::Undecided {
                    self.status = Resolution::Decided;
                }
                return Ok(self.status);
            }

            let mut pattern: Option<usize> = None;
            if index != -1 {
                // Pick a pattern to select at that index
                pattern = {
                    let random_fn = self.random_double.clone();
                    self.get_pattern_picker(ctx)
                        .ok_or("unable to get pattern picker")
                        ?.borrow_mut()
                        .get_random_possible_pattern_at(ctx, index as usize, random_fn)
                };
            }

            (index, pattern)
        };

        if let Some(pattern) = pattern {
            // println!("pattern is {}", pattern);
            self.record_backtrack(ctx, index, pattern as i32)?;

            // Use the pick
            if self.internal_select(ctx, index as usize, pattern).ok_or("unable to internal_select")? {
                // println!("wave_propagator:548");
                self.status = Resolution::Contradiction;
            }
        }

        // Re-evaluate status
        if self.status == Resolution::Undecided {
            self.pattern_model_constraint.propagate(ctx);
        }
        if self.status == Resolution::Undecided {
            self.step_constraints(ctx);
        }

        // if (index == -1 || index == 10) && self.status == Resolution::Undecided {
        //     self.status = Resolution::Decided;
        //     return Ok(self.status);
        // }

        self.try_backtrack_until_no_contradiction(ctx)?;
        Ok(self.status)
    }

    pub fn step_constraints(&mut self, ctx: &Context<T>) {
        let constraints_len = self.constraints.len();

        for _i in 0..constraints_len {
            // constraint.check(self);
            {
                if self.status != Resolution::Undecided {
                    return;
                }
            }

            self.pattern_model_constraint.propagate(ctx);
            {
                if self.status != Resolution::Undecided {
                    return;
                }
            }
        }

        self.deferred_constraints_step = false;
    }

    fn record_backtrack(&mut self, ctx: &Context<T>, index: i32, pattern: i32) -> Result<(), String> {
        // Extract everything we need in one lock acquisition
        let (backtrack_enabled, current_length, choice_observers, max_depth) = {
            let current_length = self.dropped_backtrack_items_count +
                self.backtrack_items.as_ref().map_or(0, |items| items.len());

            (self.backtrack, current_length, self.choice_observers.clone(), self.max_backtrack_depth)
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
        for co in &choice_observers {
            self.get_choice_observer(ctx, co.clone())
                .ok_or("unable to get choice observer")
                ?.borrow_mut().make_choice(index as usize, pattern as usize);
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
    pub fn internal_ban(&mut self, ctx: &Context<T>, index: usize, pattern: usize) -> Option<bool> {
        // Record information for backtracking
        if self.backtrack {
            if let Some(ref mut backtrack_items) = self.backtrack_items {
                backtrack_items.push_back(IndexPatternItem::new(index as i32, pattern as i32));
            }
        }

        self.pattern_model_constraint.do_ban(ctx, index, pattern as i32);

        // Update the wave
        let is_contradiction = self.get_wave(ctx)?.borrow_mut().remove_possibility(index, pattern);

        // Update trackers
        for tracker_id in &self.trackers {
            if let Some(tracker) = self.get_tracker(ctx, tracker_id.clone()) {
                tracker.borrow_mut().do_ban(index, pattern);
                continue;
            }

            if let Some(tracker) = self.get_index_picker_2(ctx, tracker_id.clone()) {
                tracker.borrow_mut().do_ban(index, pattern);
                continue;
            }

            if let Some(tracker) = self.get_pattern_picker_2(ctx, tracker_id.clone()) {
                tracker.borrow_mut().do_ban(index, pattern);
                continue;
            }
        }

        Some(is_contradiction)
    }

    pub fn internal_select(&mut self, ctx: &Context<T>, index: usize, chosen_pattern: usize) -> Option<bool> {
        // Simple, inefficient way
        if !Optimizations::QUICK_SELECT {
            let pattern_count = self.pattern_count;
            for pattern in 0..pattern_count {
                if pattern == chosen_pattern {
                    continue;
                }
                // Check if pattern is available
                let pattern_available = {
                    self.get_wave(ctx)?.borrow().get(index, pattern)
                };

                if pattern_available {
                    if self.internal_ban(ctx, index, pattern)? {
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
                    if self.get_wave(ctx)?.borrow().get(index, pattern) {
                        patterns.push(pattern);
                    }
                }
            }
            patterns
        };

        // Now ban each pattern (this will acquire locks individually)
        for pattern in patterns_to_ban {
            if self.internal_ban(ctx, index, pattern)? {
                return Some(true);
            }
        }

        // Do the select operation on the constraint
        {
            self.pattern_model_constraint.do_select(ctx, index, chosen_pattern as i32);
        }

        Some(false)
    }

    fn try_backtrack_until_no_contradiction(&mut self, ctx: &Context<T>) -> Result<(), String> {
        if !self.backtrack {
            return Ok(());
        }

        while self.status == Resolution::Contradiction {
            let backjump_amount = self.get_backtrack_policy(ctx)
                .ok_or("unable to get backtrack policy")
                ?.borrow_mut()
                .get_backjump()
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
                    self.do_backtrack(ctx)?;
                    let item = if let Some(ref mut choices) = self.prev_choices {
                        choices.pop_back() // Or pop_front?
                    } else {
                        None
                    };

                    self.status = Resolution::Undecided;
                    self.contradiction_reason = None;
                    self.contradiction_source = None;
                    item
                };

                // Update choice observers
                {
                    for co in &self.choice_observers {
                        self.get_choice_observer(ctx, co.clone())
                            .ok_or("unable to get choice observer")
                            ?.borrow_mut()
                            .backtrack();
                    }
                }

                if backjump_amount == 1 {
                    self.backtrack_count += 1;

                    // Mark the given choice as impossible
                    if let Some(item) = item {
                        if item.index >= 0 {
                            if self.internal_ban(ctx, item.index as usize, item.pattern as usize).ok_or("unable to internal_ban")? {
                                // println!("wave_propagator:689");
                                self.status = Resolution::Contradiction;
                            }
                        }
                    }
                }
            }

            if backjump_amount > 1 {
                self.backjump_count += 1;
            }

            // Revalidate status
            if self.status == Resolution::Undecided {
                self.pattern_model_constraint.propagate(ctx);
            }
            if self.status == Resolution::Undecided {
                self.step_constraints(ctx);
            }
        }

        Ok(())
    }

    // Undoes any work that was done since the last backtrack point
    fn do_backtrack(&mut self, ctx: &Context<T>) -> Result<(), String> {
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
        let tracker_refs: Vec<_> = self.trackers.iter().cloned().collect();

        // Now process each item
        for item in &items_to_undo {
            let index = item.index as usize;
            let pattern = item.pattern as usize;

            // Add the possibility back
            self.get_wave(ctx).ok_or("unable to get wave")?.borrow_mut().add_possibility(index, pattern);

            // Undo the pattern model constraint
            self.pattern_model_constraint.undo_ban(ctx, index, pattern as i32);

            // Update trackers
            for tracker_id in &tracker_refs {
                if let Some(tracker) = self.get_tracker(ctx, tracker_id.clone()) {
                    tracker.borrow_mut().do_ban(index, pattern);
                    continue;
                }

                if let Some(tracker) = self.get_index_picker_2(ctx, tracker_id.clone()) {
                    tracker.borrow_mut().do_ban(index, pattern);
                    continue;
                }

                if let Some(tracker) = self.get_pattern_picker_2(ctx, tracker_id.clone()) {
                    tracker.borrow_mut().do_ban(index, pattern);
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Returns the only possible value of a cell if there is only one,
    /// otherwise returns -1 (multiple possible) or -2 (none possible)
    pub fn get_decided_pattern(&self, ctx: &Context<T>, index: usize) -> Option<i32> {
        if self.wave.is_none() {
            return Some(Resolution::Contradiction as i32);
        }
        let wave = self.get_wave(ctx)?;
        let mut decided_pattern = Resolution::Contradiction as i32;
        for pattern in 0..self.pattern_count {
            if wave.borrow().get(index, pattern) {
                if decided_pattern == Resolution::Contradiction as i32 {
                    decided_pattern = pattern as i32;
                } else {
                    return Some(Resolution::Undecided as i32);
                }
            }
        }
        Some(decided_pattern)
    }

    pub fn set_self_ref(&mut self, id: WavePropagatorId) {
        self.self_ref = Some(id);
    }

    pub fn add_choice_observer(&mut self, observer: ChoiceObserverId) {
        self.choice_observers.push(observer);
    }

    pub fn add_tracker(&mut self, tracker: TrackerId) {
        self.trackers.push(tracker);
    }

    pub fn get_random_double(&self) -> Rc<dyn Fn() -> f64> {
        self.random_double.clone()
    }
}

// getters
impl<T: Topology + Clone> WavePropagator<T> {
    pub fn get_wave_id(&self) -> Option<WaveId> {
        self.wave.clone()
    }

    pub fn get_frequencies(&self) -> Vec<f64> {
        self.frequencies.clone()
    }

    pub fn status(&self) -> Resolution {
        self.status
    }

    pub fn set_contradiction(&mut self) {
        self.status = Resolution::Contradiction;
    }

    pub fn get_wave(&self, ctx: &Context<T>) -> Option<Rc<RefCell<Wave>>> {
        ctx
            .wave()
            .get(self.wave.unwrap()) // Panicking to catch logic errors here; there should always be a wave.
    }

    fn get_index_picker(&self, ctx: &Context<T>) -> Option<Rc<RefCell<dyn SuperTracker<T>>>> {
        let picker = ctx
            .index_pickers()
            .get(self.index_picker)
            ?.clone();

        if !picker.borrow().is_index_picker() {
            return None;
        }

        Some(picker.clone())
    }

    fn get_pattern_picker(&self, ctx: &Context<T>) -> Option<Rc<RefCell<dyn SuperTracker<T>>>> {
        let picker = ctx
            .pattern_pickers()
            .get(self.pattern_picker)
            ?.clone();

        if !picker.borrow().is_pattern_picker() {
            return None;
        }

        Some(picker.clone())
    }

    fn get_choice_observer(&self, ctx: &Context<T>, id: ChoiceObserverId) -> Option<Rc<RefCell<dyn ChoiceObserver>>> {
        ctx
            .choice_observers()
            .get(id)
    }

    fn get_tracker(&self, ctx: &Context<T>, id: TrackerId) -> Option<Rc<RefCell<dyn SuperTracker<T>>>> {
        let tracker = ctx
            .trackers()
            .get(id)
            ?.clone();

        if !tracker.borrow().is_tracker() {
            return None;
        }

        Some(tracker.clone())
    }

    // TODO: Better name
    fn get_index_picker_2(&self, ctx: &Context<T>, id: TrackerId) -> Option<Rc<RefCell<dyn SuperTracker<T>>>> {
        let picker = ctx
            .index_pickers()
            .get(id)
            ?.clone();

        if !picker.borrow().is_index_picker() {
            return None;
        }

        Some(picker.clone())
    }

    // TODO: Better name
    fn get_pattern_picker_2(&self, ctx: &Context<T>, id: TrackerId) -> Option<Rc<RefCell<dyn SuperTracker<T>>>> {
        let picker = ctx
            .pattern_pickers()
            .get(id)
            ?.clone();

        if !picker.borrow().is_pattern_picker() {
            return None;
        }

        Some(picker.clone())
    }

    fn get_backtrack_policy(&self, ctx: &Context<T>) -> Option<Rc<RefCell<dyn BacktrackPolicy<T>>>> {
        ctx
            .backtrack_policy()
            .get(self.backtrack_policy?)
    }
}

fn get_topology<T: Topology + Clone>(ctx: &Context<T>, id: TopologyId) -> Option<Rc<RefCell<T>>> {
    ctx
        .topologies()
        .clone()
        .get(id)
}

fn get_wave_propagator<T: Topology + Clone>(ctx: &Context<T>, id: WavePropagatorId) -> Option<Rc<RefCell<WavePropagator<T>>>> {
    ctx
        .wave_propagators()
        .clone()
        .get(id)
}

// fn get_index_picker<T: Topology + Clone>(ctx: &Context<T>, id: TrackerId) -> Rc<RefCell<dyn IndexPicker<T>>> {
//     ctx
//         .index_pickers()
//         .get(id)
//         .unwrap()
// }
//
// fn get_pattern_picker<T: Topology + Clone>(ctx: &Context<T>, id: TrackerId) -> Rc<RefCell<dyn PatternPicker<T>>> {
//     ctx
//         .pattern_pickers()
//         .get(id)
//         .unwrap()
// }