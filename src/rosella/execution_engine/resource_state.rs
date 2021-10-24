use std::borrow::BorrowMut;
use std::cmp::{max, min};
use std::ops::{Add, Mul, Sub};

/// Describes a axis aligned rectangular volume.
///
/// The volume is defined as the region between the points [start] (inclusive) and [end] (exclusive).
/// [start] must always be less than or equal to [end] in all its entries. Some functions may
/// require [start] to be strictly less than [end] to avoid zero volume.
#[derive(Clone, Copy, Debug)]
struct Region<T: Add + Sub + Ord + Copy + Clone + Default + Sync, const DIM: usize> {
    start: [T; DIM],
    end: [T; DIM],
}

impl<T: Add + Sub + Ord + Copy + Clone + Default + Sync, const DIM:usize> Region<T, DIM> where [T; DIM] : Default {
    fn volume<R: Mul<Output = R> + Ord + Copy + Clone + From<<T as Sub>::Output> + From<u8>>(&self) -> R {
        let mut result = R::from(1);
        for i in 0..DIM {
            result = result * R::from(self.end[i] - self.start[i]);
        }
        result
    }

    fn intersects(&self, other: &Self) -> bool {
        for i in 0..DIM {
            if !((self.end[i] > other.start[i]) && (other.end[i] > self.start[i])) {
                return false;
            }
        }
        true
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        let mut result = Self{ start: Default::default(), end: Default::default() };
        for i in 0..DIM {
            result.start[i] = max(self.start[i], other.start[i]);
            result.end[i] = min(self.end[i], other.end[i]);
            if result.end[i] <= result.start[i] {
                return None;
            }
        }
        Some(result)
    }

    fn cut(&mut self, tool: &Self, splits: &mut Vec<Self>) -> Option<u8> {
        let reset_count = splits.len();
        let mut split_count: u8 = 0;
        for a in 0..DIM {
            let mut fail = true;
            if self.start[a] < tool.start[a] && self.end[a] > tool.start[a] {
                let mut cut = *self;
                cut.end[a] = tool.start[a];
                self.start[a] = tool.start[a];

                splits.push(cut);
                split_count += 1;
                fail = false;
            }
            if self.end[a] > tool.end[a] && self.start[a] < tool.end[a] {
                let mut cut = *self;
                cut.start[a] = tool.end[a];
                self.end[a] = tool.end[a];

                splits.push(cut);
                split_count += 1;
                fail = false;
            }
            if fail {
                splits.resize_with(reset_count, || panic!("Should never increase in size"));
                return None;
            }
        }
        Some(split_count)
    }

    fn cut_regions<R: Mul<Output = R> + Ord + Copy + Clone + From<<T as Sub>::Output> + From<u8>>(&self, regions: &mut Vec<Region<T, DIM>>, intersections: &mut Vec<Region<T, DIM>>) -> R {
        let mut volume = R::from(0);
        let mut tool_count = regions.len();
        let mut tail_count: usize = 0;

        let mut i = 0;
        while i < tool_count {
            let mut current = regions[i];
            match current.cut(self, regions) {
                Some(count) => {
                    tail_count += count as usize;
                    regions.swap_remove(i);

                    volume += current.volume();
                    intersections.push(current);

                    // If there are no new entries at the end we swapped a value that needs to be processed as well.
                    if tail_count == 0 {
                        tool_count -= 1;
                    } else {
                        i += 1;
                        tail_count -= 1;
                    }
                }
                None => {
                    i += 1;
                }
            }
        }
        volume
    }
}

trait TransitionSystem<V: Sync, T: Add + Sub + Ord + Copy + Clone + Default + Sync, const DIM:usize> {
    fn on_update(&mut self, affected_regions: &Vec<Region<T, DIM>>, value: &mut V, value_region: &Region<T, DIM>);

    fn on_override(&mut self, affected_regions: &Vec<Region<T, DIM>>, value: &mut V, value_region: &Region<T, DIM>);

    fn on_clear(&mut self, affected_regions: &Vec<Region<T, DIM>>, value: &mut V, value_region: &Region<T, DIM>);
}

struct RegionInfo<V: Sync, T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Ord + Copy + Clone + Default + Sync + From<u8>, const DIM: usize> {
    next: Option<Box<Self>>,
    region: Region<T, DIM>,
    active_volume: T,
    value: Box<V>,
}

impl<V: Sync, T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Ord + Copy + Clone + Default + Sync + From<u8>, const DIM: usize> RegionInfo<V, T, DIM> where [T; DIM]: Default {
    fn new(region: Region<T, DIM>, value: Box<V>) -> Self {
        let active_volume = region.volume();
        Self { next: None, region, active_volume, value }
    }

    fn chain_update<S: TransitionSystem<V, T, DIM>>(&mut self, transition_system: &mut S, regions: &mut Vec<Region<T, DIM>>, intersection_vec: &mut Vec<Region<T, DIM>>) {
        intersection_vec.clear();

        self.region.cut_regions(regions, intersection_vec);
        if !intersection_vec.is_empty() {
            transition_system.on_update(intersection_vec, self.value.borrow_mut(), &self.region);
        }

        if let Some(ref mut next) = self.next {
            next.chain_update(transition_system, regions, intersection_vec);
        }
    }

    fn chain_override<S: TransitionSystem<V, T, DIM>>(&mut self, transition_system: &mut S, regions: &mut Vec<Region<T, DIM>>, intersection_vec: &mut Vec<Region<T, DIM>>) -> Option<Option<Box<Self>>> {
        intersection_vec.clear();

        self.active_volume = self.active_volume - self.region.cut_regions(regions, intersection_vec);
        if !intersection_vec.is_empty() {
            transition_system.on_override(intersection_vec, self.value.borrow_mut(), &self.region);
        }

        if let Some(ref mut next) = self.next {
            if let Some(new_next) = next.chain_override(transition_system, regions, intersection_vec) {
                 self.next = new_next;
            }
        }

        if self.active_volume == T::from(0) {
            Some(self.next.take())
        } else {
            None
        }
    }
}

mod test {

}