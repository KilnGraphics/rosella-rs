use std::borrow::BorrowMut;
use std::cmp::{max, min};
use std::ops::{Add, Mul, Sub};

use num_traits::{Num, NumRef};

/// Describes a axis aligned rectangular volume.
///
/// The volume is defined as the region between the points [start] (inclusive) and [end] (exclusive).
/// [start] must always be less than or equal to [end] in all its entries. Some functions may
/// require [start] to be strictly less than [end] to avoid zero volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Region<T: Num + Copy + Clone + Ord, const DIM: usize> {
    start: [T; DIM],
    end: [T; DIM],
}

impl<T: Num + Copy + Clone + Ord, const DIM: usize> Region<T, DIM> where [T; DIM]: Default {
    /// Calculates the volume of the region
    fn volume<R: Num + From<T>>(&self) -> R {
        let mut result = R::one();
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
        let mut result = Self { start: Default::default(), end: Default::default() };
        for i in 0..DIM {
            result.start[i] = max(self.start[i], other.start[i]);
            result.end[i] = min(self.end[i], other.end[i]);
            if result.end[i] <= result.start[i] {
                return None;
            }
        }
        Some(result)
    }

    ///
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

    fn cut_regions<R: Num + From<T>>(&self, regions: &mut Vec<Region<T, DIM>>, intersections: &mut Vec<Region<T, DIM>>) -> R {
        let mut volume = R::zero();
        let mut tool_count = regions.len();
        let mut tail_count: usize = 0;

        let mut i = 0;
        while i < tool_count {
            let mut current = regions[i];
            match current.cut(self, regions) {
                Some(count) => {
                    tail_count += count as usize;
                    regions.swap_remove(i);

                    volume = volume + current.volume();
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

trait TransitionSystem<V: Sync, T: Num + Copy + Clone + Ord, const DIM: usize> {
    fn on_update(&mut self, affected_regions: &Vec<Region<T, DIM>>, value: &mut V, value_region: &Region<T, DIM>);

    fn on_override(&mut self, affected_regions: &Vec<Region<T, DIM>>, value: &mut V, value_region: &Region<T, DIM>);

    fn on_clear(&mut self, affected_regions: &Vec<Region<T, DIM>>, value: &mut V, value_region: &Region<T, DIM>);
}

struct RegionInfo<V: Sync, T: Num + Copy + Clone + Ord, const DIM: usize> {
    next: Option<Box<Self>>,
    region: Region<T, DIM>,
    active_volume: T,
    value: Box<V>,
}

impl<V: Sync, T: Num + Copy + Clone + Ord, const DIM: usize> RegionInfo<V, T, DIM> where [T; DIM]: Default {
    fn new(region: Region<T, DIM>, value: Box<V>) -> Self {
        let active_volume = region.volume();
        Self { next: None, region, active_volume, value }
    }

    fn chain_update<S: TransitionSystem<V, T, DIM>>(&mut self, transition_system: &mut S, regions: &mut Vec<Region<T, DIM>>, intersection_vec: &mut Vec<Region<T, DIM>>) {
        intersection_vec.clear();

        self.region.cut_regions::<T>(regions, intersection_vec);
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

        if self.active_volume == T::zero() {
            Some(self.next.take())
        } else {
            None
        }
    }
}

struct BufferState {
    pending_writes: ash::vk::AccessFlags2KHR,
    pending_stages: ash::vk::PipelineStageFlags2KHR,

}

mod test {
    use super::*;

    #[test]
    fn test_region_volume() {
        let region = Region { start: [0], end: [12] };
        assert_eq!(region.volume::<i32>(), 12);

        let region = Region { start: [-12], end: [5] };
        assert_eq!(region.volume::<i32>(), 17);

        let region = Region { start: [3], end: [3] };
        assert_eq!(region.volume::<i32>(), 0);

        let region = Region { start: [0u32], end: [5u32] };
        assert_eq!(region.volume::<u32>(), 5u32);

        let region = Region { start: [8u32], end: [8u32] };
        assert_eq!(region.volume::<u32>(), 0u32);


        let region = Region { start: [0, 0], end: [12, 12] };
        assert_eq!(region.volume::<i32>(), 144);

        let region = Region { start: [-12, 8], end: [5, 10] };
        assert_eq!(region.volume::<i32>(), 34);

        let region = Region { start: [3, 7], end: [3, 19] };
        assert_eq!(region.volume::<i32>(), 0);

        let region = Region { start: [7, 3], end: [19, 3] };
        assert_eq!(region.volume::<i32>(), 0);

        let region = Region { start: [0u32, 0u32], end: [5u32, 5u32] };
        assert_eq!(region.volume::<u32>(), 25u32);

        let region = Region { start: [8u32, 0u32], end: [8u32, 2u32] };
        assert_eq!(region.volume::<u32>(), 0u32);

        let region = Region { start: [0u32, 8u32], end: [2u32, 8u32] };
        assert_eq!(region.volume::<u32>(), 0u32);
    }

    #[test]
    fn test_region_cut1d() {
        let mut vec = Vec::<Region<i32, 1>> ::new();

        let mut intersection = Region { start: [0], end: [2] };
        let count = intersection.cut(&Region { start: [1], end: [3] }, &mut vec);

        assert_eq!(intersection, Region { start: [1], end: [2] });
        assert_eq!(count, Some(1));
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], Region { start: [0], end: [1] });


        let mut vec = Vec::<Region<i32, 1>> ::new();

        let mut intersection = Region { start: [12], end: [37] };
        let count = intersection.cut(&Region { start: [7], end: [20] }, &mut vec);

        assert_eq!(intersection, Region { start: [12], end: [20] });
        assert_eq!(count, Some(1));
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], Region { start: [20], end: [37] });


        let mut vec = Vec::<Region<i32, 1>> ::new();

        let mut intersection = Region { start: [-23], end: [38] };
        let count = intersection.cut(&Region { start: [5], end: [10] }, &mut vec);

        assert_eq!(intersection, Region { start: [5], end: [10] });
        assert_eq!(count, Some(2));
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], Region { start: [-23], end: [5] });
        assert_eq!(vec[1], Region { start: [10], end: [38] });


        let mut vec = Vec::<Region<i32, 1>> ::new();

        let mut intersection = Region { start: [-11], end: [-1] };
        let count = intersection.cut(&Region { start: [-134], end: [-22] }, &mut vec);

        assert_eq!(count, None);
        assert_eq!(vec.len(), 0);
    }
}