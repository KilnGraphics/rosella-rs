use std::sync::{Arc, Mutex};
use ash::vk;

pub struct Submission {
    command_buffers: Vec<vk::CommandBuffer>,
    queue: Arc<Mutex<vk::Queue>>,
    wait_groups: Vec<usize>,
}

pub struct Executable {
    submissions: Vec<Submission>,
}

impl Executable {
    pub fn submit(&mut self) {

    }
}