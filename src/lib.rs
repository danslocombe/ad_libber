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

static mut GLOBAL_STATE : Option<GlobalState> = None;

#[derive(Default)]
struct GlobalState
{
    path : String,
    talker : talker::Talker,
    cache : DialogueCache,
}

impl GlobalState
{
    fn full_filename(&self, filename : &str) -> String {
        self.path.clone() + filename
    }

    fn preload(&mut self, filename : &str) {
        self.cache.preload(&self.full_filename(filename));
    }

    fn queue(&mut self, filename : &str, section : &str) {
        self.preload(filename);

        // Assume in cache
        let dialogue_file = self.cache.get(&self.full_filename(filename));
        self.talker.queue(dialogue_file.get(section).unwrap())
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
        let splits : Vec<String> = input.split("|").map(|x| x.to_owned()).collect();
        let filename = String::from(&splits[0]) + ".adlib";
        state.queue(&filename, &splits[1]);
        0.0
    }
}

#[no_mangle]
#[gms_bind]
pub extern "C" fn get_string() -> *const c_char {
    unsafe {
        GLOBAL_STATE.as_ref().unwrap().talker.current_ptr()
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