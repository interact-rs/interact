use std::cell::RefCell;

use crate::deser::{Deser, Result, Tracker};

impl<T> Deser for RefCell<T>
where
    T: Deser,
{
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        Ok(RefCell::new(T::deser(tracker)?))
    }
}
