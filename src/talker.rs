use std::ffi::{CString, CStr};
use std::os::raw::{c_char};

use crate::dialogue::*;

const CLEAR_T_MAX : f32 = 35.0;
const CLEAR_T_SUCK_MIN : f32 = 31.0;

#[derive(Default, Clone, Debug)]
pub struct Talker
{
    cursor : Option<DialogueCursor>,
    cur_string : CString,

    ////char_rate : f32, 
    t : f32,

    clear_t : f32,
}

impl Talker {
    pub fn queue(&mut self, dialogue : &Dialogue) {
        if let Some(c) = self.cursor.as_ref() {
            if c.dialogue_name_eq(dialogue) {
                // Already queued
                // Reduce clear time
                self.clear_t /= 2.0;
                return;
            }
        }

        self.clear();
        self.cursor = Some(DialogueCursor::new(dialogue));
    }

    pub fn clear(&mut self) {
        self.cursor = None;
        self.cur_string = CString::default();
        self.t = 0.0;
        self.clear_t = 0.0;
    }

    pub fn current_ptr(&self) -> *const c_char {
        self.cur_string.as_ptr()
    }

    pub fn tick(&mut self, dt_norm : f32) {
        if (self.cursor.is_none()) {
            return;
        }

        if (self.clear_t > 0.0) {
            self.clear_t += dt_norm;
            let cur_line = self.cursor.as_ref().unwrap().get();
            let mut line_len = cur_line.len();

            let mut ix = 0.0;

            if (self.clear_t > CLEAR_T_SUCK_MIN) {
                let norm = ((self.clear_t - CLEAR_T_SUCK_MIN) / (CLEAR_T_MAX-CLEAR_T_SUCK_MIN))
                    .clamp(0.0, 1.0);

                ix = line_len as f32 * norm;
            }

            self.cur_string = CString::new(cur_line).unwrap();
            if (ix > 0.0) {
                // HACKYYY just want to mutate a cstring gugh
                let mut tmp_string = CString::default();
                std::mem::swap(&mut tmp_string, &mut self.cur_string);
                for i in 0..(ix as isize) {
                    unsafe {
                        let raw = tmp_string.into_raw();
                        *raw.offset(i) = ' ' as i8;
                        tmp_string = CString::from_raw(raw);
                    }
                }
                std::mem::swap(&mut tmp_string, &mut self.cur_string);
            }

            if (self.clear_t > CLEAR_T_MAX) {
                self.clear();
            }

            return;
        }

        const RATE : f32 = 0.75;
        self.t += dt_norm * RATE;

        while (self.t > 1.0) {
            self.t -= 1.0;
            if (!self.cursor.as_mut().unwrap().incr()) {
                self.clear_t = 1.0;
                break;
            }
        }

        self.cur_string = CString::new(self.cursor.as_ref().unwrap().get()).unwrap();
    }
}