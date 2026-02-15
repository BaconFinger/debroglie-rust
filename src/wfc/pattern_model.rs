#[derive(Clone)]
pub struct PatternModel {
    /**
     * propagator[pattern1][edge_label] contains all the patterns that can be placed
     * next to pattern1 according to the given edge label.
     * NB: For grid topologies edge label corresponds to the direction.
     */
    propagator: Vec<Vec<Vec<usize>>>,

    /**
     * Stores the desired relative frequencies of each pattern
     */
    frequencies: Vec<f64>,
}

impl PatternModel {
    pub fn new(propagator: Vec<Vec<Vec<usize>>>, frequencies: Vec<f64>) -> Self {
        PatternModel {
            propagator,
            frequencies,
        }
    }

    pub fn propagator(&self) -> &Vec<Vec<Vec<usize>>> {
        &self.propagator
    }

    pub fn propagator_mut(&mut self) -> &mut Vec<Vec<Vec<usize>>> {
        &mut self.propagator
    }

    pub fn set_propagator(&mut self, propagator: Vec<Vec<Vec<usize>>>) {
        self.propagator = propagator;
    }

    pub fn frequencies(&self) -> &Vec<f64> {
        &self.frequencies
    }

    pub fn frequencies_mut(&mut self) -> &mut Vec<f64> {
        &mut self.frequencies
    }

    pub fn set_frequencies(&mut self, frequencies: Vec<f64>) {
        self.frequencies = frequencies;
    }

    pub fn pattern_count(&self) -> usize {
        self.frequencies.len()
    }
}