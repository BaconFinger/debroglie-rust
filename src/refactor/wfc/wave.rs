/// Wave is a fancy array that tracks various per-cell information.
/// Most importantly, it tracks possibilities - which patterns are possible to put
/// into which cells.
/// It has no notion of cell adjacency, cells are just referred to by integer index.
#[derive(Debug,Clone)]
pub struct Wave {
    pattern_count: usize,

    // possibilities[index*pattern_count + pattern] is true if we haven't eliminated putting
    // that pattern at that index.
    possibilities: Vec<bool>,

    pattern_counts: Vec<usize>,

    indices: usize,
}

impl Wave {
    pub fn new(pattern_count: usize, indices: usize) -> Self {
        let possibilities = vec![true; indices * pattern_count];
        let pattern_counts = vec![pattern_count; indices];

        Wave {
            pattern_count,
            possibilities,
            pattern_counts,
            indices,
        }
    }

    pub fn indices(&self) -> usize {
        self.indices
    }

    pub fn get(&self, index: usize, pattern: usize) -> bool {
        self.possibilities[index * self.pattern_count + pattern]
    }

    pub fn get_pattern_count(&self, index: usize) -> usize {
        self.pattern_counts[index]
    }

    /// Returns true if there is a contradiction
    pub fn remove_possibility(&mut self, index: usize, pattern: usize) -> bool {
        debug_assert!(self.possibilities[index * self.pattern_count + pattern]);

        self.possibilities[index * self.pattern_count + pattern] = false;
        self.pattern_counts[index] -= 1;

        self.pattern_counts[index] == 0
    }

    pub fn add_possibility(&mut self, index: usize, pattern: usize) {
        debug_assert!(!self.possibilities[index * self.pattern_count + pattern]);

        self.possibilities[index * self.pattern_count + pattern] = true;
        self.pattern_counts[index] += 1;
    }

    // TODO: This should respect mask. Maybe move out of Wave
    pub fn get_progress(&self) -> f64 {
        // TODO: Use pattern_count info?
        let c = self.possibilities.iter().filter(|&&b| !b).count();

        // We're basically done when we've banned all but one pattern for each index
        c as f64 / (self.pattern_count - 1) as f64 / self.indices as f64
    }
}