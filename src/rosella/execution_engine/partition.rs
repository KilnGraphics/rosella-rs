mod partition {

    /// Describes a axis aligned rectangular volume.
    ///
    /// `start` must always be less than or equal to `end` in all its entries. Some functions may
    /// require `start` to be strictly less than `end` to avoid zero volume.
    #[derive(Sync)]
    pub struct Extent<T:Add+Sub+Ord+Copy, const DIM:usize> {
        pub start: [T; DIM],
        pub end: [T; DIM],
    }

    impl<T:Add+Sub+Ord+Copy, const DIM:usize> Extent<T, DIM> {

    }
}