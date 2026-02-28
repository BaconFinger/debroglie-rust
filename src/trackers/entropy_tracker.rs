use crate::models::tile_model_mapping::TileModelMapping;
use crate::topology::topology::Topology;
use crate::trackers::index_picker::IndexPicker;
use crate::trackers::pattern_picker::PatternPicker;
use crate::trackers::tracker::{SuperTracker, Tracker};
use crate::wfc::wave::Wave;
use crate::wfc::wave_propagator::WavePropagatorState;

pub struct EntropyTracker {
    pattern_count: usize,
    frequencies: Vec<f64>,
    entropy_values: Vec<EntropyValues>,
    plogp: Vec<f64>,
    mask: Option<Vec<bool>>,
    indices: usize,
}

impl EntropyTracker {
    pub fn new() -> Self {
        Self {
            pattern_count: 0,
            frequencies: Vec::new(),
            entropy_values: Vec::new(),
            plogp: Vec::new(),
            mask: None,
            indices: 0,
        }
    }

    // For debugging
    pub fn init_debug<T: Topology + Clone>(&mut self, wave: &Wave, frequencies: Vec<f64>, mask: Option<Vec<bool>>) -> Result<(), String> {
        self.frequencies = frequencies;
        self.pattern_count = self.frequencies.len();
        self.mask = mask;
        self.indices = wave.indices();

        // Initialize plogp
        self.plogp = vec![0.0; self.pattern_count];
        for pattern in 0..self.pattern_count {
            let f = self.frequencies[pattern];
            let v = if f > 0.0 { f * f.ln() } else { 0.0 };
            self.plogp[pattern] = v;
        }

        self.entropy_values = vec![EntropyValues::default(); self.indices];
        self.reset();

        Ok(())
    }
}

impl<T: Topology + Clone + 'static> IndexPicker<T> for EntropyTracker {
    fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), String> {
        {
            self.init_debug::<T>(
                wave_propagator_state.get_wave().as_ref().ok_or("unable to get wave id")?,
                wave_propagator_state.get_frequencies(),
                topology.mask(),
            )?
        }
        Ok(())
    }

    fn get_random_index(&mut self, wave_propagator_state: &WavePropagatorState<T>, tile_model_mapping: &TileModelMapping<T>) -> Option<i32> {
        let mut selected_index = -1i32;
        let mut min_entropy = f64::INFINITY;
        let mut count_at_min_entropy = 0;

        let wave = wave_propagator_state.get_wave().as_ref()?;

        for i in 0..self.indices {
            if let Some(ref mask) = self.mask {
                if !mask[i] {
                    continue;
                }
            }

            let c = wave.get_pattern_count(i);
            let e = self.entropy_values[i].entropy;

            if c <= 1 {
                continue;
            } else if e < min_entropy {
                count_at_min_entropy = 1;
                min_entropy = e;
            } else if e == min_entropy {
                count_at_min_entropy += 1;
            }
        }

        let mut n = (count_at_min_entropy as f64 * wave_propagator_state.get_random_double()()) as i32;

        for i in 0..self.indices {
            if let Some(ref mask) = self.mask {
                if !mask[i] {
                    continue;
                }
            }

            let c = wave.get_pattern_count(i);
            let e = self.entropy_values[i].entropy;

            if c <= 1 {
                continue;
            } else if e == min_entropy {
                if n == 0 {
                    selected_index = i as i32;
                    break;
                }
                n -= 1;
            }
        }

        Some(selected_index)
    }

    fn as_super_tracker(&self) -> Option<&dyn SuperTracker<T>> {
        Some(self as &dyn SuperTracker<T>)
    }

    fn as_super_tracker_mut(&mut self) -> Option<&mut dyn SuperTracker<T>> {
        Some(self as &mut dyn SuperTracker<T>)
    }
}

impl Tracker for EntropyTracker {
    fn reset(&mut self) -> Result<(), String> {
        // Assumes Reset is called on a truly new Wave.
        let mut initial = EntropyValues::default();
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

        for index in 0..self.indices {
            self.entropy_values[index] = initial;
        }
        Ok(())
    }

    fn do_ban(&mut self, index: usize, pattern: usize) -> Result<(), String> {
        self.entropy_values[index].decrement(
            self.frequencies[pattern],
            self.plogp[pattern],
        );

        Ok(())
    }

    fn undo_ban(&mut self, index: usize, pattern: usize) -> Result<(), String> {
        self.entropy_values[index].increment(
            self.frequencies[pattern],
            self.plogp[pattern],
        );

        Ok(())
    }
}

impl<T: Topology + Clone + 'static> PatternPicker<T> for EntropyTracker {
    fn init(&mut self, wave_propagator_state: &WavePropagatorState<T>, topology: &T) -> Result<(), String> {
        unimplemented!("EntropyTracker is not a PatternPicker")
    }

    fn get_random_possible_pattern_at(&mut self, index: usize, wave_propagator_state: &WavePropagatorState<T>) -> Option<usize> {
        unimplemented!("EntropyTracker is not a PatternPicker")
    }

    fn as_super_tracker(&self) -> Option<&dyn SuperTracker<T>> {
        Some(self as &dyn SuperTracker<T>)
    }

    fn as_super_tracker_mut(&mut self) -> Option<&mut dyn SuperTracker<T>> {
        Some(self as &mut dyn SuperTracker<T>)
    }
}

impl<T: Topology + Clone + 'static> SuperTracker<T> for EntropyTracker {
    fn is_index_picker(&self) -> bool {
        true
    }

    fn is_pattern_picker(&self) -> bool {
        false
    }

    fn is_tracker(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct EntropyValues {
    plogp_sum: f64,     // The sum of p'(pattern) * log(p'(pattern)).
    sum: f64,           // The sum of p'(pattern).
    entropy: f64,       // The entropy of the cell.
}

impl EntropyValues {
    fn recompute_entropy(&mut self) {
        self.entropy = self.sum.ln() - self.plogp_sum / self.sum;
    }

    fn decrement(&mut self, p: f64, plogp: f64) {
        self.plogp_sum -= plogp;
        self.sum -= p;
        self.recompute_entropy();
    }

    fn increment(&mut self, p: f64, plogp: f64) {
        self.plogp_sum += plogp;
        self.sum += p;
        self.recompute_entropy();
    }
}