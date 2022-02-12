use ash::vk;
use crate::objects::ImageSubresourceRange;

pub enum AccessType {
    None,
    ReadPending,
    WritePending,
}

pub struct BufferState {
    access_type: AccessType,
    access_mask: vk::AccessFlags2KHR,
    stage_mask: vk::PipelineStageFlags2KHR,
}

pub struct ImageSubresourceState {
    subresource_range: ImageSubresourceRange,
}

pub struct ImageState {

}