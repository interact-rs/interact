use crate::deser::Deser;
use std::collections::BTreeMap;

// TODO: implement

impl<K, V> Deser for BTreeMap<K, V>
where
    K: Eq + Ord + Deser,
    V: Deser,
{
}
