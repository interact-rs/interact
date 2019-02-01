/// This module defines the main traits used to dynamically operate and reflect on Rust types using Interact.
use std::sync::Arc;

use crate::deser::Deser;
use crate::{deser, ClimbError, Climber, NodeTree, Reflector};

/// The indirect Reflect allows indirect climber or reflector access, and meant to be used as a
/// trait object for that purpose.
///
/// It is expected that the provided callback would be called at this or some other thread in order
/// to continue traversal of the access expression. For example, if a processes uses internal message
/// passing, the traversal can continue upon message reception.
pub trait ReflectIndirect {
    /// Provides indirection for immutable access.
    fn indirect(&self, fnc: Box<FnMut(&dyn Access) + Send>);

    /// Provides indirection for mutable access.
    fn indirect_mut(&mut self, fnc: Box<FnMut(&mut dyn Access) + Send>);
}

/// The direct Reflect allows direct climber or reflector access, and meant
/// to be used as a trait object for that purpose.
pub trait ReflectDirect {
    /// The specific implementation of the following method will mostly likely call
    /// Reflector::reflect with the specific type.
    fn immut_reflector(&self, _reflector: &Arc<Reflector>) -> NodeTree;

    /// Implement climbing for the specific type. Returns a reflection of the inner value,
    /// depending on the expression remaining to parse.
    fn immut_climber<'a>(&self, _climber: &mut Climber<'a>)
        -> Result<Option<NodeTree>, ClimbError>;

    /// Implement mutable climbing for the specific type, allowing to modifying it.
    /// Returns a reflection of the inner value, depending on the expression remaining to parse.
    fn mut_climber<'a>(
        &mut self,
        _climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError>;
}

/// An arbitrar between the two possible way to climb into an immutable value.
pub enum Reflect<'a> {
    Indirect(&'a dyn ReflectIndirect),
    Direct(&'a dyn ReflectDirect),
}

pub struct Function {
    pub name: &'static str,
    pub args: &'static [&'static str],
}

/// MutAccess adds function call information over `ReflectMut`.
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

/// ImmutAccess adds function call information over `Reflect`.
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

/// An arbitrar between the two possible way to climb into a mutable value.
pub enum ReflectMut<'a> {
    Indirect(&'a mut dyn ReflectIndirect),
    Direct(&'a mut dyn ReflectDirect),

    /// Internally signals that the value is not really mutable, for example
    /// we cannot change a reference value field from Interact context.
    Immutable,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AssignError {
    Deser(deser::DeserError),

    /// Some types, having ignored fields, will be unbuildable.
    Unbuildable,

    /// Other values are immutable, such as reference values.
    Immutable,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CallError {
    Deser(deser::DeserError),

    /// Signals the Climber stack to retract into a mutable path so that the
    /// field we are attempting to operate on will be recalled in a mutable
    /// state.
    NeedMutable,

    /// The called function does not exist.
    NoSuchFunction,
}

pub type RetValCallback<'a> = Box<FnMut(&dyn Access, &mut Climber<'a>)>;

/// The `Access` trait, meant to be used as a trait object, provides methods that
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
