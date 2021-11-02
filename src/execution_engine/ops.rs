use std::any::Any;
use std::marker::PhantomData;

pub trait OpAllocator {
    type O: Op;

    fn allocate() -> Self::O;
}

pub trait Op : Any {
    fn as_any(&self) -> &dyn Any;
}

pub struct OpMetadata<'a> {
    op: Box<dyn Op>,
    phantom: PhantomData<&'a u8>,
}

impl<'a> OpMetadata<'a> {
    fn new(op: Box<dyn Op>) -> Self {
        Self{ op, phantom: PhantomData }
    }

    fn borrow_as_op<T: Op>(&mut self) -> &mut T {
        self.op.as_mut().as_any().downcast_mut().unwrap()
    }
}

pub struct OpList<'a> {
    ops: Vec<OpMetadata<'a>>,
}

impl<'a> OpList<'a> {
    pub fn allocate_add<T: OpAllocator, F: Fn(&mut T::O, &mut OpMetadata) -> Result<(), &str>>(&mut self, setup: &F) -> Result<(), &str> {
        let mut metadata = OpMetadata::new(Box::new(T::allocate()));

        setup(metadata.borrow_as_op(), &mut metadata)?;

        self.ops.push(op);
        Result::Ok(())
    }
}