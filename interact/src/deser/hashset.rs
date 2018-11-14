use std::collections::HashSet;
use std::hash::BuildHasher;
use std::hash::Hash;

use crate::deser::Deser;

// TODO: implement

impl<V, S> Deser for HashSet<V, S>
where
    V: Eq + Hash + Deser,
    S: BuildHasher,
{
}
