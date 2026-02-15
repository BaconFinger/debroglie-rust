use std::cell::RefCell;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::procedural_generation::debroglie::heap::HeapNode;

/// Implements a basic min-key heap.
/// Items are kept in an RcRefCell so they can be mutated by both this class and the parent.
#[allow(unused)]
pub struct SharedMutHeap<T, TKey>
where
    T: HeapNode<TKey>,
    TKey: Ord + Clone,
{
    data: Vec<Rc<RefCell<T>>>,
    size: usize,
    _phantom: PhantomData<TKey>,
}

impl<T, TKey> SharedMutHeap<T, TKey>
where
    T: HeapNode<TKey>,
    TKey: Ord + Clone,
{
    fn parent(i: usize) -> usize {
        (i.saturating_sub(1)) >> 1
    }

    fn left(i: usize) -> usize {
        (i << 1) + 1
    }

    fn right(i: usize) -> usize {
        (i << 1) + 2
    }

    /// Creates a new empty heap
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            size: 0,
            _phantom: PhantomData,
        }
    }

    /// Creates a new heap with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            size: 0,
            _phantom: PhantomData,
        }
    }

    /// Creates a heap from a vector of items
    pub fn from_vec(mut items: Vec<Rc<RefCell<T>>>) -> Self {
        let size = items.len();

        // Set heap indices
        for (i, item) in items.iter_mut().enumerate() {
            item.borrow_mut().set_heap_index(Some(i));
        }

        let mut heap = Self {
            data: items,
            size,
            _phantom: PhantomData,
        };

        heap.heapify();
        heap
    }

    /// Returns the number of items in the heap
    pub fn count(&self) -> usize {
        self.size
    }

    /// Returns true if the heap is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns a reference to the minimum item without removing it
    pub fn peek(&self) -> Option<Rc<RefCell<T>>> {
        if self.size == 0 {
            None
        } else {
            Some(self.data[0].clone())
        }
    }

    /// Restores the heap property for the entire heap
    pub fn heapify(&mut self) {
        if self.size == 0 {
            return;
        }

        // let start = Self::parent(self.size - 1);
        // for i in (0..=start).rev() {
        //     self.heapify_at(i);
        // }
        let start = Self::parent(self.size); // match C#
        for i in (0..=start).rev() {
            self.heapify_at(i);
        }
    }

    /// Restores the heap property starting at the given index
    fn heapify_at(&mut self, i: usize) {
        let mut smallest = i;
        let l = Self::left(i);
        let r = Self::right(i);

        // Find the smallest among i, left child, and right child
        if l < self.size && self.data[l].borrow_mut().key() < self.data[smallest].borrow_mut().key() {
            smallest = l;
        }

        if r < self.size && self.data[r].borrow_mut().key() < self.data[smallest].borrow_mut().key() {
            smallest = r;
        }

        if i == smallest {
            self.data[i].borrow_mut().set_heap_index(Some(i));
        } else {
            // Swap and recursively heapify
            self.data.swap(i, smallest);
            self.data[i].borrow_mut().set_heap_index(Some(i));
            self.heapify_at(smallest);
        }
    }

    /// Restores the heap property starting at the given index. Returns the new heap_index of the item we sent in.
    fn heapify_at_with_new_index(&mut self, heap_index: usize, index: usize) -> usize {
        let mut smallest = heap_index;
        let l = Self::left(heap_index);
        let r = Self::right(heap_index);

        // Find the smallest among i, left child, and right child
        if l < self.size && self.data[l].borrow_mut().key() < self.data[smallest].borrow_mut().key() {
            smallest = l;
        }

        if r < self.size && self.data[r].borrow_mut().key() < self.data[smallest].borrow_mut().key() {
            smallest = r;
        }

        if heap_index == smallest {
            self.data[heap_index].borrow_mut().set_heap_index(Some(heap_index));
        } else {
            // Swap and recursively heapify
            self.data.swap(heap_index, smallest);
            self.data[heap_index].borrow_mut().set_heap_index(Some(heap_index));
            self.heapify_at(smallest);
        }

        // Now get the final, updated index of the item we sent in
        for element in self.data.iter() {
            if element.borrow_mut().index() == index {
                if element.borrow_mut().heap_index().is_none() {
                    // If this happens, need to investigate how. This shouldn't happen.
                    panic!("Item with index {} has no heap index", index);
                }
                return element.borrow_mut().heap_index().unwrap().clone();
            }
        }
        heap_index
    }

    /// Called when an item's key has decreased
    pub fn decreased_key(&mut self, item_index: usize) {
        if item_index >= self.size {
            return;
        }

        let mut i = item_index;

        loop {
            if i == 0 {
                self.data[i].borrow_mut().set_heap_index(Some(i));
                return;
            }

            let p = Self::parent(i);

            // Compare keys
            if self.data[p].borrow_mut().key() > self.data[i].borrow_mut().key() {
                self.data.swap(p, i);
                self.data[i].borrow_mut().set_heap_index(Some(i));
                i = p;
            } else {
                self.data[i].borrow_mut().set_heap_index(Some(i));
                return;
            }
        }
    }

    /// Called when an item's key has increased, returns the new heap_index.
    pub fn increased_key(&mut self, item_heap_index: usize, item_index: Option<usize>) -> usize {
        if item_heap_index < self.size && item_index.is_some() { // TODO: Maybe this is a problem?
            return self.heapify_at_with_new_index(item_heap_index, item_index.unwrap()); // Safe because we checked that item_index is Some
        }
        self.heapify_at(item_heap_index);
        item_heap_index
    }

    /// Called when an item's key has changed (either increased or decreased)
    pub fn changed_key(&mut self, item_index: usize) {
        self.decreased_key(item_index);
        self.increased_key(item_index, None);
    }

    /// Inserts a new item into the heap
    pub fn insert(&mut self, mut item: Rc<RefCell<T>>) {
        // Ensure capacity
        if self.data.len() == self.size {
            self.data.reserve(std::cmp::max(1, self.size));
        }

        // Add item at the end
        if self.size < self.data.len() {
            item.borrow_mut().set_heap_index(Some(self.size));
            self.data[self.size] = item;
        } else {
            item.borrow_mut().set_heap_index(Some(self.size));
            self.data.push(item);
        }

        self.size += 1;
        self.decreased_key(self.size - 1);
    }
    
    /// Removes an item from the heap by its index
    // pub unsafe fn delete_with_pop(&mut self, item_heap_index: usize) -> Option<Rc<RefCell<T>>> {
    //     if item_heap_index >= self.size {
    //         return None;
    //     }
    // 
    //     if item_heap_index == self.size - 1 {
    //         // Remove last item
    //         self.size -= 1;
    //         if item_heap_index < self.data.len() {
    //             Some(std::mem::replace(&mut self.data[item_heap_index], unsafe { std::mem::zeroed() }))
    //         } else {
    //             self.data.pop()
    //         }
    //     } else {
    //         // Move last item to deleted position
    //         let last_index = self.size - 1;
    //         self.data.swap(item_heap_index, last_index);
    //         self.data[item_heap_index].borrow_mut().set_heap_index(Some(item_heap_index));
    //         self.size -= 1;
    //         let item_index = self.data[item_heap_index].borrow_mut().index();
    // 
    //         // Restore heap property
    //         let new_heap_index = self.increased_key(item_heap_index, Some(item_index));
    //         self.decreased_key(new_heap_index);
    // 
    //         // Return the deleted item (now at the end)
    //         if last_index < self.data.len() {
    //             Some(std::mem::replace(&mut self.data[last_index], unsafe { std::mem::zeroed() }))
    //         } else {
    //             self.data.pop()
    //         }
    //     }
    // }
    
    /// Removes an item from the heap by its index
    pub fn delete(&mut self, item_heap_index: usize) {
        if item_heap_index >= self.size {
            return;
        }

        if item_heap_index == self.size - 1 {
            self.size -= 1;
        } else {
            // Move last item to deleted position
            let last_index = self.size - 1;
            self.data.swap(item_heap_index, last_index);
            self.data[item_heap_index].borrow_mut().set_heap_index(Some(item_heap_index));
            self.size -= 1;
            let item_index = self.data[item_heap_index].borrow_mut().index();

            // Restore heap property
            let new_heap_index = self.increased_key(item_heap_index, Some(item_index));
            self.decreased_key(new_heap_index);
        }
    }

    /// Removes and returns the minimum item
    pub fn extract_min(&mut self) {
        if self.size == 0 {
            return;
        } else {
            self.delete(0)
        }
    }

    /// Clears all items from the heap
    pub fn clear(&mut self) {
        self.size = 0;
        self.data.clear();
    }
}

