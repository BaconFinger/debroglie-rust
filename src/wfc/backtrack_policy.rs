use std::marker::PhantomData;
use crate::topology::topology::Topology;
use crate::trackers::tracker::ChoiceObserver;
use crate::wfc::wave_propagator::WavePropagator;

pub trait BacktrackPolicy<T: Topology + Clone> {
    fn init(&mut self, wave_propagator: &mut WavePropagator<T>) -> Result<(), String>;

    /// Returns:
    /// 0  = Give up
    /// 1  = Backtrack
    /// >1 = Backjump
    fn get_backjump(&mut self) -> Option<i32>;
}

pub struct ConstantBacktrackPolicy<T: Topology + Clone> {
    amount: i32,
    _phantom: PhantomData<T>,
}

impl<T: Topology + Clone> ConstantBacktrackPolicy<T> {
    pub fn new(amount: i32) -> Self {
        ConstantBacktrackPolicy { amount, _phantom: PhantomData }
    }
}

impl<T> BacktrackPolicy<T> for ConstantBacktrackPolicy<T> where T: Topology + Clone {
    fn init(&mut self, _wave_propagator: &mut WavePropagator<T>) -> Result<(), String> {
        // Empty implementation
        Ok(())
    }

    fn get_backjump(&mut self) -> Option<i32> {
        Some(self.amount)
    }
}

/// After 10 failed backtracks, backjumps by 4 (level 0) and repeats.
/// Each subsequent level backtracks twice as far, but waits twice as long to trigger.
/// This means that after a backjump, we try exactly as hard the 2nd time as we did the first, including
/// trying smaller backjumps.
/// Whenever forward progress is made, all levels are reset.
pub struct PatienceBackjumpPolicy<T: Topology + Clone> {
    counter: i64,
    depth: i32,
    max_depth: i32,
    start: i64,
    levels: Option<Vec<Level>>,
    _phantom: PhantomData<T>,
}

impl<T: Topology + Clone> PatienceBackjumpPolicy<T> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            depth: 0,
            max_depth: 0,
            start: 0,
            levels: None,
            _phantom: PhantomData
        }
    }

    fn create_level(&self, level: i32) -> Level {
        Level {
            depth: self.max_depth - 4 * 2_i32.pow(level as u32),
            timeout: self.start + 10 * 2_i64.pow(level as u32),
        }
    }

    fn reset_level(&mut self, level: usize) {
        if let Some(ref mut levels) = self.levels {
            levels[level].timeout = self.counter + 10 * 2_i64.pow(level as u32);
        }
    }
}

impl<T: Topology + Clone + 'static> BacktrackPolicy<T> for PatienceBackjumpPolicy<T> {
    fn init(&mut self, wave_propagator: &mut WavePropagator<T>) -> Result<(), String> {
        let choice_observer = PatienceChoiceObserver {
            policy: self as *mut PatienceBackjumpPolicy<T>,
        };
        // wave_propagator.add_choice_observer(Box::new(choice_observer));
        self.counter = 0;
        self.depth = 0;
        self.max_depth = 0;
        self.start = 0;

        Ok(())
    }

    fn get_backjump(&mut self) -> Option<i32> {
        if self.levels.is_none() {
            self.levels = Some(Vec::new());
        }

        // Find first non-expired level
        let mut i = 0;
        {
            let levels = self.levels.as_ref()?;
            while i < levels.len() {
                if levels[i].timeout > self.counter {
                    break;
                }
                i += 1;
            }
        }

        // Lazily add higher levels as needed
        let level = self.create_level(i as i32);
        let levels = self.levels.as_mut()?;
        if levels.len() <= i {
            levels.push(level);
        }

        if i == 0 {
            return Some(1);
        }

        // Get the target depth before we start mutating
        let target_depth = levels[i - 1].depth;

        // Reset any expired levels
        for j in 0..i {
            self.reset_level(j);
        }

        Some(self.depth - target_depth)
    }
}

impl<T: Topology + Clone> ChoiceObserver for PatienceBackjumpPolicy<T> {
    fn make_choice(&mut self, _index: usize, _pattern: usize) {
        self.depth += 1;
        if self.depth > self.max_depth {
            self.max_depth = self.depth;
            // Reset levels
            self.levels = None;
            self.start = self.counter;
        }
    }

    fn backtrack(&mut self) {
        self.counter += 1;
        self.depth -= 1;
    }
}

// Helper struct to implement ChoiceObserver for the policy
// This is needed because we need to share the policy between the wave propagator and the observer
struct PatienceChoiceObserver<T: Topology + Clone> {
    policy: *mut PatienceBackjumpPolicy<T>,
}

impl<T: Topology + Clone> ChoiceObserver for PatienceChoiceObserver<T> {
    fn make_choice(&mut self, index: usize, pattern: usize) {
        unsafe {
            (*self.policy).make_choice(index, pattern);
        }
    }

    fn backtrack(&mut self) {
        unsafe {
            (*self.policy).backtrack();
        }
    }
}

#[derive(Debug, Clone)]
struct Level {
    depth: i32,
    timeout: i64,
}

impl<T: Topology + Clone> Default for PatienceBackjumpPolicy<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Mock trait definition - you'll need to implement this based on your actual WavePropagator
pub trait WavePropagatorChoiceObserver {
    fn add_choice_observer(&mut self, observer: Box<dyn ChoiceObserver>);
}