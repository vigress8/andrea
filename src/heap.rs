use crate::value::Value;
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

const HEAP_THRESHOLD: usize = 1024;
#[derive(Debug, Clone, PartialEq)]
pub struct Heap {
    head: *mut HeapObject,
    size: usize,
    threshold: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeapObject {
    pub next: *mut Self,
    pub color: Cell<Color>,
    pub data: Object,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub tag: u8,
    pub fields: Vec<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectPtr(pub NonNull<HeapObject>);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Color {
    #[default]
    Unmarked,
    Reachable,
}

impl Heap {
    pub fn new() -> Self {
        Default::default()
    }

    pub const fn is_full(&self) -> bool {
        self.size >= self.threshold
    }

    pub fn new_object(&mut self, obj: Object) -> ObjectPtr {
        if self.is_full() {
            self.sweep();
        }

        let obj = HeapObject::new(self.head, obj);

        let ptr = Box::into_raw(Box::new(obj));
        self.head = ptr;
        self.size += 1;

        ObjectPtr(NonNull::new(ptr).unwrap())
    }

    pub fn sweep(&mut self) {
        let mut ptr = &mut self.head;
        while let Some(obj) = unsafe { ptr.as_mut() } {
            if obj.reachable() {
                obj.unmark();
                ptr = &mut obj.next;
            } else {
                *ptr = unsafe { Box::from_raw(*ptr) }.next;
                self.size -= 1;
            }
        }

        self.threshold = self.size * 2;
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self {
            head: ptr::null_mut(),
            size: 0,
            threshold: HEAP_THRESHOLD,
        }
    }
}

impl HeapObject {
    pub fn new(next: *mut Self, data: Object) -> Self {
        Self {
            next,
            color: Cell::new(Color::default()),
            data,
        }
    }

    pub fn reachable(&self) -> bool {
        self.color.get() == Color::Reachable
    }

    pub fn mark(&self) {
        self.color.set(Color::Reachable);

        for field in &self.data.fields {
            if let Some(ptr) = field.get_object_ptr() {
                ptr.mark();
            }
        }
    }

    pub fn unmark(&self) {
        self.color.set(Color::Unmarked);
    }
}

impl ObjectPtr {
    pub fn new(obj: HeapObject) -> Self {
        Self(NonNull::from(&obj))
    }
}

impl Deref for ObjectPtr {
    type Target = HeapObject;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl DerefMut for ObjectPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}
