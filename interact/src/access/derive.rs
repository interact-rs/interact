use super::Access;

#[derive(Debug, Eq, PartialEq)]
pub enum StructKind {
    Unit,
    Tuple(usize),
    Fields(&'static [&'static str]),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Struct {
    pub name: &'static str,
    pub kind: StructKind,
}

#[derive(Debug)]
pub struct Enum {
    pub name: &'static str,
    pub opts: &'static [&'static str],
}

pub trait ReflectStruct {
    fn get_desc(&self) -> Struct;
    fn get_field_by_name(&self, name: &'static str) -> Option<&dyn Access>;
    fn get_field_by_idx(&self, idx: usize) -> Option<&dyn Access>;
    fn get_field_by_name_mut(&mut self, name: &'static str) -> Option<&mut dyn Access>;
    fn get_field_by_idx_mut(&mut self, idx: usize) -> Option<&mut dyn Access>;
}

pub trait ReflectEnum {
    fn get_variant_desc(&self) -> Enum;
    fn get_variant_struct(&self) -> &dyn ReflectStruct;
    fn get_variant_struct_mut(&mut self) -> &mut dyn ReflectStruct;
}
