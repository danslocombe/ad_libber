use std::collections::HashMap;
use std::str::Lines;
use std::str::FromStr;

use crate::talker::Talker;

#[derive(Clone, Debug)]
pub enum Command {
    AnnotationStart(Annotation),
    AnnotationEnd(Annotation),
    Speaker(String),
    Wait(u32),
    Clear,
}

fn is_command(input : &str, name : &str) -> bool {
    unicase::eq_ascii(input, name) || unicase::eq_ascii(input, &name[0..1])
}

fn is_end_command(input : &str, name : &str) -> bool {
    unicase::eq_ascii(input, &("/".to_owned() + name)) || unicase::eq_ascii(input, &("/".to_owned() + &name[0..1]))
}

impl Command {
    pub fn tick_len(&self) -> u32 {
        match *self {
            Command::Wait(delay) => delay,
            _ => 0,
        }
    }

    pub fn parse(s : &str) -> Option<Self> {
        let mut splits = s.split_ascii_whitespace();
        let command = splits.next()?;

        if (is_command(command, "clear")) {
            Some(Self::Clear)
        }
        else if (is_command(command, "wait")) {

            let dur : u32 = if let Some(t) = splits.next() {
                let mut mult = 1.0;
                let mut parse_t = t;
                if (t.ends_with("ms")) {
                    parse_t = &t[0..(t.len()-2)];
                    mult = 60.0 / 1000.0;
                }
                else if (t.ends_with("s")) {
                    parse_t = &t[0..(t.len()-1)];
                    mult = 60.0;
                }

                let f = mult * f32::from_str(parse_t).expect(&format!("Failed to parse duration from wait command {}", parse_t));
                f.round() as u32
            }
            else {
                60
            };

            Some(Self::Wait(dur))
        }
        else if (is_command(command, "speaker")) {
            Some(Self::Speaker(splits.next().unwrap().to_owned()))
        }
        else if (is_command(command, "jiggle")) {
            Some(Self::AnnotationStart(Annotation::Jiggly))
        }
        else if (is_end_command(command, "jiggle")) {
            Some(Self::AnnotationEnd(Annotation::Jiggly))
        }
        else if (is_command(command, "wide")) {
            Some(Self::AnnotationStart(Annotation::Wide))
        }
        else if (is_end_command(command, "wide")) {
            Some(Self::AnnotationEnd(Annotation::Wide))
        }
        else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum Chunk {
    Text(String),
    Newline,
    Command(Command),
}

impl Chunk {
    pub fn tick_len(&self) -> u32 {
        match self {
            Chunk::Text(s) => s.len() as u32,
            Chunk::Newline => 1,
            Chunk::Command(c) => c.tick_len(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Dialogue
{
    pub name : String,
    pub filename : String,
    pub chunks : Vec<Chunk>,
}

impl Dialogue {
    pub fn from_error(err : &str) -> Self {
        Self {
            name : "error".to_owned(),
            filename : "error".to_owned(),
            chunks : vec![
                Chunk::Text(err.to_owned()),
            ],
        }
    }
    fn name_eq(&self, other: &Self) -> bool {
        unicase::eq_ascii(&self.name, &other.name) && unicase::eq_ascii(&self.filename, &other.filename)
    }

    fn empty(&self) -> bool {
        for chunk in &self.chunks {
            if let Chunk::Text(l) = chunk {
                if (l.len() > 0) {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Default, Clone, Debug)]
pub struct DialogueFile
{
    talkers : Vec<Talker>,
    sections : Vec<Dialogue>,
}

impl<'a> DialogueFile {
    fn parse_talker(lines : &mut Lines<'a>) -> (Option<&'a str>, Talker) {
        let mut talker = Talker::default();

        for line in lines {
            if (line.is_empty() || line.starts_with("#")) {
                continue;
            }

            if let Some((field, value)) = line.split_once('=') {
                if unicase::eq_ascii(field, "sprite") {
                    talker.sprite = value.trim().to_owned();
                }
                else if unicase::eq_ascii(field, "sound") {
                    talker.sound = value.trim().to_owned();
                }
                else if unicase::eq_ascii(field, "rate") {
                    talker.rate = Some(value.parse::<f32>().unwrap());
                }
            }

            if (line.starts_with("[")) {
                return (Some(line), talker);
            }
        }

        (None, talker)
    }

    fn parse_section(lines : &mut Lines<'a>) -> (Option<&'a str>, Dialogue) {
        for line in lines {
            if (line.is_empty() || line.starts_with("#")) {
                continue;
            }

            if (line.starts_with("[")) {
                cur_section.take().map(|x| {
                    if (!x.empty())
                    {
                        sections.push(x);
                    }
                });

                let section_name = &line[1..line.len()-1];

                if let Some((keyword, name)) = section_name.split_once(" ") {
                    if (unicase::eq_ascii(keyword, "talker") {
                        
                    }
                }
                else {

                }

                eprintln!("Read Section: {}", section_name);

                cur_section = Some(Dialogue {
                    name: String::from(new_name),
                    filename: p.to_owned(),
                    chunks : vec![],
                });
            }
            else if (line.starts_with("(")) {
                let command = Command::parse(&line[1..(line.len() - 1)]).expect(&format!("Could not parse command {}", line));
                eprintln!("parsed command: {:?}", command);
                cur_section.as_mut().map(|x| {
                    x.chunks.push(Chunk::Command(command));
                });
            }
            else {
                cur_section.as_mut().map(|x| {
                    let mut splits = line.split_ascii_whitespace();
                    let mut cur_str = String::new();
                    while let Some(split) = splits.next() {
                        if (split.starts_with("(")) {
                            let command = Command::parse(&split[1..(split.len() - 1)]).expect(&format!("Could not parse command in line '{}' '{}'", line, split));
                            x.chunks.push(Chunk::Text(cur_str));
                            x.chunks.push(Chunk::Command(command));
                            cur_str = String::new();
                        }
                        else {
                            if (cur_str.len() > 0) {
                                cur_str.push(' ');
                            }
                            cur_str.push_str(split);
                        }
                    }
                    x.chunks.push(Chunk::Text(cur_str.to_owned()));
                    x.chunks.push(Chunk::Newline);
                });
            }
    }
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
            if (line.is_empty() || line.starts_with("#")) {
                continue;
            }

            if (line.starts_with("[")) {
                cur_section.take().map(|x| {
                    if (!x.empty())
                    {
                        sections.push(x);
                    }
                });

                let section_name = &line[1..line.len()-1];

                if let Some((keyword, name)) = section_name.split_once(" ") {
                    if (unicase::eq_ascii(keyword, "talker") {
                        
                    }
                }
                else {

                }

                eprintln!("Read Section: {}", section_name);

                cur_section = Some(Dialogue {
                    name: String::from(new_name),
                    filename: p.to_owned(),
                    chunks : vec![],
                });
            }
            else if (line.starts_with("(")) {
                let command = Command::parse(&line[1..(line.len() - 1)]).expect(&format!("Could not parse command {}", line));
                eprintln!("parsed command: {:?}", command);
                cur_section.as_mut().map(|x| {
                    x.chunks.push(Chunk::Command(command));
                });
            }
            else {
                cur_section.as_mut().map(|x| {
                    let mut splits = line.split_ascii_whitespace();
                    let mut cur_str = String::new();
                    while let Some(split) = splits.next() {
                        if (split.starts_with("(")) {
                            let command = Command::parse(&split[1..(split.len() - 1)]).expect(&format!("Could not parse command in line '{}' '{}'", line, split));
                            x.chunks.push(Chunk::Text(cur_str));
                            x.chunks.push(Chunk::Command(command));
                            cur_str = String::new();
                        }
                        else {
                            if (cur_str.len() > 0) {
                                cur_str.push(' ');
                            }
                            cur_str.push_str(split);
                        }
                    }
                    x.chunks.push(Chunk::Text(cur_str.to_owned()));
                    x.chunks.push(Chunk::Newline);
                });
            }
        }

        cur_section.take().map(|x| {
            if (!x.empty())
            {
                sections.push(x);
            }
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

    pub fn get(&self, filename : &str) -> Option<&DialogueFile> {
        self.cache.get(filename)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Annotation {
    Jiggly,
    Wide,
}

#[derive(Clone, Debug)]
pub struct SpanAnnotation
{
    start : usize,
    end : usize,
    pub annotations : Vec<Annotation>,
}

#[derive(Default, Clone, Debug)]
pub struct AnnotatedString
{
    pub string : String,
    pub annotations : Vec<SpanAnnotation>,
}

impl<'a> AnnotatedString {
    pub fn iter(&'a self) -> AnnotatedStringIterator<'a> {
        AnnotatedStringIterator {
            annotated : self,
            i: 0,
        }
    }
}

impl AnnotatedString {
    pub fn owned_iter(self) -> OwnedAnnotatedStringIterator {
        OwnedAnnotatedStringIterator {
            annotated : self,
            i: 0,
        }
    }
}

pub struct AnnotatedStringIterator<'a> {
    annotated : &'a AnnotatedString,
    i : usize,
}

impl<'a> AnnotatedStringIterator<'a> {
    pub fn next(&mut self) -> Option<(&str, &SpanAnnotation)> {
        if self.i < self.annotated.annotations.len() {
            let x = &self.annotated.annotations[self.i];

            let substring = &self.annotated.string[x.start..x.end];
            let annotations = &self.annotated.annotations[self.i];

            self.i += 1;

            Some((substring, annotations))
        }
        else {
            None
        }
    }
}

#[derive(Default)]
pub struct OwnedAnnotatedStringIterator {
    annotated : AnnotatedString,
    i : usize,
}

impl OwnedAnnotatedStringIterator {
    pub fn next(&mut self) -> Option<(&str, &SpanAnnotation)> {
        if self.i < self.annotated.annotations.len() {
            let x = &self.annotated.annotations[self.i];

            let substring = &self.annotated.string[x.start..x.end];
            let annotations = &self.annotated.annotations[self.i];

            self.i += 1;

            Some((substring, annotations))
        }
        else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct DialogueCursor
{
    dialogue : Dialogue,
    start : usize,
    end : usize,
    line_i : usize,
    exhausted : bool,
}

impl DialogueCursor {
    pub fn new(dialogue : &Dialogue) -> Self {
        Self {
            dialogue : dialogue.clone(),
            start : 0,
            end : 0,
            line_i : 0,
            exhausted: false,
        }
    }

    pub fn dialogue_name_eq(&self, other: &Dialogue) -> bool {
        self.dialogue.name_eq(other)
    }

    pub fn get(&self) -> AnnotatedString {
        let mut s = String::default();
        let mut span_annotations = vec![];
        let mut annotations : Vec<Annotation> = vec![];
        let mut start = 0;
        for i in self.start..=self.end {
            match &self.dialogue.chunks[i] {
                Chunk::Newline => {
                    s.push('#');
                },
                Chunk::Text(text) => {
                    if i < self.end {
                        s.push_str(text);
                    }
                    else {
                        s.push_str(&text[0..self.line_i.min(text.len())]);
                    }
                },
                Chunk::Command(command) => {
                    span_annotations.push(SpanAnnotation {
                        start,
                        end : s.len(),
                        annotations: annotations.clone(),
                    });

                    match command {
                        Command::AnnotationStart(an) => {
                            annotations.push(*an)
                        },
                        Command::AnnotationEnd(an) => {
                            annotations = annotations.into_iter().filter(|x| *x != *an).collect();
                        },
                        _ => {},
                    }

                    if (s.len() > 0) {
                        s.push(' ');
                    }

                    start = s.len();
                }
            }
        }

        span_annotations.push(SpanAnnotation {
            start, 
            end : s.len(),
            annotations: annotations.clone(),
        });

        AnnotatedString {
            string: s,
            annotations : span_annotations, 
        }
    }

    pub fn incr(&mut self) -> bool {
        if (self.exhausted) {
            false
        }
        else {
            //println!("incr {} {}", self.end, self.line_i);
            if (self.line_i >= self.dialogue.chunks[self.end].tick_len() as usize) {

                if (self.end + 1 >= self.dialogue.chunks.len()) {
                    self.exhausted = true;
                    return false;
                }

                self.end += 1;
                self.line_i = 0;

                if let Chunk::Command(Command::Clear) = self.dialogue.chunks[self.end] {
                    self.start = self.end;
                }
            }
            else {
                self.line_i += 1;
            }

            true
        }
    }
}