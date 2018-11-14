use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::sync::Arc;

use crate::access::{iter::ReflectIter, Access, ReflectDirect};
use crate::climber::{ClimbError, Climber};
use crate::deser;
use crate::node_tree::NodeTree;
use crate::reflector::Reflector;

impl<'a, K, V> ReflectIter<(&'a dyn Access, &'a dyn Access)>
    for std::collections::hash_map::Iter<'a, K, V>
where
    K: Eq + Hash + Access,
    V: Access,
{
    fn reflect_next(&mut self) -> Option<(&'a dyn Access, &'a dyn Access)> {
        match self.next() {
            None => None,
            Some((key, value)) => Some((key, value)),
        }
    }
}

impl<K, V, S> ReflectDirect for HashMap<K, V, S>
where
    K: Eq + Hash + Access + deser::Deser,
    V: Access,
    S: BuildHasher,
{
    fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
        let mut i = Box::new(self.iter());
        Reflector::reflect_map(reflector, &mut *i, "HashMap")
    }

    fn immut_climber<'a>(&self, climber: &mut Climber<'a>) -> Result<Option<NodeTree>, ClimbError> {
        if !climber.open_bracket() {
            return Ok(None);
        }

        let v = K::deser(&mut climber.borrow_tracker())
            .map(|x| <HashMap<K, V, S>>::get(self, &x))
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
            Ok(x) => Ok(match <HashMap<K, V, S>>::get_mut(self, &x) {
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
    #[interact(skip_bound(S))]
    #[interact(immut_fn(len()))]
    struct HashMap<K, V, S>
    where
        K: Eq + std::hash::Hash + deser::Deser,
        S: std::hash::BuildHasher;
}
