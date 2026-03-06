use std::cell::RefCell;
use std::rc::Rc;
use ordered_float::OrderedFloat;
use crate::heap::{Heap, HeapNode};
use crate::shared_mut_heap::{SharedMutHeap};
use crate::models::tile_model_mapping::TileModelMapping;
use crate::topology::topology::{Topology, TopologyError};
use crate::trackers::change_tracker::{ChangeTracker, ChangeTrackerError};
use crate::trackers::index_picker::IndexPicker;
use crate::trackers::pattern_picker::PatternPicker;
use crate::trackers::tracker::{Tracker};
use crate::trait_error::TraitError;
use crate::wfc::wave_propagator::WavePropagatorState;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HeapEntropyTrackerError {
    #[error("no wave")]
    NoWave,

    #[error("random_double missing")]
    RandomDoubleMissing,

    #[error(transparent)]
    ChangeTrackerError(#[from] ChangeTrackerError),
}

pub struct HeapEntropyTracker<T: Topology + Clone> {
    pattern_count: usize,
    frequencies: Vec<f64>,
    entropy_values: Vec<Rc<RefCell<EntropyValues>>>,
    plogp: Vec<f64>,
    mask: Option<Vec<bool>>,
    random_double: Option<Rc<dyn Fn() -> f64>>,
    index_count: usize,
    heap: Option<SharedMutHeap<EntropyValues, OrderedFloat<f64>>>,
    tracker: Option<ChangeTracker<T>>,
}

impl<T: Topology + Clone + 'static> HeapEntropyTracker<T> {
    pub fn new() -> Box<dyn Tracker<T>> {
        let me = Self::new_empty();
        Box::new(me)
    }

    pub fn new_empty() -> Self {
        Self {
            pattern_count: 0,
            frequencies: Vec::new(),
            entropy_values: Vec::new(),
            plogp: Vec::new(),
            mask: None,
            random_double: None,
            index_count: 0,
            heap: None,
            tracker: None,
        }
    }

    // For debugging
    pub fn init_debug(
        &mut self,
        wave_propagator_state: &WavePropagatorState<T>,
        mask: Option<Vec<bool>>,
    ) -> Result<(), TraitError> {
        self.frequencies = wave_propagator_state.get_frequencies();
        self.pattern_count = self.frequencies.len();
        self.mask = mask;
        self.random_double = Some(wave_propagator_state.get_random_double());
        self.index_count = wave_propagator_state.get_wave().as_ref().ok_or(HeapEntropyTrackerError::NoWave)?.indices();

        // Initialize plogp
        self.plogp = vec![0.0; self.pattern_count];
        for pattern in 0..self.pattern_count {
            let f = self.frequencies[pattern];
            let v = if f > 0.0 { f * f.ln() } else { 0.0 };
            self.plogp[pattern] = v;
        }

        self.entropy_values = vec![Rc::new(RefCell::new(EntropyValues::new())); self.index_count];
        self.heap = Some(SharedMutHeap::with_capacity(self.index_count));
        self.tracker = Some(ChangeTracker::with_index_count(self.index_count));

        self.reset()?;

        Ok(())
    }

    const THRESHOLD: f64 = 1e-17;
}

impl<T: Topology + Clone + 'static> Tracker<T> for HeapEntropyTracker<T> {
    fn reset(&mut self) -> Result<(), TraitError> {
        // Assumes Reset is called on a truly new Wave.
        let mut initial = EntropyValues::new();
        initial.plogp_sum = 0.0;
        initial.sum = 0.0;
        initial.entropy = 0.0;

        for pattern in 0..self.pattern_count {
            let f = self.frequencies[pattern];
            let v = if f > 0.0 { f * f.ln() } else { 0.0 };
            initial.plogp_sum += v;
            initial.sum += f;
        }
        initial.recompute_entropy();

        if let Some(ref mut heap) = self.heap {
            heap.clear();
        }

        let random_fn = self.random_double.as_ref().ok_or(HeapEntropyTrackerError::RandomDoubleMissing)?;

        for index in 0..self.index_count {
            if self.mask.as_ref().map_or(true, |m| m[index]) {
                let mut ev = EntropyValues::from_other(&initial);
                ev.index = index;
                ev.tiebreaker = random_fn() * 1e-10;
                let ev = Rc::new(RefCell::new(ev));

                if self.pattern_count > 1 {
                    if let Some(ref mut heap) = self.heap {
                        heap.insert(ev.clone());
                    }
                }
                self.entropy_values[index] = ev;
            }
        }

        if let Some(ref mut tracker) = self.tracker {
            tracker.reset()?;
        }

        Ok(())
    }

    fn do_ban(&mut self, index: usize, pattern: usize) -> Result<(), TraitError> {
        self.entropy_values[index].borrow_mut().decrement(
            self.frequencies[pattern],
            self.plogp[pattern],
        );
        // self.sync_heap_evs();

        if let Some(ref mut tracker) = self.tracker {
            tracker.do_ban(index, pattern)?;
        }

        Ok(())
    }

    fn undo_ban(&mut self, index: usize, pattern: usize) -> Result<(), TraitError> {
        self.entropy_values[index].borrow_mut().increment(
            self.frequencies[pattern],
            self.plogp[pattern],
        );
        // self.sync_heap_evs();

        if let Some(ref mut tracker) = self.tracker {
            tracker.undo_ban(index, pattern)?;
        }

        Ok(())
    }

    fn as_pattern_picker(&self) -> Option<&dyn PatternPicker<T>> {
        None
    }

    fn as_pattern_picker_mut(&mut self) -> Option<&mut dyn PatternPicker<T>> {
        None
    }

    fn as_index_picker(&self) -> Option<&dyn IndexPicker<T>> {
        Some(self)
    }

    fn as_index_picker_mut(&mut self) -> Option<&mut dyn IndexPicker<T>> {
        Some(self)
    }
}

