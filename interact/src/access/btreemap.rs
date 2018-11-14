use std::collections::BTreeMap;
use std::sync::Arc;

use crate::access::{iter::ReflectIter, Access, ReflectDirect};
use crate::climber::{ClimbError, Climber};
use crate::deser;
use crate::node_tree::NodeTree;
use crate::reflector::Reflector;

impl<'a, K, V> ReflectIter<(&'a dyn Access, &'a dyn Access)>
    for std::collections::btree_map::Iter<'a, K, V>
where
    K: Eq + Access,
    V: Access,
{
    fn reflect_next(&mut self) -> Option<(&'a dyn Access, &'a dyn Access)> {
        match self.next() {
            None => None,
            Some((key, value)) => Some((key, value)),
        }
    }
}

impl<K, V> ReflectDirect for BTreeMap<K, V>
where
    K: Eq + Ord + Access + deser::Deser,
    V: Access,
{
    fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
        let mut i = Box::new(self.iter());
        Reflector::reflect_map(reflector, &mut *i, "BTreeMap")
    }

    fn immut_climber<'a>(&self, climber: &mut Climber<'a>) -> Result<Option<NodeTree>, ClimbError> {
        if !climber.open_bracket() {
            return Ok(None);
        }

        let v = K::deser(&mut climber.borrow_tracker())
            .map(|x| <BTreeMap<K, V>>::get(self, &x))
            .map(|x| x.map(|y| y as &dyn Access));
        let v = match v {
            Ok(None) => return Err(ClimbError::NotFound),
            Err(err) => return Err(ClimbError::DeserError(err)),
            Ok(Some(v)) => v,
        };

        climber.close_bracket()?;

        return climber.general_access_immut(v).map(Some);
    }

    fn mut_climber<'a>(
        &mut self,
        climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        if !climber.open_bracket() {
            return Ok(None);
        }

        let v = match K::deser(&mut climber.borrow_tracker()) {
            Ok(x) => Ok(match <BTreeMap<K, V>>::get_mut(self, &x) {
                None => None,
                Some(x) => Some(x),
            }),
            Err(e) => Err(e),
        };
        let v = match v {
            Ok(None) => return Err(ClimbError::NotFound),
            Err(err) => return Err(ClimbError::DeserError(err)),
            Ok(Some(v)) => v,
        };

        climber.close_bracket()?;

        return climber.general_access_mut(v).map(Some);
    }
}

use interact_derive::derive_interact_opaque;

derive_interact_opaque! {
    #[interact(immut_fn(len()))]
    struct BTreeMap<K, V>
    where
        K: Eq + Ord + deser::Deser;
}
