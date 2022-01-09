/// Enables mutable access to one element of a slice while still providing immutable access
/// to other elements.
pub struct Splitter<'a, T> {
    index: usize,
    head: &'a [T],
    tail: &'a [T],
}

impl<'a, T> Splitter<'a, T> {
    /// Creates a new Splitter providing mutable access to the element at the specified index.
    pub fn new<'b: 'a>(slice: &'b mut [T], index: usize) -> (Self, &'a mut T) {
        let (head, tail) = slice.split_at_mut(index);
        let (elem, tail) = tail.split_first_mut().unwrap();

        (Self {
            index,
            head,
            tail
        }, elem)
    }

    /// Returns a reference to any element that is not mutably accessed.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.index {
            self.head.get(index)
        } else if index > self.index {
            self.tail.get(index - self.index - 1)
        } else {
            None
        }
    }
}