use std::sync::Arc;

use crate::access::{Access, ImmutAccess, MutAccess, Reflect, ReflectDirect, ReflectMut};
use crate::climber::{ClimbError, Climber};
use crate::deser::Deser;
use crate::node_tree::NodeTree;
use crate::reflector::Reflector;

pub trait ReflectVec {
    fn get_len(&self) -> usize;
    fn get_item(&self, idx: usize) -> Option<&dyn Access>;
    fn get_item_mut(&mut self, _idx: usize) -> Option<&mut dyn Access>;
}

macro_rules! if_mut {
    (mut, {$t: expr} else {$f:expr}) => {
        $t
    };
    (immut, {$t: expr} else {$f:expr}) => {
        $f
    };
}

macro_rules! sized_iter {
    ($t:ty, $i:ident, $name:expr) => {
        impl<T> ReflectVec for $t
        where
            T: Access,
        {
            fn get_len(&self) -> usize {
                self.len()
            }

            fn get_item(&self, idx: usize) -> Option<&dyn Access> {
                if idx >= self.len() {
                    None
                } else {
                    Some(&self[idx])
                }
            }

            fn get_item_mut(&mut self, _idx: usize) -> Option<&mut dyn Access> {
                if_mut! {
                    $i, {
                        if _idx >= self.len() {
                            None
                        } else {
                            Some(&mut self[_idx])
                        }
                    } else {
                        None
                    }
                }
            }
        }

        impl<T> ReflectDirect for $t
        where
            T: Access,
        {
            fn immut_reflector(&self, reflector: &Arc<Reflector>) -> NodeTree {
                Reflector::reflect_vec(reflector, self, $name)
            }

            fn immut_climber<'a>(
                &self,
                climber: &mut Climber<'a>,
            ) -> Result<Option<NodeTree>, ClimbError> {
                if !climber.open_bracket() {
                    return Ok(None);
                }

                let v = match usize::deser(&mut climber.borrow_tracker()) {
                    Err(e) => Err(e),
                    Ok(i) => Ok(self.get_item(i)),
                };

                let v = match v {
                    Ok(None) => return Err(ClimbError::NotFound),
                    Err(err) => return Err(ClimbError::DeserError(err)),
                    Ok(Some(v)) => v,
                };

                climber.close_bracket()?;

                return climber.general_access_immut(v).map(Some);
            }

            fn mut_climber<'a>(
                &mut self,
                climber: &mut Climber<'a>,
            ) -> Result<Option<NodeTree>, ClimbError> {
                if !climber.open_bracket() {
                    return Ok(None);
                }

                let v = match usize::deser(&mut climber.borrow_tracker()) {
                    Err(e) => Err(e),
                    Ok(i) => Ok(self.get_item_mut(i)),
                };

                let v = match v {
                    Ok(None) => return Err(ClimbError::NotFound),
                    Err(err) => return Err(ClimbError::DeserError(err)),
                    Ok(Some(v)) => v,
                };

                climber.close_bracket()?;

                return climber.general_access_mut(v).map(Some);
            }
        }

        impl<T> Access for $t
        where
            T: Access,
        {
            fn immut_access(&self) -> ImmutAccess {
                ImmutAccess::no_funcs(Reflect::Direct(self))
            }

            fn mut_access(&mut self) -> MutAccess {
                if_mut! {
                    $i, {
                        MutAccess::no_funcs(ReflectMut::Direct(self))
                    } else {
                        MutAccess::no_funcs(ReflectMut::Immutable)
                    }
                }
            }
        }
    };
}

sized_iter!(&[T], immut, "");
sized_iter!(&mut [T], mut, "");
sized_iter!(Vec<T>, mut, "Vec");
sized_iter!([T; 1], mut, "");
sized_iter!([T; 2], mut, "");
sized_iter!([T; 3], mut, "");
sized_iter!([T; 4], mut, "");
sized_iter!([T; 5], mut, "");
sized_iter!([T; 6], mut, "");
sized_iter!([T; 7], mut, "");
sized_iter!([T; 8], mut, "");
sized_iter!([T; 9], mut, "");
sized_iter!([T; 10], mut, "");
sized_iter!([T; 11], mut, "");
sized_iter!([T; 12], mut, "");
sized_iter!([T; 13], mut, "");
sized_iter!([T; 14], mut, "");
sized_iter!([T; 15], mut, "");
sized_iter!([T; 16], mut, "");
sized_iter!([T; 17], mut, "");
sized_iter!([T; 18], mut, "");
sized_iter!([T; 19], mut, "");
sized_iter!([T; 21], mut, "");
sized_iter!([T; 22], mut, "");
sized_iter!([T; 23], mut, "");
sized_iter!([T; 24], mut, "");
sized_iter!([T; 25], mut, "");
sized_iter!([T; 27], mut, "");
sized_iter!([T; 28], mut, "");
sized_iter!([T; 29], mut, "");
sized_iter!([T; 31], mut, "");
sized_iter!([T; 32], mut, "");
