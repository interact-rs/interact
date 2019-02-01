/// Cover private Mutex derive
///
/// This module checks that we can manually derive `Access` and `Deser` for types for which we
/// cannot use #[derive(Interact)]
///
use interact::access::{Access, ReflectDirect};
use interact::climber::{ClimbError, Climber};
use interact::deser::{self, Tracker};
use interact::{Deser, NodeTree, Reflector};

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

pub struct PseudoMutex<T> {
    _t: T,
}

impl<T> PseudoMutex<T> {
    pub fn new(_t: T) -> Self {
        PseudoMutex { _t }
    }
}

struct Guard<'a, T>(&'a T);

impl<T> PseudoMutex<T> {
    fn lock(&self) -> Guard<T> {
        Guard(&self._t)
    }
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

impl<'a, T> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        panic!("cannot get mutable refs for PseudoMutex");
    }
}

impl<T> ReflectDirect for PseudoMutex<T>
where
    T: Access,
{
    fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
        let locked = self.lock();
        Reflector::reflect(reflector, &*locked)
    }

    fn immut_climber<'a>(&self, climber: &mut Climber<'a>) -> Result<Option<NodeTree>, ClimbError> {
        let save = climber.clone();
        let retval = {
            let locked = self.lock();
            climber.general_access_immut(&*locked).map(Some)
        };

        if let Err(ClimbError::NeedMutPath) = &retval {
            *climber = save;
            let mut locked = self.lock();
            climber.general_access_mut(&mut *locked).map(Some)
        } else {
            retval
        }
    }

    fn mut_climber<'a>(
        &mut self,
        climber: &mut Climber<'a>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        let mut locked = self.lock();
        climber.general_access_mut(&mut *locked).map(Some)
    }
}

impl<T> Deser for PseudoMutex<T>
where
    T: Deser,
{
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> deser::Result<Self> {
        Ok(PseudoMutex::new(T::deser(tracker)?))
    }
}

use interact::derive_interact_extern_opqaue;
derive_interact_extern_opqaue! {
    struct PseudoMutex<T>;
}
