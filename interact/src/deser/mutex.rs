use std::sync::Mutex;

use crate::deser::{Deser, Result, Tracker};

impl<T> Deser for Mutex<T>
where
    T: Deser,
{
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        Ok(Mutex::new(T::deser(tracker)?))
    }
}
