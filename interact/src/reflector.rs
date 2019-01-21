use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use crate::access::vec::ReflectVec;
use crate::access::{
    derive::{ReflectStruct, Struct, StructKind},
    iter::ReflectIter,
    Access,
};
use crate::node_tree::{NodeInfo, NodeTree, PtrMeta, Wrap};

type ObjPtr = (usize, usize);

/// `Reflector` operates on types implementing `Access`. Some of its methods are behind called
/// automatically from `#[derive(Interact)]` impls. It provides a thread-safe context, because on
/// the extreme case, where it is possible that reflection is done via indirection using multiple
/// process threads (see `ReflectIndirect`).
pub struct Reflector {
    limit: usize,
    used: AtomicUsize,

    seen: Mutex<HashMap<ObjPtr, PtrMeta>>,
    synced_thread: ThreadId,
}

impl Reflector {
    pub fn new(limit: usize) -> Arc<Self> {
        Arc::new(Self {
            limit,
            used: AtomicUsize::new(0),
            seen: Mutex::new(HashMap::new()),
            synced_thread: std::thread::current().id(),
        })
    }

    pub fn reflect_struct(
        a_self: &Arc<Self>,
        desc: &Struct,
        p_struct: &dyn ReflectStruct,
        anon: bool,
    ) -> NodeTree {
        let meta = try_seen_dyn!(p_struct, a_self);

        match &desc.kind {
            StructKind::Unit => {
                NodeInfo::Leaf(std::borrow::Cow::Borrowed(desc.name)).with_meta(meta)
            }
            StructKind::Tuple(n) => {
                let mut v = vec![];

                for i in 0..*n {
                    if a_self.limit <= a_self.used.load(Ordering::Relaxed) {
                        v.push(NodeInfo::Limited.into_node());
                        break;
                    }

                    let reflect_node = Self::reflect(a_self, p_struct.get_field_by_idx(i).unwrap());
                    v.push(reflect_node);
                }

                let grouped =
                    NodeInfo::Grouped('(', Box::new(NodeInfo::Delimited(',', v).into_node()), ')');

                let elem = if !desc.name.is_empty() && !anon {
                    NodeInfo::named(desc.name, grouped.into_node())
                } else {
                    grouped
                };

                elem.with_meta(meta)
            }
            StructKind::Fields(fields) => {
                let mut result = vec![];
                let mut items = vec![];
                let mut missing_keys = false;

                for field in *fields {
                    if a_self.limit <= a_self.used.load(Ordering::Relaxed) {
                        missing_keys = true;
                        break;
                    }

                    a_self.used.fetch_add(1, Ordering::SeqCst);
                    items.push((field, p_struct.get_field_by_name(field).unwrap()));
                }

                for (key, value) in items.into_iter() {
                    let node = {
                        if a_self.limit <= a_self.used.load(Ordering::Relaxed) {
                            NodeInfo::Limited.into_node()
                        } else {
                            Self::reflect(a_self, value)
                        }
                    };

                    result.push(
                        NodeInfo::Tuple(
                            Box::new(NodeInfo::Leaf(std::borrow::Cow::Borrowed(key)).into_node()),
                            ":",
                            Box::new(node),
                        )
                        .into_node(),
                    )
                }

                if missing_keys {
                    result.push(NodeInfo::Limited.into_node());
                }

                let grouped = NodeInfo::Grouped(
                    '{',
                    Box::new(NodeInfo::Delimited(',', result).into_node()),
                    '}',
                );

                let elem = if !desc.name.is_empty() && !anon {
                    NodeInfo::named(desc.name, grouped.into_node())
                } else {
                    grouped
                };

                elem.with_meta(meta)
            }
        }
    }

