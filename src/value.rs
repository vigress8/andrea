use crate::heap::ObjectPtr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Char(char),
    Integer(i64),
    Word(u64),
    Float(f64),
    ObjectPtr(ObjectPtr),
}

impl Value {
    pub fn get_object_ptr(&self) -> Option<ObjectPtr> {
        if let Self::ObjectPtr(ptr) = self {
            Some(*ptr)
        } else {
            None
        }
    }
}
