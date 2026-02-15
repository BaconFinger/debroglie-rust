use std::mem;

#[derive(Debug)]
pub struct Deque<T> {
    data: Vec<T>,
    data_length: usize,

    // Data is in range lo to hi, exclusive of hi
    // hi == lo if the Deque is empty
    // You may have hi < lo if we've wrapped the end of data
    lo: usize,
    hi: usize,
}

impl<T> Deque<T> {
    pub fn new() -> Self {
        Self::with_capacity(4)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut data = Vec::with_capacity(capacity);
        // Fill with default values - we need to handle this carefully in Rust
        // For now, we'll use MaybeUninit pattern or require Default trait
        // unsafe {
        //     data.set_len(capacity);
        // }

        Deque {
            data,
            data_length: capacity,
            lo: 0,
            hi: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, t: T) {
        let hi = self.hi;
        let lo = self.lo;

        // Store the value (this overwrites any existing value)
        if hi < self.data.len() {
            self.data[hi] = t;
        } else {
            // This shouldn't happen if our logic is correct
            panic!("Index out of bounds");
        }

        let mut new_hi = hi + 1;
        if new_hi == self.data_length {
            new_hi = 0;
        }
        self.hi = new_hi;

        if new_hi == lo {
            self.resize_from_full();
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        let lo = self.lo;
        let mut hi = self.hi;

        if lo == hi {
            return None;
        }

        if hi == 0 {
            hi = self.data_length;
        }
        hi -= 1;
        self.hi = hi;

        // We need to move the value out
        // This is tricky in Rust - we'll use unsafe or require Clone/Copy
        unsafe {
            Some(std::ptr::read(&self.data[hi]))
        }
    }

    #[inline]
    pub fn shift(&mut self, t: T) {
        let mut lo = self.lo;
        let hi = self.hi;

        if lo == 0 {
            lo = self.data_length;
        }
        lo -= 1;

        self.data[lo] = t;
        self.lo = lo;

        if hi == lo {
            self.resize_from_full();
        }
    }

    #[inline]
    pub fn unshift(&mut self) -> T {
        let mut lo = self.lo;
        let hi = self.hi;

        if lo == hi {
            panic!("Deque is empty");
        }

        let old_lo = lo;
        lo += 1;
        if lo == self.data_length {
            lo = 0;
        }
        self.lo = lo;

        unsafe {
            std::ptr::read(&self.data[old_lo])
        }
    }

    pub fn drop_first(&mut self, n: usize) {
        let hi = self.hi;
        let mut lo = self.lo;

        if lo <= hi {
            lo += n;
            if lo >= hi {
                // Empty
                self.lo = 0;
                self.hi = 0;
            } else {
                self.lo = lo;
            }
        } else {
            lo += n;
            if lo >= self.data_length {
                lo -= self.data_length;
                if lo >= hi {
                    // Empty
                    self.lo = 0;
                    self.hi = 0;
                } else {
                    self.lo = lo;
                }
            } else {
                self.lo = lo;
            }
        }
    }

    pub fn drop_last(&mut self, n: usize) {
        let mut hi = self.hi;
        let lo = self.lo;

        if lo <= hi {
            if n > hi {
                hi = 0;
            } else {
                hi -= n;
            }

            if lo >= hi {
                // Empty
                self.lo = 0;
                self.hi = 0;
            } else {
                self.hi = hi;
            }
        } else {
            if n > hi {
                hi = hi + self.data_length - n;
                if lo >= hi {
                    // Empty
                    self.lo = 0;
                    self.hi = 0;
                } else {
                    self.hi = hi;
                }
            } else {
                hi -= n;
                self.hi = hi;
            }
        }
    }

    pub fn count(&self) -> usize {
        let c = if self.hi >= self.lo {
            self.hi - self.lo
        } else {
            self.hi + self.data_length - self.lo
        };
        c
    }

    pub fn is_empty(&self) -> bool {
        self.lo == self.hi
    }

    fn resize_from_full(&mut self) {
        let data_length = self.data_length;
        let new_length = data_length * 2;
        let mut new_data = Vec::with_capacity(new_length);

        unsafe {
            new_data.set_len(new_length);
        }

        let mut i = self.lo;
        let mut j = 0;
        let hi = self.hi;

        loop {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &self.data[i] as *const T,
                    &mut new_data[j] as *mut T,
                    1
                );
            }

            j += 1;
            i += 1;
            if i == data_length {
                i = 0;
            }

            if i == hi {
                break;
            }
        }

        self.data = new_data;
        self.data_length = new_length;
        self.lo = 0;
        self.hi = j;
    }