    pub fn reflect_map(
        a_self: &Arc<Self>,
        iter: &mut dyn ReflectIter<(&dyn Access, &dyn Access)>,
        name: &'static str,
    ) -> NodeTree {
        let meta = try_seen_dyn!(iter, a_self);

        let mut result = vec![];
        let mut items = vec![];
        let mut missing_keys = false;

        while let Some((key, value)) = iter.reflect_next() {
            if a_self.limit <= a_self.used.load(Ordering::Relaxed) {
                missing_keys = true;
                break;
            }

            a_self.used.fetch_add(1, Ordering::SeqCst);
            items.push((Self::reflect(a_self, key), value));
        }

        for (key, value) in items.into_iter() {
            let node = {
                if a_self.limit <= a_self.used.load(Ordering::Relaxed) {
                    NodeInfo::Limited.into_node()
                } else {
                    Self::reflect(a_self, value)
                }
            };

            let reflect_node = NodeInfo::Tuple(Box::new(key), ":", Box::new(node));
            result.push(reflect_node.into_node());
        }

        if missing_keys {
            result.push(NodeInfo::Limited.into_node());
        }

        NodeInfo::named(
            name,
            NodeInfo::Grouped(
                '{',
                Box::new(NodeInfo::Delimited(',', result).into_node()),
                '}',
            )
            .into_node(),
        )
        .with_meta(meta)
    }

    pub fn reflect_set(
        a_self: &Arc<Self>,
        iter: &mut dyn ReflectIter<&dyn Access>,
        name: &'static str,
    ) -> NodeTree {
        let mut v = vec![];
        let meta = try_seen_dyn!(iter, a_self);

        while let Some(member) = iter.reflect_next() {
            if a_self.limit <= a_self.used.load(Ordering::Relaxed) {
                v.push(NodeInfo::Limited.into_node());
                break;
            }

            let member = Self::reflect(a_self, member);
            v.push(member);
        }

        NodeInfo::named(
            name,
            NodeInfo::Grouped('{', Box::new(NodeInfo::Delimited(',', v).into_node()), '}')
                .into_node(),
        )
        .with_meta(meta)
    }

    pub fn reflect_vec(a_self: &Arc<Self>, vec: &dyn ReflectVec, name: &'static str) -> NodeTree {
        let mut v = vec![];

        let meta = try_seen_dyn!(vec, a_self);

        for i in 0..vec.get_len() {
            if a_self.limit <= a_self.used.load(Ordering::Relaxed) {
                v.push(NodeInfo::Limited.into_node());
                break;
            }

            let reflect_node = Self::reflect(a_self, vec.get_item(i).unwrap());
            v.push(reflect_node);
        }

        let item = NodeInfo::Grouped('[', Box::new(NodeInfo::Delimited(',', v).into_node()), ']');

        let item = if !name.is_empty() {
            NodeInfo::named(name, item.into_node())
        } else {
            item
        };

        item.with_meta(meta)
    }

    pub fn seen_ptr(a_self: &Arc<Self>, obj_ptr: ObjPtr) -> Result<NodeTree, PtrMeta> {
        let mut seen = a_self.seen.lock().unwrap();
        match seen.entry(obj_ptr) {
            Entry::Occupied(entry) => {
                let entry = entry.get();
                entry.fetch_add(1, Ordering::SeqCst);

                Ok(NodeTree::new(NodeInfo::Repeated, Some(Wrap(entry.clone()))))
            }
            Entry::Vacant(entry) => {
                let meta = Arc::new(AtomicUsize::new(1));
                entry.insert(meta.clone());
                Err(meta)
            }
        }
    }

    pub fn reflect(a_self: &Arc<Self>, access: &dyn Access) -> NodeTree {
        use crate::Reflect::*;

        let immut_access = access.immut_access();

        let reflect_node = match immut_access.reflect {
            Direct(v) => v.immut_reflector(a_self),
            Indirect(access) => {
                let (sender, receiver) = channel();
                let b_self = a_self.clone();

                access.indirect(Box::new(move |access| {
                    let res = Self::reflect(&b_self, access);
                    let _ = sender.send(res);
                }));

                if a_self.synced_thread == std::thread::current().id() {
                    receiver.recv().unwrap()
                } else {
                    NodeInfo::Hole(Box::new(receiver)).into_node()
                }
            }
        };

        a_self.used.fetch_add(1, Ordering::SeqCst);
        reflect_node
    }
}
