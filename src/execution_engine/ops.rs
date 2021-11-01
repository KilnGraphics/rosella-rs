use std::any::Any;
use std::marker::PhantomData;

pub trait OpAllocator {
    type O: Op;

    fn allocate() -> Self::O;
}

pub trait Op : Any {
    fn as_any(&self) -> &dyn Any;
}

struct OpMetadata<'a> {
    // This box is unsafe. DO NOT DROP IT BEFORE 'a EXPIRES (which is when the struct is destroyed so really just dont drop it ever)
    op: Box<dyn Op>,
    phantom: PhantomData<&'a u8>,
}

impl<'a> OpMetadata<'a> {
    fn new(op: Box<dyn Op>) -> Self {
        Self{ op, phantom: PhantomData }
    }
}

pub struct OpList<'a> {
    ops: Vec<OpMetadata<'a>>,
}

impl<'a> OpList<'a> {
    pub fn allocate_add<T:OpAllocator>(&mut self) -> &'a mut T::O {
        let op = Box::leak(Box::new(T::allocate()));
        unsafe {
            self.ops.push(OpMetadata::new(Box::from_raw(op)));
        }
        op
    }
}