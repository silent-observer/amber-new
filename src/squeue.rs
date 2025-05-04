struct SQueue<T> {
    data: Box<[T]>,
    start: usize,
    end: usize,
    mask: usize,
}

impl<T> SQueue<T>
where
    T: Ord + Default + Clone + Copy,
{
    pub fn new(size_exp: usize) -> Self {
        let size = 1 << size_exp;
        let mask = size - 1;
        Self {
            data: vec![T::default(); size].into_boxed_slice(),
            start: 0,
            end: 0,
            mask,
        }
    }

    fn at(&self, i: usize) -> &T {
        let index = (self.start + i) as usize;
        &self.data[index & self.mask]
    }

    pub fn len(&self) -> usize {
        let len = self.end.wrapping_sub(self.start);
        return len & self.mask;
    }

    fn find_place_for(&self, item: &T) -> usize {
        let len = self.len();
        let mut l = 0;
        let mut h = len - 1;
        while l < h {
            let mid = (l + h) / 2;
            let x = self.at(mid);
            if *item < *x {
                h = mid - 1;
            } else if *item > *x {
                l = mid + 1;
            } else {
                return mid;
            }
        }
        l
    }

    fn insert(&mut self, i: usize, item: T) {
        let len = self.len();
        if self.end > self.start {
            for j in (i..len).rev() {
                let j1_raw = self.start + j;
                let j2_raw = j1_raw + 1;
                let j1 = j1_raw & self.mask;
                let j2 = j2_raw & self.mask;
                self.data[j2] = self.data[j1];
            }
            self.end += 1;
        } else if self.end < self.start {
            let max_size = self.mask + 1;
            // At end
            if i > (max_size - self.start) {
                for j in (i..len).rev() {
                    let j1_raw = self.start + j;
                    let j2_raw = j1_raw + 1;
                    let j1 = j1_raw & self.mask;
                    let j2 = j2_raw & self.mask;
                    self.data[j2] = self.data[j1];
                }
                self.end += 1;
            } else {
                for j in 0..=i {
                    let j1_raw = self.start + j;
                    let j2_raw = j1_raw - 1;
                    let j1 = j1_raw & self.mask;
                    let j2 = j2_raw & self.mask;
                    self.data[j2] = self.data[j1];
                }
                self.start -= 1;
            }
        }

        let index = (self.start + i) & self.mask;
        self.data[index] = item;
    }

    pub fn push(&mut self, item: T) {
        assert_ne!(self.len(), self.mask);

        let new_index = self.find_place_for(&item);
        self.insert(new_index, item);
    }

    pub fn pop(&mut self) -> T {
        assert_ne!(self.len(), 0);

        let item = self.data[self.start];
        self.start += 1;
        self.start &= self.mask;

        item
    }
}
