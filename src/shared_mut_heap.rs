use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use crate::heap::HeapNode;

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

        let start = Self::parent(self.size); // match C#, was let start = Self::parent(self.size - 1);
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
    pub fn increased_key(&mut self, item_heap_index: usize) {
        if item_heap_index < self.size {
            self.heapify_at(item_heap_index);
        }
    }

    /// Called when an item's key has changed (either increased or decreased)
    pub fn changed_key(&mut self, item_index: usize) {
        self.decreased_key(item_index);
        self.increased_key(item_index);
    }

    /// Inserts a new item into the heap
    pub fn insert(&mut self, item: Rc<RefCell<T>>) {
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
    pub fn delete(&mut self, item_heap_index: usize) {
        if item_heap_index >= self.size {
            return;
        }

        if item_heap_index == self.size - 1 { // i.e. we're removing the last element
            self.size -= 1;
        } else {
            // Move last item to deleted position
            let last_index = self.size - 1;
            self.data.swap(item_heap_index, last_index);
            self.data[item_heap_index].borrow_mut().set_heap_index(Some(item_heap_index));
            self.size -= 1;
            let item_index = self.data[item_heap_index].borrow_mut().index();

            // Restore heap property
            self.increased_key(item_heap_index);
            self.decreased_key(item_index);
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

    #[test]
    fn test_single_item() {
        let mut heap = SharedMutHeap::new();
        let node = Rc::new(RefCell::new(TestNode::new(5, "test".to_string())));
        heap.insert(node);

        assert_eq!(heap.count(), 1);
        assert!(!heap.is_empty());
        assert_eq!(heap.peek().unwrap().borrow_mut().key, 5);
    }

    /// When an element is deleted, the size of the heap is reduced by one, and the deleted element
    /// is moved to the end of the heap.
    #[test]
    fn test_delete() {
        // Arrange
        let nodes = vec![
            Rc::new(RefCell::new(TestNode::new(1, "test".to_string()))),
            Rc::new(RefCell::new(TestNode::new(2, "test".to_string()))),
            Rc::new(RefCell::new(TestNode::new(3, "test".to_string()))),
        ];
        let mut heap = SharedMutHeap::from_vec(nodes);
        let before_delete = heap.count();

        // Act
        heap.delete(1);

        // Assert
        assert_eq!(3, before_delete);
        assert_eq!(2, heap.count());
        assert_eq!(1, heap.data[0].borrow_mut().key);
        assert_eq!(3, heap.data[1].borrow_mut().key);
        assert_eq!(2, heap.data[2].borrow_mut().key);
    }

    #[test]
    fn test_delete_multiple_same_index() {
        // Arrange
        let nodes = vec![
            Rc::new(RefCell::new(TestNode::new(2, "test".to_string()))),
            Rc::new(RefCell::new(TestNode::new(5, "test".to_string()))),
            Rc::new(RefCell::new(TestNode::new(3, "test".to_string()))),
            Rc::new(RefCell::new(TestNode::new(4, "test".to_string()))),
            Rc::new(RefCell::new(TestNode::new(1, "test".to_string()))),
        ];
        let mut heap = SharedMutHeap::from_vec(nodes);
        let before_delete = heap.count();

        // Act
        heap.delete(1);
        heap.delete(1);

        // Assert
        assert_eq!(5, before_delete);
        assert_eq!(3, heap.count());
        assert_eq!(1, heap.data[0].borrow_mut().key);
        assert_eq!(5, heap.data[1].borrow_mut().key);
        assert_eq!(3, heap.data[2].borrow_mut().key);
    }
}