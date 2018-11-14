use crate::deser::Deser;

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

// TODO: implement

impl<K, V, S> Deser for HashMap<K, V, S>
where
    K: Eq + Hash + Deser,
    V: Deser,
    S: BuildHasher,
{
}
