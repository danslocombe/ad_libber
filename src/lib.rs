#![allow(unused_parens)]

//use std::os::raw::c_char;
//use std::ffi::{CString};
#[macro_use]
extern crate gms_binder;
use gms_binder::*;

mod dialogue;
mod talker;

use std::time::Instant;
use std::os::raw::{c_char};
use std::ffi::{CString, CStr};
use dialogue::*;
use std::collections::HashSet;

static mut GLOBAL_STATE : Option<GlobalState> = None;

struct QueueParams<'a>
{
    filename: &'a str,
    section: &'a str,
    oneshot: bool,
}

impl<'a> QueueParams<'a> {
    pub fn parse(input : &'a str) -> Option<Self> {
        let mut splits = input.split("|").map(|x| x);
        let filename = splits.next()?;
        let section = splits.next()?;
        let mut oneshot = false;

        while let Some(arg) = splits.next() {
            if (unicase::eq_ascii(arg, "oneshot")) {
                oneshot = true;
            }
        }

        Some(Self {
            filename,
            section,
            oneshot,
        })
    }
}

#[derive(Debug)]
struct FilenameSectionPair
{
    filename: String,
    section : String,
}

impl PartialEq for FilenameSectionPair {
    fn eq(&self, other : &Self) -> bool {
        unicase::eq_ascii(&self.filename, &other.filename) && 
            unicase::eq_ascii(&self.section, &other.section)
    }
}

impl Eq for FilenameSectionPair {}

impl std::hash::Hash for FilenameSectionPair {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.filename.to_ascii_lowercase().hash(state);
        self.section.to_ascii_lowercase().hash(state);
    }
}

#[derive(Default)]
struct GlobalState
{
    path : String,
    talker : talker::Talker,
    cache : DialogueCache,
    one_shot_cache : HashSet<FilenameSectionPair>,
    iter_wrapper : Option<IterWrapper>,
}

impl GlobalState
{
    fn full_filename(&self, filename : &str) -> String {
        self.path.clone() + filename + ".adlib"
    }

    fn preload(&mut self, filename : &str) {
        self.cache.preload(&self.full_filename(filename));
    }

}

impl<'a> GlobalState
{
    fn queue(&mut self, queue_args: QueueParams<'a>) {
        self.preload(queue_args.filename);

        if (queue_args.oneshot) {
            let cache_key = FilenameSectionPair  {
                filename: queue_args.filename.to_owned(),
                section: queue_args.section.to_owned(),
            };

            if (!self.one_shot_cache.insert(cache_key))
            {
                return;
            }
        }

        if let Some(dialogue_file) = self.cache.get(&self.full_filename(queue_args.filename)) {
            if let Some(dialogue) = dialogue_file.get(queue_args.section) {
                self.talker.queue(dialogue);
            }
            else {
                self.talker.queue(&Dialogue::from_error(&format!("Could not find dialogue section {}", queue_args.section)));
            }
        }
        else {
            self.talker.queue(&Dialogue::from_error(&format!("Could not find dialogue file {}", queue_args.filename)));
        }
        //dialogue.get_section()
    }
}

gms_bind_start!("ad_libber", "ad_libber.dll", "ad_lib");

#[no_mangle]
#[gms_bind]
pub extern "C" fn reset() -> f64 {
    unsafe {
        GLOBAL_STATE = Some(GlobalState::default());
    }
    0.0
}

#[no_mangle]
#[gms_bind]
pub extern "C" fn set_base_path(path_raw : *const c_char) -> f64 {
    unsafe {
        let path = CStr::from_ptr(path_raw).to_str().unwrap();
        GLOBAL_STATE.as_mut().unwrap().path = path.to_owned();
    }
    0.0
}

#[no_mangle]
#[gms_bind]
pub extern "C" fn preload(filename_raw: *const c_char) -> f64 {
    unsafe {
        let state = GLOBAL_STATE.as_mut().unwrap();
        let input = CStr::from_ptr(filename_raw).to_str().unwrap();
        println!("Calling preload {}", input);
        state.preload(input);
        0.0
    }
}

#[no_mangle]
#[gms_bind]
pub extern "C" fn queue_dialogue(input_raw: *const c_char) -> f64 {
    unsafe {
        // input of the form "filename|section"
        let state = GLOBAL_STATE.as_mut().unwrap();
        let input = CStr::from_ptr(input_raw).to_str().unwrap();
        let queue_args = QueueParams::parse(input).expect(&format!("Could not parse {} as queue input", input));
        state.queue(queue_args);
        0.0
    }
}

/*
#[no_mangle]
#[gms_bind]
pub extern "C" fn get_string() -> *const c_char {
    unsafe {
        GLOBAL_STATE.as_ref().unwrap().talker.current_ptr()
    }
}
*/

struct IterWrapper { 
    inner : OwnedAnnotatedStringIterator,
    current_c_string : Option<CString>,
    current_annotation : Option<SpanAnnotation>,
}

impl IterWrapper {
    fn new(inner : OwnedAnnotatedStringIterator) -> Self {
        IterWrapper {
            inner,
            current_annotation: None,
            current_c_string: None,
        }
    }

    fn move_next(&mut self) -> bool {
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

#[no_mangle]
#[gms_bind]
pub extern "C" fn reset_iterator() -> f64 {
    unsafe {
        let iter = GLOBAL_STATE.as_ref().unwrap().talker.current_string_iter();
        GLOBAL_STATE.as_mut().unwrap().iter_wrapper = Some(IterWrapper::new(iter));
        0.0
    }
}

#[no_mangle]
#[gms_bind]
pub extern "C" fn move_iterator() -> f64 {
    unsafe {
        let iter = GLOBAL_STATE.as_mut().unwrap().iter_wrapper.as_mut().unwrap();
        if (iter.move_next()) {
            1.0
        }
        else {
            0.0
        }
    }
}

#[no_mangle]
#[gms_bind]
pub extern "C" fn iterator_cur_string() -> *const c_char {
    unsafe {
        let iter = GLOBAL_STATE.as_ref().unwrap().iter_wrapper.as_ref().unwrap();
        let ptr = iter.current_c_string.as_ref().unwrap().as_ptr();
        ptr
    }
}

#[no_mangle]
#[gms_bind]
pub extern "C" fn is_jiggly() -> f64 {
    unsafe {
        let iter = GLOBAL_STATE.as_ref().unwrap().iter_wrapper.as_ref().unwrap();
        let annotations = &iter.current_annotation.as_ref().unwrap().annotations;
        if (annotations.iter().any(|x| *x == Annotation::Jiggly)) {
            1.0
        }
        else {
            0.0
        }
    }
}

#[no_mangle]
#[gms_bind]
pub extern "C" fn is_wide() -> f64 {
    unsafe {
        let iter = GLOBAL_STATE.as_ref().unwrap().iter_wrapper.as_ref().unwrap();
        let annotations = &iter.current_annotation.as_ref().unwrap().annotations;
        if (annotations.iter().any(|x| *x == Annotation::Wide)) {
            1.0
        }
        else {
            0.0
        }
    }
}


#[no_mangle]
#[gms_bind]
pub extern "C" fn tick() -> f64 {
    unsafe {
        GLOBAL_STATE.as_mut().unwrap().talker.tick(1.0);
        0.0
    }
}

gms_bind_end!();