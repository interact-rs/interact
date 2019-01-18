use std::sync::Arc;

use crate::deser::Deser;
use crate::{deser, ClimbError, Climber, NodeTree, Reflector};

pub trait ReflectIndirect {
    fn indirect(&self, fnc: Box<FnMut(&dyn Access) + Send>);
    fn indirect_mut(&mut self, fnc: Box<FnMut(&mut dyn Access) + Send>);
}

pub trait ReflectDirect {
    fn immut_reflector(&self, _reflector: &Arc<Reflector>) -> NodeTree;
    fn immut_climber<'a>(&self, _climber: &mut Climber<'a>)
        -> Result<Option<NodeTree>, ClimbError>;
    fn mut_climber<'a>(
        &mut self,
        _climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError>;
}

pub enum Reflect<'a> {
    Indirect(&'a dyn ReflectIndirect),
    Direct(&'a dyn ReflectDirect),
}

pub struct Function {
    pub name: &'static str,
    pub args: &'static [&'static str],
}

pub struct MutAccess<'a> {
    pub reflect: ReflectMut<'a>,
    pub functions: &'static [Function],
}

impl<'a> MutAccess<'a> {
    pub fn no_funcs(reflect: ReflectMut<'a>) -> Self {
        Self {
            reflect,
            functions: &[],
        }
    }
}

pub struct ImmutAccess<'a> {
    pub reflect: Reflect<'a>,
    pub functions: &'static [Function],
}

impl<'a> ImmutAccess<'a> {
    pub fn no_funcs(reflect: Reflect<'a>) -> Self {
        Self {
            reflect,
            functions: &[],
        }
    }
}

pub enum ReflectMut<'a> {
    Indirect(&'a mut dyn ReflectIndirect),
    Direct(&'a mut dyn ReflectDirect),
    Immutable,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AssignError {
    Deser(deser::DeserError),

    // Some types, having ignored fields, will be unbuildable.
    Unbuildable,
    Immutable,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CallError {
    Deser(deser::DeserError),
    NeedMutable,
    NoSuchFunction,
}

pub type RetValCallback<'a> = Box<FnMut(&dyn Access, &mut Climber<'a>)>;

/// The `Access` trait, meant to be used as a trait objects provides methods that
/// dynamically expose read&write access to the underlying objects.
pub trait Access {
    /// Expose an immmutable accessor, used when `Access` is immutable or mutable.
    fn immut_access(&self) -> ImmutAccess;

    /// Expose a mutable accessor, used when `Access` is mutable.
    fn mut_access(&mut self) -> MutAccess;

    /// Perform an optional method call for a certain function, with the return value provided to
    /// the callback. The arguments are parsed from the Token tracker in the Climber parameter.
    ///
    /// Depending on the state of the Climber, we may just parsing the arguments not not actually
    /// calling the function, in order to provide user feedback.
    fn immut_call<'a>(
        &self,
        _func_name: &'static str,
        _climber: &mut Climber<'a>,
        mut _retcall: RetValCallback<'a>,
    ) -> Result<(), CallError> {
        Err(CallError::NoSuchFunction)
    }

    /// Perform an optional method call for a certain function which may modify the underlying
    /// value, with the return value provided to the callback. The arguments are parsed from the
    /// Token tracker in the Climber parameter.
    ///
    /// Depending on the state of the Climber, we may just parsing the arguments not not actually
    /// calling the function, in order to provide user feedback.
    fn mut_call<'a>(
        &mut self,
        _func_name: &'static str,
        _climber: &mut Climber<'a>,
        mut _retcall: RetValCallback<'a>,
    ) -> Result<(), CallError> {
        Err(CallError::NoSuchFunction)
    }

    /// Assign a new value to this object. `probe_only` determines whether the implementation would
    /// only parse the new value and not actually assign it. This is in order to provide user
    /// feedback for the parsing bits.
    fn mut_assign<'a, 'b>(
        &mut self,
        _tokens: &mut deser::Tracker<'a, 'b>,
        _probe_only: bool,
    ) -> Result<(), AssignError> {
        Err(AssignError::Unbuildable)
    }
}

macro_rules! mut_assign_deser {
    () => {
        fn mut_assign<'x, 'y>(
            &mut self,
            tracker: &mut deser::Tracker<'x, 'y>,
            probe_only: bool,
        ) -> Result<(), AssignError> {
            crate::access::deser_assign(self, tracker, probe_only)
        }
    }
}

/// A helper for the specific implementations of `Access` to use with `mut_assign` methods
pub fn deser_assign<'a, 'b, T: Deser>(
    dest: &mut T,
    tracker: &mut deser::Tracker<'a, 'b>,
    probe_only: bool,
) -> Result<(), AssignError> {
    match T::deser(tracker) {
        Ok(v) => {
            if !probe_only {
                *dest = v;
            }
            Ok(())
        }
        Err(e) => Err(AssignError::Deser(e)),
    }
}

mod basic;
mod btreemap;
mod derefs;
pub mod derive;
mod explicit;
mod hashmap;
mod hashset;
mod instant;
pub mod iter;
mod mutex;
mod refcell;
mod tuple;
pub mod vec;
