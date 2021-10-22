use crate::rosella::execution_engine::partition::Partition;

/// Describes the state of a buffer range after a execution.
///
/// The range is defined by the context of this struct (For example, if provided by a uniform state
/// the range is the entire buffer).
///
/// Availability and visibility of memory is not tracked. The execution engine requires that all
/// writes have been made available to the device domain after a execution. Because semaphore or
/// fence signal operations implicitly make all memory available to the device domain, this is in
/// most cases implicitly guaranteed.
/// Similarly a call to [vkQueueSubmit] will implicitly make all memory visible to all device stages
/// so it is not necessary to track visibility.
///
/// The only exception is host domain availability, which is tracked by [host_available]. Host
/// visibility is dependant on the memory type and may require a call to
/// [vkInvalidateMappedMemoryRanges].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct BufferRangeState {
    pub host_available: bool,
    pub queue: Option<u32>,
}

pub enum BufferState {
    Uniform(BufferRangeState),
    Split(BufferRangeState), // TODO
}

/// Describes the state of an image subresource range after a execution.
///
/// The subresource range is defined by the context of this struct (For example, if provided by a
/// uniform state the subresource range is the entire image).
///
/// Availability and visibility of memory is not tracked. The execution engine requires that all
/// writes have been made available to the device domain after a execution. Because semaphore or
/// fence signal operations implicitly make all memory available to the device domain, this is in
/// most cases implicitly guaranteed.
/// Similarly a call to [vkQueueSubmit] will implicitly make all memory visible to all device stages
/// so it is not necessary to track visibility.
///
/// The only exception is host domain availability, which is tracked by [host_available]. Host
/// visibility is dependant on the memory type and may require a call to
/// [vkInvalidateMappedMemoryRanges].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct ImageRangeState {
    pub host_available: bool,
    pub layout: ash::vk::ImageLayout,
    pub queue: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct ImageMipRange {
    base_level: u32,
    level_count: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct ImageArrayRange {
    base_layer: u32,
    layer_count: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct ImageMipArrayRange {
    mip_levels: ImageMipRange,
    array_layers: ImageArrayRange,
}

#[derive(Clone, Debug)]
pub enum ImageState {
    Uniform(ImageRangeState),
    SplitMipLevels(Vec<(ImageMipRange, ImageRangeState)>),
    SplitArrayLayers(Vec<(ImageArrayRange, ImageRangeState)>),
    SplitAll(Vec<(ImageMipArrayRange, ImageRangeState)>),
}

pub struct StateDictionary {

}