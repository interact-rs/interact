use crate::deser::Deser;

impl<T> Deser for &[T] where T: Deser {}

impl<T> Deser for &mut [T] where T: Deser {}

// TODO: implement
impl<T> Deser for Vec<T> where T: Deser {}

macro_rules! sized_iter {
    ($t:ty) => {
        // TODO: implement
        impl<T> Deser for $t where T: Deser {}
    };
}

sized_iter!([T; 1]);
sized_iter!([T; 2]);
sized_iter!([T; 3]);
sized_iter!([T; 4]);
sized_iter!([T; 5]);
sized_iter!([T; 6]);
sized_iter!([T; 7]);
sized_iter!([T; 8]);
sized_iter!([T; 9]);
sized_iter!([T; 10]);
sized_iter!([T; 11]);
sized_iter!([T; 12]);
sized_iter!([T; 13]);
sized_iter!([T; 14]);
sized_iter!([T; 15]);
sized_iter!([T; 16]);
sized_iter!([T; 17]);
sized_iter!([T; 18]);
sized_iter!([T; 19]);
sized_iter!([T; 20]);
sized_iter!([T; 21]);
sized_iter!([T; 22]);
sized_iter!([T; 23]);
sized_iter!([T; 24]);
sized_iter!([T; 25]);
sized_iter!([T; 26]);
sized_iter!([T; 27]);
sized_iter!([T; 28]);
sized_iter!([T; 29]);
sized_iter!([T; 30]);
sized_iter!([T; 31]);
sized_iter!([T; 32]);
