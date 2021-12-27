pub struct Splitter<'a, T> {
    index: usize,
    head: &'a [T],
    tail: &'a [T],
}

impl<'a, T> Splitter<'a, T> {
    pub fn new<'b: 'a>(slice: &'b mut [T], index: usize) -> (Self, &'a mut T) {
        let (head, tail) = slice.split_at_mut(index);
        let (elem, tail) = tail.split_first_mut().unwrap();

        (Self {
            index,
            head,
            tail
        }, elem)
    }

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