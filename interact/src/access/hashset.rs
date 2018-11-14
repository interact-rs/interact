use std::collections::HashSet;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::sync::Arc;

use crate::access::{iter::ReflectIter, Access, ReflectDirect};
use crate::climber::{ClimbError, Climber};
use crate::node_tree::NodeTree;
use crate::reflector::Reflector;

impl<'a, K> ReflectIter<&'a dyn Access> for std::collections::hash_set::Iter<'a, K>
where
    K: Eq + Hash + Access,
{
    fn reflect_next(&mut self) -> Option<&'a dyn Access> {
        match self.next() {
            None => None,
            Some(value) => Some(value),
        }
    }
}

impl<K, S> ReflectDirect for HashSet<K, S>
where
    K: Eq + Hash + Access,
    S: BuildHasher,
{
    fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
        let mut i = Box::new(self.iter());
        Reflector::reflect_set(reflector, &mut *i, "HashSet")
    }

    fn immut_climber<'a>(
        &self,
        _climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        return Ok(None);
    }

    fn mut_climber<'a>(
        &mut self,
        _climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        return Ok(None);
    }
}

use interact_derive::derive_interact_opaque;

derive_interact_opaque! {
    #[interact(skip_bound(S))]
    #[interact(immut_fn(len()))]
    struct HashSet<K, S>
    where
        K: Eq + std::hash::Hash,
        S: std::hash::BuildHasher;
}
