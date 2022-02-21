/*use ash::vk;
use crate::objects::ImageSubresourceRange;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AccessType {
    None,
    ReadPending,
    WritePending,
}

impl AccessType {
    pub const READ_MASK: vk::AccessFlags2KHR = vk::AccessFlags2KHR::INDIRECT_COMMAND_READ |
        vk::AccessFlags2KHR::INDEX_READ |
        vk::AccessFlags2KHR::VERTEX_ATTRIBUTE_READ |
        vk::AccessFlags2KHR::UNIFORM_READ |
        vk::AccessFlags2KHR::INPUT_ATTACHMENT_READ |
        vk::AccessFlags2KHR::SHADER_READ |
        vk::AccessFlags2KHR::COLOR_ATTACHMENT_READ |
        vk::AccessFlags2KHR::DEPTH_STENCIL_ATTACHMENT_READ |
        vk::AccessFlags2KHR::TRANSFER_READ |
        vk::AccessFlags2KHR::HOST_READ |
        vk::AccessFlags2KHR::MEMORY_READ |
        vk::AccessFlags2KHR::SHADER_SAMPLED_READ |
        vk::AccessFlags2KHR::SHADER_STORAGE_READ |
        vk::AccessFlags2KHR::VIDEO_DECODE_READ |
        vk::AccessFlags2KHR::VIDEO_ENCODE_READ |
        vk::AccessFlags2KHR::TRANSFORM_FEEDBACK_COUNTER_READ_EXT |
        vk::AccessFlags2KHR::CONDITIONAL_RENDERING_READ_EXT |
        vk::AccessFlags2KHR::ACCELERATION_STRUCTURE_READ |
        vk::AccessFlags2KHR::FRAGMENT_DENSITY_MAP_READ_EXT;

    pub const WRITE_MASK: vk::AccessFlags2KHR = vk::AccessFlags2KHR::SHADER_WRITE |
        vk::AccessFlags2KHR::COLOR_ATTACHMENT_WRITE |
        vk::AccessFlags2KHR::DEPTH_STENCIL_ATTACHMENT_WRITE |
        vk::AccessFlags2KHR::TRANSFER_WRITE |
        vk::AccessFlags2KHR::HOST_WRITE |
        vk::AccessFlags2KHR::MEMORY_WRITE |
        vk::AccessFlags2KHR::SHADER_STORAGE_WRITE |
        vk::AccessFlags2KHR::VIDEO_DECODE_WRITE |
        vk::AccessFlags2KHR::VIDEO_ENCODE_WRITE |
        vk::AccessFlags2KHR::TRANSFORM_FEEDBACK_WRITE_EXT |
        vk::AccessFlags2KHR::TRANSFORM_FEEDBACK_COUNTER_WRITE_EXT |
        vk::AccessFlags2KHR::ACCELERATION_STRUCTURE_WRITE;

    pub fn new(access_mask: vk::AccessFlags2KHR) -> Self {
        if access_mask == vk::AccessFlags2KHR::NONE {
            return Self::None;
        }

        // If there is any write bit set the whole access is a write
        if (access_mask & Self::WRITE_MASK) != vk::AccessFlags2KHR::NONE {
            return Self::WritePending;
        }

        // Ensure that all flags are read accesses. If not there must be some unknown bits
        if (access_mask & Self::READ_MASK) == access_mask {
            return Self::ReadPending;
        }

        panic!("Unknown flags bits");
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BufferState {
    pub access_mask: vk::AccessFlags2KHR,
    pub stage_mask: vk::PipelineStageFlags2KHR,
}

impl BufferState {
    pub fn new_empty() -> Self {
        Self {
            access_mask: vk::AccessFlags2KHR::NONE,
            stage_mask: vk::PipelineStageFlags2KHR::NONE,
        }
    }
}

pub struct AccessScopeInfo {
    pub access_mask: vk::AccessFlags2KHR,
    pub stage_mask: vk::PipelineStageFlags2KHR,
}

/// Tracks access to a buffer and generates required access scopes for synchronization.
pub struct BufferStateTracker {
    pre_state: Option<BufferState>,
    post_state: BufferState,
}

impl BufferStateTracker {
    pub fn new() -> Self {
        Self {
            pre_state: None,
            post_state: BufferState::new_empty(),
        }
    }

    /// Adds an access to the tracker.
    ///
    /// If the access requires a new access scope the old scope is returned. A memory barrier must
    /// be inserted before the old scope with the second access scope defined by the old scope as
    /// well as a barrier after the old scope with the first access scope defined by the old scope.
    pub fn add_access(&mut self, access_mask: vk::AccessFlags2KHR, stage_mask: vk::PipelineStageFlags2KHR) -> Option<AccessScopeInfo> {
        let self_type = AccessType::new(self.post_state.access_mask);
        let new_type = AccessType::new(access_mask);

        // If either are write accesses we need to start a new access scope, unless it is the first access
        if self_type == AccessType::WritePending || (new_type == AccessType::WritePending && self_type == AccessType::None) {
            if self.pre_state.is_none() {
                self.pre_state = Some(self.post_state);
            }

            let old_scope = AccessScopeInfo {
                access_mask: self.post_state.access_mask,
                stage_mask: self.post_state.stage_mask,
            };

            self.post_state.access_mask = access_mask;
            self.post_state.stage_mask = stage_mask;

            Some(old_scope)

        } else {
            self.post_state.access_mask |= access_mask;
            self.post_state.stage_mask |= stage_mask;

            None
        }
    }

    /// Returns the pre state of the buffer
    ///
    /// The pre state is equivalent to the first access scope the tracker generated.
    pub fn get_pre_state(&self) -> &BufferState {
        match &self.pre_state {
            None => &self.post_state,
            Some(state) => state,
        }
    }

    /// Returns the post state of the buffer
    ///
    /// The post state is equivalent to the last access scope the tracker generated.
    pub fn get_post_state(&self) -> &BufferState {
        &self.post_state
    }

    /// Returns true if the tracker has generated more than 1 access scope.
    ///
    /// If this returns false the pre and post state are the same.
    pub fn has_multiple_scopes(&self) -> bool {
        self.pre_state.is_some()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ImageSubresourceState {
    pub subresource_range: ImageSubresourceRange,
    pub layout: vk::ImageLayout,
    pub access_mask: vk::AccessFlags2KHR,
    pub stage_mask: vk::PipelineStageFlags2KHR,
}

#[derive(Clone, Debug)]
pub struct ImageState {
    access_type: AccessType,
    states: Vec<ImageSubresourceState>,
}

impl ImageState {
    pub fn new_empty() -> Self {
        Self {
            access_type: AccessType::None,
            states: Vec::with_capacity(2),
        }
    }

    pub fn get_subresource_states(&self) -> &[ImageSubresourceState] {
        self.states.as_slice()
    }

    pub fn get_subresource_states_mut(&mut self) -> &mut [ImageSubresourceState] {
        self.states.as_mut_slice()
    }

    pub fn get_subresource_state_vec(&mut self) -> &Vec<ImageSubresourceState> {
        &self.states
    }
}

pub struct ImageStateTracker {
    pre_state: Option<ImageState>,
    post_state: ImageState,
}

impl ImageStateTracker {
    pub fn new() -> Self {
        Self {
            pre_state: None,
            post_state: ImageState::new_empty(),
        }
    }

    pub fn add_access(&mut self, access_mask: vk::AccessFlags2KHR, stage_mask: vk::PipelineStageFlags2KHR, layout: vk::ImageLayout, subresource_range: ImageSubresourceRange) -> Option<()> {
        todo!()
        /*if self.post_state.states.is_empty() {
            self.post_state.states.push(ImageSubresourceState {
                subresource_range,
                layout,
                access_mask,
                stage_mask
            });

            None

        } else {
            let new_type = AccessType::new(access_mask);

            if self.post_state.access_type == AccessType::WritePending || new_type == AccessType::WritePending {

            }

            None
        }*/

    }
}*/