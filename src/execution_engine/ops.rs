//! Ops are a low level intermediate representation of vulkan commands.
//! The IR is organized as lists of ops where each list represents a single vulkan command buffer and
//! every op has a 1 to 1 mapping to a vulkan command. Synchronization commands are omitted and
//! vulkan objects are replaced with placeholder objects.

use std::any::Any;
use std::marker::PhantomData;

pub trait OpAllocator {
    type O: Op;

    fn allocate() -> Self::O;
}

pub trait Op : Any {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct OpConfig {
}

impl OpConfig {
    fn new() -> Self {
        Self{}
    }
}

pub struct OpEntry {
    pub op: Box<dyn Op>,
    pub config: OpConfig,
}

pub struct OpList {
    ops: Vec<OpEntry>,
}

impl OpList {
    pub fn allocate_add<'r, T: OpAllocator, F: Fn(&mut T::O, &mut OpConfig) -> Result<(), &'r str>>(&mut self, setup: &F) -> Result<(), &'r str> {
        let mut entry = OpEntry{ op: Box::new(T::allocate()), config: OpConfig::new() };

        setup(entry.op.as_mut().as_any_mut().downcast_mut().unwrap(), &mut entry.config)?;

        self.ops.push(entry);
        Result::Ok(())
    }

    pub fn get_entries(&self) -> &Vec<OpEntry> {
        &self.ops
    }
}