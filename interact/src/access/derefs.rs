use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::Arc;

use crate::access::{Access, AssignError, ImmutAccess, MutAccess, ReflectMut};
use crate::deser::{self, Deser};

impl<'a, T: 'a> Access for &'a T
where
    T: Access,
{
    fn immut_access(&self) -> ImmutAccess {
        (*self).immut_access()
    }

    fn mut_access(&mut self) -> MutAccess {
        MutAccess::no_funcs(ReflectMut::Immutable)
    }

    fn mut_assign<'c, 'b>(
        &mut self,
        _tracker: &mut deser::Tracker<'c, 'b>,
        _probe_only: bool,
    ) -> Result<(), AssignError> {
        Err(AssignError::Immutable)
    }
}

impl<'a, T: 'a> Access for &'a mut T
where
    T: Access + Deser,
{
    fn immut_access(&self) -> ImmutAccess {
        (**self).immut_access()
    }

    fn mut_access(&mut self) -> MutAccess {
        (*self).mut_access()
    }

    mut_assign_suggest!();
}

impl<T> Access for Box<T>
where
    T: Access + Deser,
{
    fn immut_access(&self) -> ImmutAccess {
        self.deref().immut_access()
    }

    fn mut_access(&mut self) -> MutAccess {
        self.deref_mut().mut_access()
    }

    mut_assign_suggest!();
}

impl<T> Access for Rc<T>
where
    T: Access + Deser,
{
    fn immut_access(&self) -> ImmutAccess {
        self.deref().immut_access()
    }

    fn mut_access(&mut self) -> MutAccess {
        match Rc::get_mut(self) {
            None => MutAccess::no_funcs(ReflectMut::Immutable),
            Some(mutref) => mutref.mut_access(),
        }
    }

    mut_assign_suggest!();
}

impl<T> Access for Arc<T>
where
    T: Access + Deser,
{
    fn immut_access(&self) -> ImmutAccess {
        self.deref().immut_access()
    }

    fn mut_access(&mut self) -> MutAccess {
        match Arc::get_mut(self) {
            None => MutAccess::no_funcs(ReflectMut::Immutable),
            Some(mutref) => mutref.mut_access(),
        }
    }

    mut_assign_suggest!();
}
