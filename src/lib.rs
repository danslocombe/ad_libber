#![allow(unused_parens)]

pub mod dialogue;
pub mod dialogue_engine;
pub mod interop;
pub mod talker;

use dialogue::Annotation;
use interop::global_state::GlobalState;
use interop::queue_params::QueueParams;
use interop::iter_wrapper::IterWrapper;

#[cfg(feature = "gms")]
pub mod gms {
    use std::os::raw::{c_char};
    use std::ffi::{CStr};

    #[macro_use]
    extern crate gms_binder;

    use gms_binder::*;

    static mut GLOBAL_STATE : Option<GlobalState> = None;

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
            GLOBAL_STATE.as_ref().unwrap().engine.current_ptr()
        }
    }
    */


    #[no_mangle]
    #[gms_bind]
    pub extern "C" fn reset_iterator() -> f64 {
        unsafe {
            let iter = GLOBAL_STATE.as_ref().unwrap().engine.current_string_iter();
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
            GLOBAL_STATE.as_mut().unwrap().engine.tick(1.0);
            0.0
        }
    }

    #[no_mangle]
    #[gms_bind]
    pub extern "C" fn set_text_rate(rate : f64) -> f64 {
        unsafe {
            GLOBAL_STATE.as_mut().unwrap().engine.options.text_rate = rate as f32;
            0.0
        }
    }

    #[no_mangle]
    #[gms_bind]
    pub extern "C" fn set_line_linger_time(time : f64) -> f64 {
        unsafe {
            GLOBAL_STATE.as_mut().unwrap().engine.options.line_linger_time = time as f32;
            0.0
        }
    }

    #[no_mangle]
    #[gms_bind]
    pub extern "C" fn get_current_sprite() -> *const c_char {
        unsafe {
            todo!()
            //GLOBAL_STATE.as_mut().unwrap().engine.
        }
    }

    gms_bind_end!();
}