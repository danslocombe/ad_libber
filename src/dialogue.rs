use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum Command {
    Wait(u32),
    Clear,
}

impl Command {
    pub fn tick_len(&self) -> u32 {
        match *self {
            Command::Wait(delay) => delay,
            Command::Clear => 0,
        }
    }

    pub fn parse(s : &str) -> Option<Self> {
        let mut splits = s.split_ascii_whitespace();
        let command = splits.next()?;

        if (unicase::eq_ascii(command, "clear") || unicase::eq_ascii(command, "c")) {
            Some(Self::Clear)
        }
        else if (unicase::eq_ascii(command, "wait") || unicase::eq_ascii(command, "w")) {

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
        else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub enum Chunk {
    Line(String),
    Command(Command),
}

impl Chunk {
    pub fn tick_len(&self) -> u32 {
        match self {
            Chunk::Line(s) => s.len() as u32,
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
                Chunk::Line(err.to_owned()),
            ],
        }
    }
    fn name_eq(&self, other: &Self) -> bool {
        unicase::eq_ascii(&self.name, &other.name) && unicase::eq_ascii(&self.filename, &other.filename)
    }

    fn empty(&self) -> bool {
        for chunk in &self.chunks {
            if let Chunk::Line(l) = chunk {
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

                let new_name = &line[1..line.len()-1];
                eprintln!("Read Section: {}", new_name);

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
                    x.chunks.push(Chunk::Line(line.to_owned()));
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

    pub fn get(&self) -> String {
        let mut s = String::default();
        for i in self.start..=self.end {
            if let Chunk::Line(line) = &self.dialogue.chunks[i] {
                if (s.len() > 0) {
                    s.push('#');
                }
                if i < self.end {
                    s.push_str(line);
                }
                else {
                    s.push_str(&line[0..self.line_i.min(line.len())]);
                }
            }
        }

        //println!("start {} end {} '{}'", self.start, self.end, s);
        s
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