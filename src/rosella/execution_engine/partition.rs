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

        fn transition_split(&mut self, extent: &Extent<T, DIM>, overlap: Extent<T, DIM>, transition_function: TransitionFunction<V, T, DIM>) {
            todo!()
        }

        fn transition(&mut self, extent: &Extent<T, DIM>, transition_function: TransitionFunction<V, T, DIM>) -> Option<EntryChain<V, T, DIM>> {
            let mut remove = false;
            match self.extent.get_overlap(&extent) {
                Some(overlap) => {
                    match transition_function(&overlap, Some(self.value.borrow())) {
                        TransitionAction::Ignore => {
                            self.transition_recurse(extent, transition_function);
                            None
                        },
                        TransitionAction::Update(value) => {
                            self.transition_split(extent, overlap, transition_function);
                            self.value = value;
                            None
                        }
                        TransitionAction::Clear => {
                            self.transition_split(extent, overlap, transition_function);
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
}