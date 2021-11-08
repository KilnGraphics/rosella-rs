//! Commands are the last intermediate representation before conversion into vulkan command buffers.
//! The IR is designed to be a direct mapping to vulkan commands with only placeholders for
//! specializable resources and external synchronization for them left unresolved.

use crate::execution_engine::placeholder_objects::*;

pub struct QueueRecorder {

}

pub trait Command {
    fn record(&self, recorder: &mut QueueRecorder, specialization_set: &SpecializationSet) -> Result<(), &'static str>;
}

pub struct CommandList {
    commands: Vec<Box<dyn Command>>,
}

pub struct UnspecializedExecutable {

}

pub struct Executable {
    
}