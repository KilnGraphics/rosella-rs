//! Ops are a low level intermediate representation of vulkan commands.
//! The IR is organized as lists of ops where each list represents a single vulkan command buffer and
//! every op has a 1 to 1 mapping to a vulkan command. Synchronization commands are omitted and
//! vulkan objects are replaced with placeholder objects.

use std::any::Any;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use ash::vk;
use bumpalo::Bump;
use ouroboros::self_referencing;

use crate::objects::{id, ImageSubresourceRange};

pub trait ObjectUsageRegistry {
    fn register_buffer(&mut self, id: id::BufferId, stages: vk::PipelineStageFlags2KHR, accesses: vk::AccessFlags2KHR);

    fn register_buffer_view(&mut self, id: id::BufferViewId, stages: vk::PipelineStageFlags2KHR, accesses: vk::AccessFlags2KHR);

    fn register_image(&mut self, id: id::ImageId, stages: vk::PipelineStageFlags2KHR, accesses: vk::AccessFlags2KHR, required_layout: vk::ImageLayout, range: ImageSubresourceRange);

    fn register_image_view(&mut self, id: id::ImageViewId, stages: vk::PipelineStageFlags2KHR, accesses: vk::AccessFlags2KHR, required_layout: vk::ImageLayout, range: ImageSubresourceRange);

    fn register_event(&mut self, id: id::EventId);
}

pub trait Op {
    fn get_used_objects(&self, registry: &mut dyn ObjectUsageRegistry);
}

#[derive(Copy, Clone)]
pub enum OpPreAction {
}

#[derive(Copy, Clone)]
pub enum OpPostAction {
}

pub struct OpEntry<'a> {
    op: &'a (dyn Op + 'a),
    pre: Option<bumpalo::boxed::Box<'a, [OpPreAction]>>,
    post: Option<bumpalo::boxed::Box<'a, [OpPostAction]>>,
}

impl<'a> OpEntry<'a> {
    fn make_boxed_list<T: Clone>(list: Option<&[T]>, allocator: &'a Bump) -> Option<bumpalo::boxed::Box<'a, [T]>> {
        if let Some(list) = list {
            if list.len() == 0 {
                None
            } else {
                Some(bumpalo::boxed::Box::from_iter_in(list.iter().cloned(), allocator))
            }
        } else {
            None
        }
    }

    pub fn new<T: Op + 'a>(op: T, allocator: &'a Bump) -> Self {
        Self {
            op: allocator.alloc(op),
            pre: None,
            post: None,
        }
    }

    pub fn new_actions<T: Op + 'a>(op: T, pre: Option<&[OpPreAction]>, post: Option<&[OpPostAction]>, allocator: &'a Bump) -> Self {
        Self {
            op: allocator.alloc(op),
            pre: Self::make_boxed_list(pre, allocator),
            post: Self::make_boxed_list(post, allocator),
        }
    }

    pub fn get_op(&self) -> &(dyn Op + 'a) {
        self.op
    }

    pub fn get_pre_actions(&self) -> Option<&[OpPreAction]> {
        if let Some(pre) = &self.pre {
            Some(pre.as_ref())
        } else {
            None
        }
    }

    pub fn get_post_actions(&self) -> Option<&[OpPostAction]> {
        if let Some(post) = &self.post {
            Some(post.as_ref())
        } else {
            None
        }
    }
}

impl<'a> Drop for OpEntry<'a> {
    fn drop(&mut self) {
        unsafe {
            // Needed because the reference must be non mut for covariance to work
            std::ptr::drop_in_place((self.op as *const (dyn Op)) as *mut (dyn Op))
        }
    }
}

struct OpListList<'a> {
    list: Vec<OpEntry<'a>>,
}

#[self_referencing]
struct OpListImpl {
    allocator: Bump,
    #[borrows(allocator)]
    #[covariant]
    list: OpListList<'this>,
}

pub struct OpList(OpListImpl);

impl OpList {
    pub fn new() -> Self {
        Self(OpListImplBuilder{ allocator: Bump::new(), list_builder: |_| OpListList{ list: Vec::new() } }.build())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(OpListImplBuilder{ allocator: Bump::new(), list_builder: |_| OpListList{ list: Vec::with_capacity(capacity) } }.build())
    }

    pub fn push<T: Op + Copy + 'static>(&mut self, op: T) {
        self.0.with_mut(|fields| {
            
        });
    }

    pub fn push_with<F>(&mut self, generator: F) where F: for<'a> FnOnce(&'a Bump) -> OpEntry<'a> {
        self.0.with_mut(|fields| {
            fields.list.list.push(generator(fields.allocator));
        });
    }

    pub fn get(&self) -> &[OpEntry] {
        self.0.borrow_list().list.as_slice()
    }
}

pub struct OpClearColorImage<'a> {
    image: id::ImageId,
    layout: vk::ImageLayout,
    ranges: bumpalo::boxed::Box<'a, [ImageSubresourceRange]>,
}