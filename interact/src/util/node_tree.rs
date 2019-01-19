use std::borrow::Cow;
use std::io::Cursor;
use std::io::Write;
use std::ops::Deref;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

type Delimiter = char;

#[derive(Debug)]
pub enum NodeInfo {
    Grouped(char, Box<NodeTree>, char),
    Delimited(Delimiter, Vec<NodeTree>),
    Named(Box<NodeTree>, Box<NodeTree>),
    Tuple(Box<NodeTree>, &'static str, Box<NodeTree>),
    Leaf(Cow<'static, str>),
    Hole(Box<Receiver<NodeTree>>),
    BorrowedMut,
    Locked,
    Repeated,
    Limited,
}

pub type PtrMeta = Arc<AtomicUsize>;

#[derive(Debug)]
pub struct Wrap(pub PtrMeta);

/// NodeTree represent a reflection of an Interact type that implemented the `Access` trait. It may
/// be a partial reflection due to limits and indirections (see `Reflector`).
#[derive(Debug)]
pub struct NodeTree {
    pub info: NodeInfo,
    pub meta: Option<Wrap>,
    pub size: usize,
}

impl NodeTree {
    pub fn new(info: NodeInfo, meta: Option<Wrap>) -> Self {
        Self {
            info,
            meta,
            size: 0,
        }
    }
}

impl Eq for Wrap {}
impl PartialEq for Wrap {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

struct Printer {
    accum: Cursor<Vec<u8>>,
}

impl Printer {
    fn write(&mut self, s: &str) -> Result<(), std::io::Error> {
        self.accum.write_all(s.as_bytes())?;
        Ok(())
    }
}

impl NodeInfo {
    pub fn into_node(self) -> NodeTree {
        NodeTree {
            info: self,
            meta: None,
            size: 0,
        }
    }

    pub fn with_meta(self, ptr_meta: PtrMeta) -> NodeTree {
        NodeTree {
            info: self,
            meta: Some(Wrap(ptr_meta)),
            size: 0,
        }
    }

    pub fn named(name: &'static str, a_self: NodeTree) -> Self {
        NodeInfo::Named(
            Box::new(NodeTree {
                info: NodeInfo::Leaf(Cow::Borrowed(name)),
                meta: None,
                size: 0,
            }),
            Box::new(a_self),
        )
    }

    pub fn format(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut state = Printer {
            accum: Cursor::new(Vec::new()),
        };

        self.inner_pretty_print(&mut state)?;

        let Printer { accum, .. } = state;

        Ok(accum.into_inner())
    }

    fn inner_pretty_print(&self, state: &mut Printer) -> Result<(), std::io::Error> {
        use crate::NodeInfo::*;

        match self {
            Grouped(prefix, sub, end) => {
                let space = match sub.deref().info {
                    Delimited(_, ref v) if v.is_empty() => "",
                    _ => " ",
                };

                state.write(&format!("{}{}", prefix, space))?;
                sub.info.inner_pretty_print(state)?;
                state.write(&format!("{}{}", space, end))?;
            }
            Delimited(delimiter, v) => {
                for (idx, i) in v.iter().enumerate() {
                    if idx > 0 {
                        state.write(&format!("{} ", delimiter))?;
                    }

                    i.info.inner_pretty_print(state)?;
                }
            }
            Tuple(key, sep, value) => {
                key.info.inner_pretty_print(state)?;
                state.write(&format!(" {} ", sep))?;
                value.info.inner_pretty_print(state)?;
            }
            Named(item, next) => {
                item.info.inner_pretty_print(state)?;
                state.write(&" ")?;
                next.info.inner_pretty_print(state)?;
            }
            Leaf(s) => {
                state.write(s)?;
            }
            Hole(_) => {
                state.write(&format!("<hole>"))?;
            }
            BorrowedMut => {
                state.write(&format!("<borrowed-mut>"))?;
            }
            Locked => {
                state.write(&format!("<locked>"))?;
            }
            Limited => {
                state.write("...")?;
            }
            Repeated => {
                state.write(&format!("<repeated>"))?;
            }
        };

        Ok(())
    }
}

impl NodeTree {
    pub fn resolve(&mut self) -> usize {
        use crate::NodeInfo::*;

        loop {
            let mut count = 1;

            let r = match &mut self.info {
                Grouped(_, sub, _) => {
                    count += 2 + sub.resolve();
                    None
                }
                Delimited(_, v) => {
                    for i in v.iter_mut() {
                        count += i.resolve() + 2;
                    }
                    None
                }
                Tuple(key, sep, value) => {
                    count += key.resolve();
                    count += sep.len() + 2;
                    count += value.resolve();
                    None
                }
                Named(item, next) => {
                    count += item.resolve();
                    count += next.resolve();
                    None
                }
                Leaf(v) => {
                    count += v.len();
                    None
                }
                Hole(receiver) => Some((*receiver).recv().unwrap()),
                Limited => None,
                Repeated => None,
                BorrowedMut => None,
                Locked => None,
            };

            if let Some(r) = r {
                *self = r;
                continue;
            }

            self.size = count;

            return count;
        }
    }
}

use std::fmt;

impl fmt::Display for NodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = self.format().unwrap();

        write!(f, "{}", String::from_utf8_lossy(v.as_slice()))
    }
}
