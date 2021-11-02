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

struct OpMetadata<'a> {
    op: Box<dyn Op>,
    config: OpConfig,
    phantom: PhantomData<&'a u8>,
}

impl<'a> OpMetadata<'a> {
    fn new(op: Box<dyn Op>) -> Self {
        Self{ op, config: OpConfig::new(), phantom: PhantomData }
    }
}

pub struct OpList<'a> {
    ops: Vec<OpMetadata<'a>>,
}

impl<'a> OpList<'a> {
    pub fn allocate_add<T: OpAllocator, F: Fn(&mut T::O, &mut OpConfig) -> Result<(), &'static str>>(&mut self, setup: &F) -> Result<(), &'static str> {
        let mut metadata = OpMetadata::new(Box::new(T::allocate()));

        setup(metadata.op.as_mut().as_any_mut().downcast_mut().unwrap(), &mut metadata.config)?;

        self.ops.push(metadata);
        Result::Ok(())
    }
}