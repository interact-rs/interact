pub trait ReflectIter<Item> {
    fn reflect_next(&mut self) -> Option<Item>;
}
