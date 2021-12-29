use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char};

#[derive(Default, Clone, Debug)]
pub struct Line
{
    line : String,
}

#[derive(Default, Clone, Debug)]
pub struct Dialogue
{
    name : String,
    filename : String,
    lines : Vec<Line>,
}

#[derive(Default, Clone, Debug)]
pub struct DialogueFile
{
    sections : Vec<Dialogue>,
}

impl DialogueFile
{
    pub fn parse(p : &str) -> std::io::Result<Self> {
        eprintln!("Parsing: {}", p);
        let mut sections = vec![];
        let mut cur_section : Option<Dialogue> = None;

        let read = std::fs::read_to_string(p)?;
        let lines = read.lines();
        for line in lines {
            if (line.starts_with("[")) {
                cur_section.take().map(|x| {
                    sections.push(x);
                });

                let new_name = &line[1..line.len()-1];
                eprintln!("Read: {}", new_name);

                cur_section = Some(Dialogue {
                    name: String::from(new_name),
                    filename: p.to_owned(),
                    lines : vec![],
                });
            }
            else {
                cur_section.as_mut().map(|x| {
                    x.lines.push(Line{ line: line.to_owned()});
                });
            }
        }

        cur_section.take().map(|x| {
            sections.push(x);
        });
        
        Ok(Self {
            sections,
        })
    }

    pub fn get(&self, section_name : &str) -> Option<&Dialogue> {
        for section in &self.sections {
            if (unicase::eq_ascii(section_name, &section.name)) {
                return Some(section);
            }
        }

        return None;
    }
}

#[derive(Default, Clone, Debug)]
pub struct DialogueCache
{
    cache : HashMap<String, DialogueFile>,
}

impl DialogueCache {
    pub fn preload(&mut self, filename : &str) {
        if (self.cache.contains_key(filename))
        {
            // Already loaded.
        }
        else {
            let dialogue = DialogueFile::parse(filename).unwrap();
            self.cache.insert(filename.to_owned(), dialogue);
        }
    }

    pub fn get(&self, filename : &str) -> &DialogueFile {
        self.cache.get(filename).unwrap()
    }
}

const CLEAR_T_MAX : f32 = 35.0;
const CLEAR_T_SUCK_MIN : f32 = 31.0;

#[derive(Default, Clone, Debug)]
pub struct Talker
{
    dialogue : Option<Dialogue>,
    cur_line_index : usize,
    cur_string : CString,
    ////char_rate : f32, 
    chars : f32,
    clear_t : f32,
    next_line_t : f32,
}

impl Talker {
    pub fn queue(&mut self, dialogue : &Dialogue) {
        if let Some(d) = self.dialogue.as_ref() {
            if unicase::eq_ascii(&d.name, &dialogue.name) && unicase::eq_ascii(&d.filename, &dialogue.filename) {
                // Already queued
                // Reduce clear time
                self.clear_t /= 2.0;
                return;
            }
        }

        self.clear();
        self.dialogue = Some(dialogue.clone());
    }

    pub fn clear(&mut self) {
        self.dialogue = None;
        self.cur_line_index = 0;
        self.chars = 0.0;
        self.cur_string = CString::default();
        self.clear_t = 0.0;
        self.next_line_t = 0.0;
    }

    pub fn current_ptr(&self) -> *const c_char {
        self.cur_string.as_ptr()
    }

    fn get_cur_line(&self) -> &Line {
        &self.dialogue.as_ref().unwrap().lines[self.cur_line_index]
    }

    pub fn tick(&mut self, dt_norm : f32) {
        if (self.dialogue.is_none()) {
            return;
        }

        loop {
            if (self.clear_t > 0.0)
            {
                self.clear_t += dt_norm;
                let cur_line = self.get_cur_line().clone();
                let line_len = cur_line.line.len();
                self.cur_string = CString::new(cur_line.line).unwrap();

                let mut ix = 0.0;

                if (self.clear_t > CLEAR_T_SUCK_MIN) {
                    let norm = ((self.clear_t - CLEAR_T_SUCK_MIN) / (CLEAR_T_MAX-CLEAR_T_SUCK_MIN))
                        .clamp(0.0, 1.0);

                    ix = line_len as f32 * norm;
                }

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
            self.chars += dt_norm * RATE;

            let cur_line = &self.dialogue.as_ref().unwrap().lines[self.cur_line_index];
            if (self.chars < cur_line.line.len() as f32)
            {
                let substr = &(cur_line.line)[0..=self.chars.floor() as usize];
                self.cur_string = CString::new(substr).unwrap();
                return;
            }
            else
            {
                let lines_len = self.dialogue.as_ref().unwrap().lines.len();

                if (self.cur_line_index + 1 < lines_len) {
                    const NEXT_LINE_T_MAX : f32 = 15.0;
                    if (self.next_line_t > NEXT_LINE_T_MAX) {
                        self.cur_line_index += 1;
                        self.next_line_t = 0.0;
                        self.chars = 0.0;
                    }
                    else {
                        self.next_line_t += 1.0;
                        return;
                    }
                }
                else {
                    self.chars = cur_line.line.len() as f32;
                    self.clear_t = 1.0;
                }
            }
        } 
    }
}