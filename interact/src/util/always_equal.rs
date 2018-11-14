#[derive(Debug)]
pub struct AlwaysEqual<T>(pub T);

impl<T> Eq for AlwaysEqual<T> {}

impl<T> PartialEq for AlwaysEqual<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
