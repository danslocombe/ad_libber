use std::ffi::{CString};

use crate::dialogue::{OwnedAnnotatedStringIterator, SpanAnnotation};

#[derive(Default)]
pub struct IterWrapper { 
    inner : OwnedAnnotatedStringIterator,
    pub current_c_string : Option<CString>,
    pub current_annotation : Option<SpanAnnotation>,
}

impl IterWrapper {
    pub fn new(inner : OwnedAnnotatedStringIterator) -> Self {
        IterWrapper {
            inner,
            current_annotation: None,
            current_c_string: None,
        }
    }

    pub fn move_next(&mut self) -> bool {
        if let Some((x, y)) = self.inner.next() {
            self.current_c_string = Some(CString::new(x).unwrap());
            self.current_annotation = Some(y.clone());

            true
        }
        else {
            false
        }
    }
}