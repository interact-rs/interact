use std::borrow::Cow;
use std::sync::Arc;

use crate::access::ReflectDirect;
use crate::climber::{ClimbError, Climber};
use crate::node_tree::{NodeInfo, NodeTree};
use crate::reflector::Reflector;

use interact_derive::derive_interact_basic;

macro_rules! simple {
    ($a:tt, $fmt:expr) => {
        derive_interact_basic! {
            #[interact(mut_assign)]
            struct $a;
        }

        impl ReflectDirect for $a {
            fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
                let obj_ptr = ((self as *const _) as usize, 0);
                let meta = match Reflector::seen_ptr(reflector, obj_ptr) {
                    Ok(v) => return v,
                    Err(meta) => meta,
                };
                NodeInfo::Leaf(Cow::Owned(format!($fmt, self))).with_meta(meta)
            }

            fn immut_climber<'a>(
                &self,
                _climber: &mut Climber<'a>,
            ) -> Result<Option<NodeTree>, ClimbError> {
                Ok(None)
            }

            fn mut_climber<'a>(
                &mut self,
                _climber: &mut Climber<'a>,
            ) -> Result<Option<NodeTree>, ClimbError> {
                Ok(None)
            }
        }
    };
}

simple!(usize, "{}");
simple!(u64, "{}");
simple!(u32, "{}");
simple!(u16, "{}");
simple!(u8, "{}");
simple!(isize, "{}");
simple!(bool, "{}");
simple!(String, "{:?}");
simple!(char, "{:?}");
simple!(i64, "{}");
simple!(i32, "{}");
simple!(i16, "{}");
simple!(i8, "{}");
