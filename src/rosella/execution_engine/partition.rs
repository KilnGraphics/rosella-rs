mod partition {
    use std::borrow::Borrow;
    use std::ops::{Add, Sub};
    use std::cmp::{max, min};
    use std::sync::Arc;

    /// Describes a axis aligned rectangular volume.
    ///
    /// `start` must always be less than or equal to `end` in all its entries. Some functions may
    /// require `start` to be strictly less than `end` to avoid zero volume.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Extent<T: Add + Sub + Ord + Copy + Default, const DIM: usize> {
        pub start: [T; DIM],
        pub end: [T; DIM],
    }

    impl<T: Add + Sub + Ord + Copy + Default, const DIM: usize> Extent<T, DIM> where [T; DIM]: Default {
        fn get_overlap(&self, other: &Extent<T, DIM>) -> Option<Extent<T, DIM>> {
            let mut res_start: [T; DIM] = Default::default();
            let mut res_end: [T; DIM] = Default::default();
            for i in 0..DIM {
                res_start[i] = max(self.start[i], other.start[i]);
                res_end[i] = min(self.end[i], other.end[i]);
                if res_start[i] >= res_end[i] {
                    return None;
                }
            }
            Some(Extent { start: res_start, end: res_end })
        }
    }

    type EntryChain<V, T: Add + Sub + Ord + Copy + Default, const DIM: usize> = Option<Box<Entry<V, T, DIM>>>;

    pub struct Partition<V, T: Add + Sub + Ord + Copy + Default, const DIM: usize> {
        first: EntryChain<V, T, DIM>,
    }

    pub enum TransitionAction<V> {
        Ignore,
        Update(Arc<V>),
        Clear,
    }

    pub type TransitionFunction<V, T: Add + Sub + Ord + Copy + Default, const DIM: usize> = fn(&Extent<T, DIM>, Option<&V>) -> TransitionAction<V>;

    impl<V, T: Add + Sub + Ord + Copy + Default, const DIM: usize> Partition<V, T, DIM> where [T; DIM]:Default {
        pub fn new() {
            Partition::<V, T, DIM>{ first: None };
        }

        pub fn transition(&mut self, extent: &Extent<T, DIM>, transition_function: TransitionFunction<V, T, DIM>) {
            match &mut self.first {
                None =>
                    match transition_function(extent, None) {
                        TransitionAction::Update(value) => self.first = Some(Box::new(Entry::new(*extent, value))),
                        _ => {}
                    },
                Some(ref mut next) =>
                    match next.transition(extent, transition_function) {
                        Some(next) => self.first = next,
                        None => {}
                    }
            }
        }

        pub fn iter(&self) -> PartitionIterator<V, T, DIM> {
            PartitionIterator::new(&self.first)
        }
    }

    pub struct PartitionIterator<'a, V, T: Add + Sub + Ord + Copy + Default, const DIM: usize> {
        current: Option<&'a Entry<V, T, DIM>>,
    }

    impl<'a, V, T: Add + Sub + Ord + Copy + Default, const DIM: usize> PartitionIterator<'a, V, T, DIM> where [T; DIM]:Default {
        fn new(first: &'a EntryChain<V, T, DIM>) -> PartitionIterator<'a, V, T, DIM> {
            match first {
                None => PartitionIterator{ current: None },
                Some(current) => PartitionIterator{ current: Some(current.as_ref()) }
            }
        }
    }

    impl<'a, V, T: Add + Sub + Ord + Copy + Default, const DIM: usize> Iterator for PartitionIterator<'a, V, T, DIM> where [T; DIM]:Default {
        type Item = (&'a Extent<T, DIM>, &'a Arc<V>);

        fn next(&mut self) -> Option<Self::Item> {
            match self.current {
                None => None,
                Some(current) => match current.next {
                    None => {
                        self.current = None;
                        None
                    },
                    Some(ref next) => {
                        let current = next.as_ref();
                        self.current = Some(current);
                        Some((&current.extent, &current.value))
                    }
                }
            }
        }
    }

    struct Entry<V, T: Add + Sub + Ord + Copy + Default, const DIM: usize> {
        next: EntryChain<V, T, DIM>,
        extent: Extent<T, DIM>,
        value: Arc<V>,
    }

    impl<V, T: Add + Sub + Ord + Copy + Default, const DIM: usize> Entry<V, T, DIM> where [T; DIM]:Default {
        fn new(extent: Extent<T, DIM>, value: Arc<V>) -> Self {
            Entry { next: None, extent, value }
        }

        fn transition_recurse(&mut self, extent: &Extent<T, DIM>, transition_function: TransitionFunction<V, T, DIM>) {
            match &mut self.next {
                Some(ref mut next) =>
                    match next.transition(extent, transition_function) {
                        Some(new_next) => self.next = new_next,
                        None => {}
                    },
                None => {}
            }
        }

        fn transition_split(&mut self, extent: &Extent<T, DIM>, transition_function: TransitionFunction<V, T, DIM>) {
            // Put new entries into a temporary list to prevent transition_recurse from iterating through them
            let mut new_chain: Vec<EntryChain<V, T, DIM>> = Vec::new();
            for i in 0..DIM {
                if extent.start[i] < self.extent.start[i] {
                    let mut new = *extent;
                    new.end[i] = self.extent.start[i];
                    self.transition_recurse(&new, transition_function);
                }
                if extent.end[i] > self.extent.end[i] {
                    let mut new = *extent;
                    new.start[i] = self.extent.end[i];
                    self.transition_recurse(&new, transition_function);
                }

                if self.extent.start[i] < extent.start[i] {
                    let mut new = self.extent;
                    new.end[i] = extent.start[i];
                    new_chain.push(Some(Box::new(Entry::new(new, Arc::clone(&self.value) ))));
                    self.extent.start[i] = extent.start[i];
                }
                if self.extent.end[i] > extent.end[i] {
                    let mut new = self.extent;
                    new.start[i] = extent.end[i];
                    new_chain.push(Some(Box::new(Entry::new(new, Arc::clone(&self.value) ))));
                    self.extent.end[i] = extent.end[i];
                }
            }

            // Add the entries from the temporary list to the main list
            let mut last = self.next.take();
            for mut entry in new_chain {
                match entry {
                    None => panic!(),
                    Some(mut entry) => {
                        entry.next = last;
                        last = Some(entry);
                    }
                }
            }
            self.next = last;
        }

        fn transition(&mut self, extent: &Extent<T, DIM>, transition_function: TransitionFunction<V, T, DIM>) -> Option<EntryChain<V, T, DIM>> {
            match self.extent.get_overlap(&extent) {
                Some(overlap) => {
                    match transition_function(&overlap, Some(self.value.borrow())) {
                        TransitionAction::Ignore => {
                            self.transition_recurse(extent, transition_function);
                            None
                        },
                        TransitionAction::Update(value) => {
                            self.transition_split(extent, transition_function);
                            self.value = value;
                            None
                        }
                        TransitionAction::Clear => {
                            self.transition_split(extent, transition_function);
                            Some(self.next.take())
                        }
                    }
                }
                None => {
                    self.transition_recurse(extent, transition_function);
                    None
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_empty() {

        }
    }
}