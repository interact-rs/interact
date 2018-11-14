use crate::deser::{Deser, Result, Tracker};
use std::rc::Rc;
use std::sync::Arc;

impl<'a, T: 'a> Deser for &'a T where T: Deser {}

impl<'a, T: 'a> Deser for &'a mut T where T: Deser {}

impl<T> Deser for Box<T>
where
    T: Deser,
{
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        Ok(Box::new(T::deser(tracker)?))
    }
}

impl<T> Deser for Rc<T>
where
    T: Deser,
{
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        Ok(Rc::new(T::deser(tracker)?))
    }
}

impl<T> Deser for Arc<T>
where
    T: Deser,
{
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        Ok(Arc::new(T::deser(tracker)?))
    }
}