impl<T: Topology + Clone + 'static> IndexPicker<T> for HeapEntropyTracker<T> {
    fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), TraitError> {
        self.init_debug(
            wave_propagator_state,
            topology.mask(),
        )?;
        Ok(())
    }

    fn get_random_index(&mut self, wave_propagator_state: &WavePropagatorState<T>, tile_model_mapping: &TileModelMapping<T>) -> Option<i32> {
        let changed_indices = if let Some(mut tracker) = self.tracker.take() {
            let indices = tracker.get_changed_indices(tile_model_mapping).unwrap_or_default();
            self.tracker = Some(tracker);
            indices
        } else {
            println!("No tracker!");
            return None;
        };

        let wave = wave_propagator_state.get_wave().as_ref()?;

        if changed_indices.len() > (wave.indices() as f64 * 0.5) as usize && changed_indices.len() > 1 {
            // A lot of indices have changed
            // It's faster to rebuild the entire heap than sync it one at a time
            for &index in &changed_indices {
                self.entropy_values[index].borrow_mut().recompute_entropy();
            }

            let mut items = Vec::new();
            for index in 0..self.index_count {
                if self.mask.as_ref().map_or(true, |m| m[index]) {
                    let c = wave.get_pattern_count(index);
                    if c <= 1 {
                        self.entropy_values[index].borrow_mut().set_heap_index(None); // Equivalent to -1
                    } else {
                        items.push(self.entropy_values[index].clone());
                    }
                }
            }

            self.heap = Some(SharedMutHeap::from_vec(items));
        } else {
            // Sync heap with new values of entropy
            for &index in &changed_indices {
                let ev = &mut self.entropy_values[index];
                ev.borrow_mut().recompute_entropy();

                let c = wave.get_pattern_count(index);
                let heap = self.heap.as_mut()?; // HERE
                // for mut heap_ev in heap.data.iter_mut() {
                //     if heap_ev.identifier == ev.identifier {
                //         heap_ev.update(ev);
                //     }
                // }

                // if ev.heap_index == usize::MAX {
                if ev.borrow_mut().heap_index == None {
                    if c > 1 {
                        heap.insert(ev.clone());
                    }
                } else if c <= 1 {
                    heap.delete(ev.borrow_mut().heap_index.unwrap()); // TODO: Fix unwrap
                    ev.borrow_mut().set_heap_index(None);
                } else {
                    let to_change = ev.borrow_mut().heap_index.unwrap().clone();
                    heap.changed_key(to_change); // TODO: Fix unwrap
                }
            }

        }

        let heap = self.heap.as_ref()?;
        if heap.is_empty() {
            return Some(-1);
        }

        let mouthful = heap.peek()?;
        let item = mouthful.borrow_mut();
        Some(item.index.clone() as i32)
    }

    fn as_tracker(&self) -> Option<&dyn Tracker<T>> {
        Some(self)
    }

    fn as_tracker_mut(&mut self) -> Option<&mut dyn Tracker<T>> {
        Some(self)
    }

    fn as_pattern_picker(&self) -> Option<&dyn PatternPicker<T>> {
        None
    }

    fn as_pattern_picker_mut(&mut self) -> Option<&mut dyn PatternPicker<T>> {
        None
    }

    // fn as_super_tracker(&self) -> Option<&dyn SuperTracker<T>> {
    //     Some(self as &dyn SuperTracker<T>)
    // }
    //
    // fn as_super_tracker_mut(&mut self) -> Option<&mut dyn SuperTracker<T>> {
    //     Some(self as &mut dyn SuperTracker<T>)
    // }
}

