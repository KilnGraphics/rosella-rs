//! Placeholder objects are used in the ops IR to represent vulkan objects. A placeholder object
//! can either be a placeholder or fully defined.
//!
//! A placeholder can later be specialized into different objects at a commands level without needing
//! to recompile the entire program. Since memory allocation takes place during the ops compile stage
//! a placeholder object must be specialized by an external object.
//!
//! Fully defined objects on the other hand will be fixed after the ops compile stage. They can either
//! be dynamically allocated by the ops compiler or be set to some external object.

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(1);

macro_rules! define_object_reference {
    ($name: ident, $id_ty: ident, $def_ty: ident, $ref_ty: ident, $info_ty: ident) => {
        #[doc = concat!("A unique id referencing a ", stringify!($name))]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $id_ty(u64);

        impl $id_ty {
            pub fn new() -> Self {
                Self(NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed))
            }

            pub fn as_u64(&self) -> u64 {
                self.0
            }
        }

        #[derive(Copy, Clone)]
        pub struct $def_ty<'a> {
            info: &'a $info_ty,
            id: $id_ty,
        }

        impl<'a> $def_ty<'a> {
            pub fn get_id(&self) -> $id_ty {
                self.id
            }

            pub fn get_info(&self) -> &'a $info_ty {
                self.info
            }
        }

        impl<'a> PartialEq for $def_ty<'a> {
            fn eq(&self, other: &Self) -> bool {
                self.get_id() == other.get_id()
            }
        }

        impl<'a> PartialEq<$id_ty> for $def_ty<'a> {
            fn eq(&self, other: &$id_ty) -> bool {
                &self.get_id() == other
            }
        }


        #[derive(Copy, Clone)]
        pub enum $ref_ty<'a> {
            Defined($def_ty<'a>),
            Placeholder($id_ty),
        }

        impl<'a> $ref_ty<'a> {
            pub fn get_id(&self) -> $id_ty {
                match self {
                    $ref_ty::Defined(ref def) => def.get_id(),
                    $ref_ty::Placeholder(id) => *id,
                }
            }

            pub fn get_info(&self) -> Option<&'a $info_ty> {
                match self {
                    $ref_ty::Defined(ref def) => Some(def.get_info()),
                    $ref_ty::Placeholder(_) => None,
                }
            }

            pub fn is_defined(&self) -> bool {
                match self {
                    $ref_ty::Defined(_) => true,
                    $ref_ty::Placeholder(_) => false,
                }
            }

            pub fn is_placeholder(&self) -> bool {
                match self {
                    $ref_ty::Defined(_) => false,
                    $ref_ty::Placeholder(_) => true,
                }
            }
        }

        impl<'a> PartialEq for $ref_ty<'a> {
            fn eq(&self, other: &Self) -> bool {
                self.get_id() == other.get_id()
            }
        }

        impl<'a> PartialEq<$id_ty> for $ref_ty<'a> {
            fn eq(&self, other: &$id_ty) -> bool {
                &self.get_id() == other
            }
        }

        impl<'a> PartialEq<$def_ty<'a>> for $ref_ty<'a> {
            fn eq(&self, other: &$def_ty<'a>) -> bool {
                self.get_id() == other.get_id()
            }
        }
    }
}

pub enum BufferInfo {
    External{ },
    Allocate{},
}

pub struct BufferViewInfo {
}

pub struct ImageInfo {
}

pub struct ImageViewInfo {
}

define_object_reference!(Buffer, BufferId, DefinedBuffer, BufferReference, BufferInfo);
define_object_reference!(BufferView, BufferViewId, DefinedBufferView, BufferViewReference, BufferViewInfo);
define_object_reference!(Image, ImageId, DefinedImage, ImageReference, ImageInfo);
define_object_reference!(ImageView, ImageViewId, DefinedImageView, ImageViewReference, ImageViewInfo);

pub struct PlaceholderObjectSet {

}

impl PlaceholderObjectSet {
    pub fn new() -> Self {
        PlaceholderObjectSet{}
    }

    pub fn add_placeholder_buffer(&mut self) -> BufferReference {
        todo!()
    }


}