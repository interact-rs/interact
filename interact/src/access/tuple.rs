use std::borrow::Cow;
use std::sync::Arc;

use crate::access::{
    derive::{ReflectStruct, Struct, StructKind},
    Access, AssignError, ImmutAccess, MutAccess, Reflect, ReflectDirect, ReflectMut,
};
use crate::climber::{ClimbError, Climber, EnumOrStruct, EnumOrStructMut};
use crate::deser::{self, Deser};
use crate::node_tree::{NodeInfo, NodeTree};
use crate::reflector::Reflector;

macro_rules! tuple {
    ($count:expr; { $(($n:ident, $i:tt)),* }) => {

        impl<$($n),*> ReflectStruct for ($($n),*)
            where $($n : Access),*
        {
            fn get_desc(&self) -> Struct {
                Struct {
                    name: "",
                    kind: StructKind::Tuple($count),
                }
            }

            fn get_field_by_name(&self, _: &'static str) -> Option<&dyn Access> {
                None
            }

            fn get_field_by_idx(&self, idx: usize) -> Option<&dyn Access> {
                $(if idx == $i { return Some(&self.$i) });*
                None
            }

            fn get_field_by_name_mut(&mut self, _: &'static str) -> Option<&mut dyn Access> {
                None
            }

            fn get_field_by_idx_mut(&mut self, idx: usize) -> Option<&mut dyn Access> {
                $(if idx == $i { return Some(&mut self.$i) });*
                None
            }
        }

        impl<$($n),*> ReflectDirect for ($($n),*)
            where $($n : Access),*
        {
            fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
                Reflector::reflect_struct(reflector, &self.get_desc(), self, true)
            }

            fn immut_climber<'a>(
                &self,
                climber: &mut Climber<'a>,
            ) -> Result<Option<NodeTree>, ClimbError> {
                climber.check_field_access_immut(&EnumOrStruct::Struct(self))
            }

            fn mut_climber<'a>(
                &mut self,
                climber: &mut Climber<'a>,
            ) -> Result<Option<NodeTree>, ClimbError> {
                climber.check_field_access_mut(EnumOrStructMut::Struct(self))
            }
        }

        impl<$($n),*> Access for ($($n),*)
            where $($n : Access + Deser),*
        {
            fn immut_access(&self) -> ImmutAccess {
                ImmutAccess::no_funcs(Reflect::Direct(self))
            }

            fn mut_access(&mut self) -> MutAccess {
                MutAccess::no_funcs(ReflectMut::Direct(self))
            }

            mut_assign_deser!();
        }
    }
}

impl<A> ReflectStruct for (A,)
where
    A: Access,
{
    fn get_desc(&self) -> Struct {
        Struct {
            name: "",
            kind: StructKind::Tuple(1),
        }
    }

    fn get_field_by_name(&self, _: &'static str) -> Option<&dyn Access> {
        None
    }

    fn get_field_by_idx(&self, idx: usize) -> Option<&dyn Access> {
        if idx == 0 {
            return Some(&self.0);
        }
        None
    }

    fn get_field_by_name_mut(&mut self, _: &'static str) -> Option<&mut dyn Access> {
        None
    }

    fn get_field_by_idx_mut(&mut self, idx: usize) -> Option<&mut dyn Access> {
        if idx == 0 {
            return Some(&mut self.0);
        }
        None
    }
}

impl<T> ReflectDirect for (T,)
where
    T: Access,
{
    fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
        Reflector::reflect_struct(reflector, &self.get_desc(), self, true)
    }

    fn immut_climber<'a>(&self, climber: &mut Climber<'a>) -> Result<Option<NodeTree>, ClimbError> {
        climber.check_field_access_immut(&EnumOrStruct::Struct(self))
    }

    fn mut_climber<'a>(
        &mut self,
        climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        climber.check_field_access_mut(EnumOrStructMut::Struct(self))
    }
}

impl<A> Access for (A,)
where
    A: Access + Deser,
{
    fn immut_access(&self) -> ImmutAccess {
        ImmutAccess::no_funcs(Reflect::Direct(self))
    }

    fn mut_access(&mut self) -> MutAccess {
        MutAccess::no_funcs(ReflectMut::Direct(self))
    }

    mut_assign_deser!();
}

impl ReflectDirect for () {
    fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
        let obj_ptr = ((self as *const _) as usize, 0);
        let meta = match Reflector::seen_ptr(reflector, obj_ptr) {
            Ok(v) => return v,
            Err(meta) => meta,
        };
        NodeInfo::Leaf(Cow::Borrowed("()")).with_meta(meta)
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

impl Access for () {
    fn immut_access(&self) -> ImmutAccess {
        ImmutAccess::no_funcs(Reflect::Direct(self))
    }

    fn mut_access(&mut self) -> MutAccess {
        MutAccess::no_funcs(ReflectMut::Direct(self))
    }

    mut_assign_deser!();
}

tuple!(2; {(A, 0), (B, 1)});
tuple!(3; {(A, 0), (B, 1), (C, 2)});
tuple!(4; {(A, 0), (B, 1), (C, 2), (D, 3)});
tuple!(5; {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4)});
tuple!(6; {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5)});
tuple!(7; {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6)});
tuple!(8; {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7)});
tuple!(9; {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8)});
tuple!(10; {(A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9)});