impl<T, TKey> Default for SharedMutHeap<T, TKey>
where
    T: HeapNode<TKey>,
    TKey: Ord + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestNode {
        key: i32,
        heap_index: usize,
        value: String,
        index: usize,
    }

    impl TestNode {
        fn new(key: i32, value: String) -> Self {
            Self {
                key,
                heap_index: 0,
                value,
                index: 0,
            }
        }
    }

    impl HeapNode<i32> for TestNode {
        fn heap_index(&self) -> Option<usize> {
            Some(self.heap_index)
        }

        fn set_heap_index(&mut self, index: Option<usize>) {
            self.heap_index = index.unwrap_or(0);
        }

        fn key(&self) -> i32 {
            self.key
        }
        fn index(&self) -> usize { self.index }
    }

    #[test]
    fn test_empty_heap() {
        let heap: SharedMutHeap<TestNode, i32> = SharedMutHeap::new();
        assert_eq!(heap.count(), 0);
        assert!(heap.is_empty());
        assert!(heap.peek().is_none());
    }
    //
    // #[test]
    // fn test_single_item() {
    //     let mut heap = Heap::new();
    //     let node = TestNode::new(5, "test".to_string());
    //     heap.insert(node);
    //
    //     assert_eq!(heap.count(), 1);
    //     assert!(!heap.is_empty());
    //     assert_eq!(heap.peek().unwrap().key, 5);
    // }

    // #[test]
    // fn test_min_heap_property() {
    //     let mut heap = Heap::new();
    //     heap.insert(TestNode::new(10, "ten".to_string()));
    //     heap.insert(TestNode::new(5, "five".to_string()));
    //     heap.insert(TestNode::new(15, "fifteen".to_string()));
    //     heap.insert(TestNode::new(3, "three".to_string()));
    //     heap.insert(TestNode::new(8, "eight".to_string()));
    //
    //     assert_eq!(heap.peek().unwrap().key, 3);
    //
    //     let min = heap.extract_min().unwrap();
    //     assert_eq!(min.key, 3);
    //     assert_eq!(heap.peek().unwrap().key, 5);
    // }

    // #[test]
    // fn test_from_vec() {
    //     let nodes = vec![
    //         TestNode::new(10, "ten".to_string()),
    //         TestNode::new(5, "five".to_string()),
    //         TestNode::new(15, "fifteen".to_string()),
    //         TestNode::new(3, "three".to_string()),
    //     ];
    //
    //     let heap = Heap::from_vec(nodes);
    //     assert_eq!(heap.count(), 4);
    //     assert_eq!(heap.peek().unwrap().key, 3);
    // }

    // #[test]
    // fn test_extract_all() {
    //     let mut heap = Heap::new();
    //     let values = vec![10, 5, 15, 3, 8, 12, 1];
    //
    //     for val in values {
    //         heap.insert(TestNode::new(val, format!("val_{}", val)));
    //     }
    //
    //     let mut extracted = Vec::new();
    //     while let Some(node) = heap.extract_min() {
    //         extracted.push(node.key);
    //     }
    //
    //     assert_eq!(extracted, vec![1, 3, 5, 8, 10, 12, 15]);
    // }
}