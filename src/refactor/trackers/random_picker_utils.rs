use std::rc::Rc;
use crate::refactor::wfc::wave::Wave;
use crate::refactor::topology::topology::Topology;

pub struct RandomPickerUtils;

impl RandomPickerUtils {
    pub fn get_random_possible_pattern<T: Topology + Clone>(
        wave: &Wave,
        random_double: Rc<dyn Fn() -> f64>,
        index: usize,
        frequencies: &[f64],
    ) -> Option<usize>
    {
        let pattern_count = frequencies.len();
        let mut s = 0.0; // The total frequency of all existing patterns

        for pattern in 0..pattern_count {
            if wave.get(index, pattern) {
                s += frequencies[pattern];
            }
        }

        let mut r = random_double() * s;

        // Randomly get a pattern weighted by how frequently it occurs.
        for pattern in 0..pattern_count {
            if wave.get(index, pattern) {
                r -= frequencies[pattern];
            }
            if r <= 0.0 {
                return Some(pattern);
            }
        }

        Some(pattern_count - 1)
    }

    pub fn get_random_possible_pattern_with_patterns<T: Topology + Clone>(
        wave: &Wave,
        random_double: Rc<dyn Fn() -> f64>,
        index: usize,
        frequencies: &[f64],
        patterns: Option<&[usize]>,
    ) -> Option<usize>
    {
        if patterns.is_none() {
            return Self::get_random_possible_pattern::<T>(wave, random_double, index, frequencies);
        }

        let patterns = patterns.unwrap();
        let mut s = 0.0;
        let pattern_count = frequencies.len();

        for i in 0..pattern_count {
            let pattern = patterns[i];
            if wave.get(index, pattern) {
                s += frequencies[i];
            }
        }

        let mut r = random_double() * s;

        for i in 0..pattern_count {
            let pattern = patterns[i];
            if wave.get(index, pattern) {
                r -= frequencies[i];
            }
            if r <= 0.0 {
                return Some(pattern);
            }
        }

        Some(patterns[patterns.len() - 1])
    }
}