impl<T: Topology + Clone + 'static> Default for HeapEntropyTracker<T> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<T: Topology + Clone + 'static> HeapEntropyTracker<T> {
    pub fn as_index_picker(het: Rc<RefCell<Self>>) -> Rc<RefCell<dyn IndexPicker<T>>> {
        het
    }
}

// impl<T: Topology + Clone + 'static> PatternPicker<T> for HeapEntropyTracker<T> {
//     fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), String> {
//         unimplemented!("HeapEntropyTracker is not a PatternPicker")
//     }
//
//     fn get_random_possible_pattern_at(&mut self, index: usize, wave_propagator_state: &WavePropagatorState<T>) -> Option<usize> {
//         unimplemented!("HeapEntropyTracker is not a PatternPicker")
//     }
//
//     fn as_super_tracker(&self) -> Option<&dyn SuperTracker<T>> {
//         Some(self as &dyn SuperTracker<T>)
//     }
//
//     fn as_super_tracker_mut(&mut self) -> Option<&mut dyn SuperTracker<T>> {
//         Some(self as &mut dyn SuperTracker<T>)
//     }
// }

// impl<T: Topology + Clone + 'static> SuperTracker<T> for HeapEntropyTracker<T> {
//     fn is_index_picker(&self) -> bool {
//         true
//     }
//
//     fn is_pattern_picker(&self) -> bool {
//         false
//     }
//
//     fn is_tracker(&self) -> bool {
//         true
//     }
// }

#[derive(Debug, Clone)]
pub(crate) struct EntropyValues {
    pub(crate) plogp_sum: f64,     // The sum of p'(pattern) * log(p'(pattern)).
    pub(crate) sum: f64,           // The sum of p'(pattern).
    pub(crate) entropy: f64,       // The entropy of the cell.
    pub(crate) index: usize,
    pub(crate) heap_index: Option<usize>,
    pub(crate) tiebreaker: f64,
}

impl EntropyValues {
    pub(crate) fn new() -> Self {
        Self {
            plogp_sum: 0.0,
            sum: 0.0,
            entropy: 0.0,
            index: 0,
            heap_index: Some(0),
            tiebreaker: 0.0,
        }
    }

    pub(crate) fn from_other(other: &EntropyValues) -> Self {
        Self {
            plogp_sum: other.plogp_sum,
            sum: other.sum,
            entropy: other.entropy,
            index: other.index,
            heap_index: other.heap_index,
            tiebreaker: other.tiebreaker,
        }
    }

    pub(crate) fn update(&mut self, other: &EntropyValues) {
        self.plogp_sum = other.plogp_sum;
        self.sum = other.sum;
        self.entropy = other.entropy;
        self.index = other.index;
        self.heap_index = other.heap_index;
        self.tiebreaker = other.tiebreaker;
    }

    pub(crate) fn recompute_entropy(&mut self) {
        self.entropy = self.sum.ln() - self.plogp_sum / self.sum;
    }

    pub(crate) fn decrement(&mut self, p: f64, plogp: f64) {
        self.plogp_sum -= plogp;
        self.sum -= p;
    }

    pub(crate)fn increment(&mut self, p: f64, plogp: f64) {
        self.plogp_sum += plogp;
        self.sum += p;
    }
}

impl HeapNode<OrderedFloat<f64>> for EntropyValues {
    fn heap_index(&self) -> Option<usize> {
        self.heap_index
    }

    fn set_heap_index(&mut self, index: Option<usize>) {
        self.heap_index = index;
    }

    fn key(&self) -> OrderedFloat<f64> {
        OrderedFloat(self.entropy + self.tiebreaker)
        // Create a temporary value for the key (entropy + tiebreaker)
        // Note: This is a bit awkward because we need to return a reference
        // but we're computing the value. In practice, you might want to
        // store the computed key as a field and update it when needed.
        // static mut TEMP_KEY: OrderedFloat<f64> = OrderedFloat(0.0);
        // unsafe {
        //     TEMP_KEY = OrderedFloat(self.entropy) + OrderedFloat(self.tiebreaker);
        //     &TEMP_KEY
        // }
    }

    fn index(&self) -> usize {
        self.index
    }
}