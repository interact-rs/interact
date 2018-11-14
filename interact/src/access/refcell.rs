use std::cell::RefCell;
use std::sync::Arc;

use crate::access::{Access, ReflectDirect};
use crate::climber::{ClimbError, Climber};
use crate::node_tree::{NodeInfo, NodeTree};
use crate::reflector::Reflector;

impl<T> ReflectDirect for RefCell<T>
where
    T: Access,
{
    fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
        match self.try_borrow() {
            Ok(borrowed) => Reflector::reflect(reflector, &*borrowed),
            Err(_) => NodeInfo::BorrowedMut.into_node(),
        }
    }

    fn immut_climber<'a>(&self, climber: &mut Climber<'a>) -> Result<Option<NodeTree>, ClimbError> {
        climber.refcell_handling(self).map(Some)
    }

    fn mut_climber<'a>(
        &mut self,
        climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        climber.refcell_handling(self).map(Some)
    }
}

use interact_derive::derive_interact_opaque;

derive_interact_opaque! {
    struct RefCell<T>;
}
