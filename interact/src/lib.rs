//! Interact
//!
//! Interact is a framework for friendly online introspection of the running program state in an
//! intuitive command-line interactive way.
//!
//! While dynamically-typed interpreted languages offer the advantage of allowing to look at a
//! running program state using a prompt, compiled languages often do not provide that feature.
//! Being hard as it is to introduce interpreters into compiled languages, the Interact project
//! aimes to provide a midway solution using stable Rust.
//!
//! # Usage
//!
//! NOTE: **Unless you are manually extending types for use under Interact, you probably don't need
//! most of the items that are exported in this crate**. Instead, look for the `interact_prompt` crate.
//!
//! # Design
//!
//! Interact introduces a series of traits, the main ones are `Access` and `Deser` trait. Those
//! crates can be custom-derived using `#[derive(Interact)]`, or be derived manually.
//!
//! The `Access` provides two methods that return special accessor trait object types. Please
//! read the documentation for the `access` part of Interact.
//!
//! The `Deser` trait is a special deserializer that allows for online interactive hints at
//! non-ambiguous parse points.
//!
//! Further relevent bits that comprise Interact are:
//!
//! * `reflector`, when provided at type it will generate a representation of it, while handling
//!    reference cycles, imposed output limitations, mutexs, and customized in-process indirections.
//! * `climber`, which when given a Rust-like expression of an inner value, knows how to go from an
//!    Interact root down to a field.

#[macro_use]
extern crate pest_derive;

//
// All the `pub use` here shows exactly what are the names that this crate exports.
//
#[doc(hidden)]
macro_rules! try_seen_dyn {
    ($e:expr, $self:expr) => {
        {
            let ptr = (&$e as *const _) as usize;
            let obj_ptr = (unsafe { *(ptr as *const usize) }, unsafe {
                *(ptr as *const usize).offset(1)
            });

            match Reflector::seen_ptr($self, obj_ptr) {
                Ok(v) => return v,
                Err(meta) => meta,
            }
        }
    }
}

// tokens
mod tokens;
pub use crate::tokens::{Token, TokenKind, TokenVec};

// deser
pub mod deser;
#[doc(inline)]
pub use crate::deser::Deser;

// reflector
#[macro_use]
mod reflector;
pub use crate::reflector::Reflector;

// access
pub mod access;
#[doc(hidden)]
pub use crate::access::{
    derive::{Enum, ReflectEnum, ReflectStruct, Struct, StructKind},
    iter::ReflectIter,
    Function,
};

#[doc(inline)]
pub use crate::access::{
    deser_assign, Access, AssignError, CallError, ImmutAccess, MutAccess, Reflect, ReflectDirect,
    ReflectIndirect, ReflectMut, RetValCallback,
};

// #derive
#[doc(hidden)]
pub use interact_derive::derive_interact_extern_opqaue;

pub use interact_derive::Interact;

// util
mod util;
pub use crate::assist::{Assist, NextOptions};
pub use crate::node_tree::{NodeInfo, NodeTree};

// climber
pub mod climber;
#[doc(inline)]
pub use crate::climber::{ClimbError, Climber};

#[doc(hidden)]
pub use crate::climber::{EnumOrStruct, EnumOrStructMut};

// root
pub mod root;
#[doc(inline)]
pub use crate::root::{Root, RootLocal, RootSend};

//
// Internally re-exported
//
use crate::expect::ExpectTree;
use crate::util::{assist, expect, node_tree};