    pub fn slice(&self, start: usize, end: usize) -> DequeSliceIter<T> {
        if start >= end {
            panic!("Invalid slice range");
        }

        let lo = self.lo;
        let hi = self.hi;
        let mut i = lo + start;
        let mut e = lo + end;

        if hi >= lo {
            if e > hi {
                panic!("Slice end out of bounds");
            }
        } else {
            if e > hi + self.data_length {
                panic!("Slice end out of bounds");
            }
        }

        if i >= self.data_length {
            i -= self.data_length;
        }
        if e >= self.data_length {
            e -= self.data_length;
        }

        DequeSliceIter {
            deque: self,
            current: i,
            end: e,
            finished: start >= end,
        }
    }

    pub fn reverse_slice(&self, start: usize, end: usize) -> DequeReverseSliceIter<T> {
        if start >= end {
            panic!("Invalid slice range");
        }

        let lo = self.lo;
        let hi = self.hi;
        let mut i = lo + start;
        let mut e = lo + end;

        if hi >= lo {
            if e > hi {
                panic!("Slice end out of bounds");
            }
        } else {
            if e > hi + self.data_length {
                panic!("Slice end out of bounds");
            }
        }

        if i >= self.data_length {
            i -= self.data_length;
        }
        if e >= self.data_length {
            e -= self.data_length;
        }

        DequeReverseSliceIter {
            deque: self,
            start: i,
            current: e,
            finished: start >= end,
        }
    }
}

// Iterator for the deque
pub struct DequeIter<'a, T> {
    deque: &'a Deque<T>,
    current: usize,
    end: usize,
    finished: bool,
}

impl<'a, T> Iterator for DequeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.current == self.end {
            return None;
        }

        let item = &self.deque.data[self.current];
        self.current += 1;
        if self.current == self.deque.data_length {
            self.current = 0;
        }

        if self.current == self.end {
            self.finished = true;
        }

        Some(item)
    }
}

// Iterator for slice
pub struct DequeSliceIter<'a, T> {
    deque: &'a Deque<T>,
    current: usize,
    end: usize,
    finished: bool,
}

impl<'a, T> Iterator for DequeSliceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.current == self.end {
            return None;
        }

        let item = &self.deque.data[self.current];
        self.current += 1;
        if self.current == self.deque.data_length {
            self.current = 0;
        }

        Some(item)
    }
}

// Iterator for reverse slice
pub struct DequeReverseSliceIter<'a, T> {
    deque: &'a Deque<T>,
    start: usize,
    current: usize,
    finished: bool,
}

impl<'a, T> Iterator for DequeReverseSliceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.current == self.start {
            return None;
        }

        if self.current == 0 {
            self.current = self.deque.data_length - 1;
        } else {
            self.current -= 1;
        }

        let item = &self.deque.data[self.current];
        Some(item)
    }
}

impl<'a, T> IntoIterator for &'a Deque<T> {
    type Item = &'a T;
    type IntoIter = DequeIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        DequeIter {
            deque: self,
            current: self.lo,
            end: self.hi,
            finished: self.lo == self.hi,
        }
    }
}

impl<T> Default for Deque<T> {
    fn default() -> Self {
        Self::new()
    }
}