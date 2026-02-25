use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::context::{Context, TrackerId, WaveId};
use crate::topology::topology::Topology;
use crate::trackers::index_picker::IndexPicker;
use crate::trackers::pattern_picker::PatternPicker;
use crate::trackers::random_picker_utils::RandomPickerUtils;
use crate::trackers::tracker::{SuperTracker, Tracker};
use crate::wfc::wave_propagator::WavePropagator;

#[derive(Default)]
pub struct WeightedRandomPatternPicker<T: Topology + Clone> {
    wave: Option<WaveId>,
    frequencies: Option<Vec<f64>>,
    self_ref: Option<TrackerId>,
    phantom_data: PhantomData<T>
}

impl<T: Topology + Clone> WeightedRandomPatternPicker<T> {
    pub fn new() -> Self {
        Self {
            wave: None,
            frequencies: None,
            self_ref: None,
            phantom_data: PhantomData
        }
    }
}

impl<T: Topology + Clone> PatternPicker<T> for WeightedRandomPatternPicker<T> {
    fn init(&mut self, wave_propagator: &WavePropagator<T>) -> Result<(), String> {
        self.wave = wave_propagator.get_wave_id();
        self.frequencies = Some(wave_propagator.get_frequencies());
        Ok(())
    }

    fn get_random_possible_pattern_at(&mut self, ctx: &Context<T>, index: usize, random_double: Rc<dyn Fn() -> f64>) -> Option<usize> {
        if self.wave.is_none() {
            println!("No wave set for WeightedRandomPatternPicker");
            return None;
        }

        let frequencies = self.frequencies.as_ref()?;
        let pattern = RandomPickerUtils::get_random_possible_pattern(
            ctx,
            self.wave?,
            random_double,
            index,
            frequencies,
        )?;

        if pattern < 0 {
            None
        } else {
            Some(pattern)
        }
    }

    // fn set_self_ref(&mut self, self_ref: TrackerId) {
    //     self.self_ref = Some(self_ref);
    // }
    //
    // fn add_self(self, ctx: &Context<T>) -> TrackerId {
    //     let id = ctx.pattern_pickers().add(Rc::new(RefCell::new(self)));
    //     let me = ctx.pattern_pickers().get(id).unwrap();
    //     me.borrow_mut().set_self_ref(id);
    //
    //     id
    // }
}

impl<T: Topology + Clone> Tracker for WeightedRandomPatternPicker<T> {
    fn reset(&mut self) -> Result<(), String> {
        unimplemented!("WeightedRandomPatternPicker is not a Tracker")
    }

    fn do_ban(&mut self, index: usize, pattern: usize) {
        unimplemented!("WeightedRandomPatternPicker is not a Tracker")
    }

    fn undo_ban(&mut self, index: usize, pattern: usize) {
        unimplemented!("WeightedRandomPatternPicker is not a Tracker")
    }
}

impl<T: Topology + Clone> IndexPicker<T> for WeightedRandomPatternPicker<T> {
    fn init(&mut self, ctx: &Context<T>, wave_propagator: &mut WavePropagator<T>) -> Result<(), String> {
        unimplemented!("WeightedRandomPatternPicker is not an IndexPicker")
    }

    fn get_random_index(&mut self, ctx: &Context<T>, random_double: Rc<dyn Fn() -> f64>) -> Option<i32> {
        unimplemented!("WeightedRandomPatternPicker is not an IndexPicker")
    }

    // fn set_self_ref(&mut self, self_ref: TrackerId) {
    //     unimplemented!("WeightedRandomPatternPicker is not an IndexPicker")
    // }
    //
    // fn add_self(self, ctx: &Context<T>) -> TrackerId {
    //     unimplemented!("WeightedRandomPatternPicker is not an IndexPicker")
    // }
}

impl<T: Topology + Clone> SuperTracker<T> for WeightedRandomPatternPicker<T> {
    fn set_self_ref(&mut self, self_ref: TrackerId) {
        self.self_ref = Some(self_ref);
    }

    fn add_self(self, ctx: &Context<T>) -> TrackerId {
        let id = ctx.pattern_pickers().add(Rc::new(RefCell::new(self)));
        let me = ctx.pattern_pickers().get(id).unwrap(); // Safe because we just added it
        me.borrow_mut().set_self_ref(id);

        id
    }

    fn is_index_picker(&self) -> bool {
        false
    }

    fn is_pattern_picker(&self) -> bool {
        true
    }

    fn is_tracker(&self) -> bool {
        false
    }
}