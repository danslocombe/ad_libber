use std::collections::HashMap;
use std::str::FromStr;

use crate::talker::Talker;

#[derive(Clone, Debug)]
pub enum Command {
    //AnnotationStart(Annotation),
    //AnnotationEnd(Annotation),
    Speaker(String),
    Wait(u32),
    Clear,
    SetStyling(Styling),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Styling {
    pub wavy: bool,
    pub jitter: bool,
    pub wide: bool,
    //rainbow: bool,
    //color: 
    //font_type: 
}

impl Default for Styling {
    fn default() -> Self {
        Styling {
            wavy: false,
            jitter: false,
            wide: false,
        }
    }
}

fn is_command(input : &str, name : &str) -> bool {
    unicase::eq_ascii(input, name)// || unicase::eq_ascii(input, &name[0..1])
}

fn is_end_command(input : &str, name : &str) -> bool {
    unicase::eq_ascii(input, &("/".to_owned() + name))// || unicase::eq_ascii(input, &("/".to_owned() + &name[0..1]))
}

impl Command {
    pub fn tick_len(&self) -> u32 {
        match *self {
            Command::Wait(delay) => delay,
            _ => 0,
        }
    }

    pub fn try_parse(command_full : &str) -> Option<Self> {
        let mut splits = command_full.split_ascii_whitespace();
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
            Some(Self::Speaker(splits.next().expect("Could not parse speaker").to_owned()))
        }
        else if (is_command(command, "style")) {
            let mut style = Styling::default();
            for split in splits {
                if (unicase::eq_ascii("wavy", split)) {
                    style.wavy = true;
                }
                else if (unicase::eq_ascii("jitter", split)) {
                    style.jitter = true;
                }
                else if (unicase::eq_ascii("wide", split)) {
                    style.wide = true;
                }
            }
            Some(Self::SetStyling(style))
        }
        else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextChunk {
    text: String,
    //#[allow(unused)]
    //talker_id : Option<u32>,
}

#[derive(Clone, Debug)]
pub enum Chunk {
    Text(TextChunk),
    Newline,
    Command(Command),
}

impl Chunk {
    pub fn tick_len(&self) -> u32 {
        match self {
            Chunk::Text(s) => s.text.len() as u32,
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
                Chunk::Text(TextChunk{ text: err.to_owned()}),
            ],
        }
    }
    fn name_eq(&self, other: &Self) -> bool {
        unicase::eq_ascii(&self.name, &other.name) && unicase::eq_ascii(&self.filename, &other.filename)
    }

    /*
    fn empty(&self) -> bool {
        for chunk in &self.chunks {
            if let Chunk::Text(l) = chunk {
                if (l.text.len() > 0) {
                    return false;
                }
            }
        }

        true
    }
    */
}

#[derive(Default, Clone, Debug)]
pub struct DialogueFile
{
    //pub talkers : Vec<Talker>,
    pub sections : Vec<Dialogue>,
}

impl<'a> DialogueFile {
    /*
    fn parse_talker(name : &str, lines : &[&'a str], i : &mut usize) -> Talker {
        let mut talker = Talker {
            name : name.to_owned(),
            ..Default::default()
        };

        while *i < lines.len() {
            let line = lines[*i];
            if (line.is_empty() || line.starts_with("#")) {
                *i += 1;
                continue;
            }

            if (line.starts_with("[")) {
                break;
            }

            if let Some((field, value)) = line.split_once('=') {
                if unicase::eq_ascii(field, "sprite") {
                    talker.sprite = value.trim().to_owned();
                }
                else if unicase::eq_ascii(field, "sound") {
                    talker.sound = value.trim().to_owned();
                }
                else if unicase::eq_ascii(field, "rate") {
                    talker.rate = Some(value.parse::<f32>().expect("Could not parse rate"));
                }
            }

            *i += 1;
        }

        talker
    }
    */

    fn parse_section(filename : &str, name : &str, lines : &[&'a str], i : &mut usize) -> Dialogue {
        let mut section = Dialogue { name : name.to_owned(), filename : filename.to_owned(), chunks: Default::default() };

        while *i < lines.len() {
            let line = lines[*i];
            if (line.starts_with("#")) {
                *i += 1;
                continue;
            }

            if (line.starts_with("[")) {
                break;
            }

            *i += 1;

            let mut added_text = false;

            let mut l_i: usize = 0;

            while let Some(open_pos_local) = (line[l_i..].find('(')) {
                let open_pos = open_pos_local + l_i;
                let mut end_pos = line.len();

                if let Some(close_pos_local) = line[open_pos..].find(')') {
                    end_pos = close_pos_local + open_pos;
                }

                if (end_pos - open_pos > 0) {
                    let command_str = &line[open_pos + 1 .. (end_pos)];

                    if let Some(command) = Command::try_parse(command_str) {
                        let ret = Self::make_add_text_chunk(&line[l_i..open_pos], &mut section.chunks);
                        added_text |= ret;
                        section.chunks.push(Chunk::Command(command));
                    }
                }

                l_i = end_pos + 1;
            }

            let ret = Self::make_add_text_chunk(&line[l_i..], &mut section.chunks);
            added_text |= ret;

            if (added_text) {
                // Only add a newline if one of the above actually added text to be displayed.
                // This handles the case where we have a line that is entirely a command.
                section.chunks.push(Chunk::Newline);
            }
        }

        section
    }

    fn make_add_text_chunk(text: &str, chunks: &mut Vec<Chunk>) -> bool {
        let normalize_whitespace = true;
        let s = if (normalize_whitespace) {
            text.trim()
         } else {
            text
         };

        if (s.len() == 0) {
            return false;
        }

        if (normalize_whitespace) {
            let mut has_a_previous_text_chunk_since_linebreak = false;
            for c in chunks.iter() {
                if let Chunk::Text(tc) = c {
                    has_a_previous_text_chunk_since_linebreak = true;
                }

                if let Chunk::Newline = c {
                    has_a_previous_text_chunk_since_linebreak = false;
                }
            }

            if (has_a_previous_text_chunk_since_linebreak) {
                chunks.push(Chunk::Text(TextChunk {
                    text: " ".to_owned(),
                }));
            }
        }

        chunks.push(Chunk::Text(TextChunk {
            text: s.to_owned(),
        }));

        return true;
    }
}


impl DialogueFile
{
    pub fn parse(p : &str) -> std::io::Result<Self> {
        let contents = std::fs::read_to_string(p)?;
        Ok(Self::parse_contents(p, &contents))
    }

    pub fn parse_contents(filename : &str, contents : &str) -> Self {
        eprintln!("Parsing: {}", filename);
        let mut sections = vec![];
        let lines = contents.lines().collect::<Vec<_>>();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            if (line.is_empty() || line.starts_with("#")) {
                i += 1;
                continue;
            }

            if (line.starts_with("[")) {
                let section_name = &line[1..line.len()-1];

                i += 1;

                /*
                if let Some((keyword, name)) = section_name.split_once(" ") {
                    if (unicase::eq_ascii(keyword, "talker")) {
                        eprintln!("Read talker: {}", name);
                        talkers.push(Self::parse_talker(name, &lines, &mut i));
                        eprintln!("{:?}", talkers.last());
                        continue;
                    }
                }
                */

                eprintln!("Read Section: {}", section_name);
                let section = Self::parse_section(filename, section_name, &lines, &mut i);
                eprintln!("{:?}", section);
                sections.push(section);
            }
            else {
                i += 1;
            }
        }

        Self {
            sections,
        }
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
            let dialogue = DialogueFile::parse(filename).expect("Could not parse file in DialogueCache");
            self.cache.insert(filename.to_owned(), dialogue);
        }
    }

    pub fn get(&self, filename : &str) -> Option<&DialogueFile> {
        self.cache.get(filename)
    }
}

pub enum TickResult {
    Char(char),
    Newline,
    Wait,
    Clear,
    Done,
    Noop,
    SetStyling(Styling),
}

pub struct DialogueCursor {
    index: usize,

    i: usize,
    line: usize,
    exhausted: bool,
}

impl DialogueCursor {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            i: 0,
            line: 0,
            exhausted: false,
        }
    }
}

struct PeekResult<'a> {
    section: Option<&'a Dialogue>,
    chunk: Option<&'a Chunk>,
    set_exhausted: bool,
}

impl<'a> Default for PeekResult<'a> {
    fn default() -> Self {
        Self {
            section: None,
            chunk: None,
            set_exhausted: false,
        }
    }
}

impl<'a> DialogueCursor {
    pub fn peek_chunk(&self, file: &'a DialogueFile) -> PeekResult<'a> {
        if (self.exhausted) {
            return PeekResult::default();
        }

        if (self.index >= file.sections.len()) {
            return PeekResult {
                set_exhausted: true,
                ..Default::default()
            };
        }

        let section = &file.sections[self.index];

        if (self.line >= section.chunks.len()) {
            return PeekResult {
                set_exhausted: true,
                ..Default::default()
            };
        }

        let chunk = &section.chunks[self.line];

        PeekResult {
            section: Some(section),
            chunk: Some(chunk),
            set_exhausted: false,
        }
    }
}

impl DialogueCursor {
    pub fn tick_time(&mut self, file: &DialogueFile, base_text_rate: f32) -> f32 {
        let peeked = self.peek_chunk(file);
        if let Some(chunk) = peeked.chunk {
            if let Chunk::Command(command) = &chunk {
                if let Command::Wait(wait_ms) = &command {
                    return *wait_ms as f32 * 60.0 / 1000.0;
                }
            }
            else {
                return base_text_rate;
            }
        }

        return 0.0;
    }


    pub fn incr(&mut self, file: &DialogueFile, keypress: bool, click: bool) -> TickResult {
        let peeked = self.peek_chunk(file);
        if (peeked.chunk.is_none()) {
            if (peeked.set_exhausted) {
                self.exhausted = true;
            }

            return TickResult::Done;
        }

        let chunk = peeked.chunk.unwrap();

        match (chunk) {
            Chunk::Text(tc) => {
                assert!(self.i < tc.text.len());

                let c = tc.text.chars().nth(self.i).unwrap();

                self.i += 1;
                if (self.i == tc.text.len()) {
                    self.line += 1;
                    self.i = 0;
                }

                return TickResult::Char(c);
            },
            Chunk::Newline => {
                self.line += 1;
                return TickResult::Newline;
            },
            Chunk::Command(cc) => {
                match (cc) {
                    //.WaitPress => {
                    //    if (keypress) {
                    //        self.line += 1;
                    //    } else {
                    //        return .{ .WaitingForKeypress = void{} };
                    //    }
                    //},
                    //.WaitClick => {
                    //    if (click) {
                    //        self.line += 1;
                    //    } else {
                    //        return .{ .WaitingForClick = void{} };
                    //    }
                    //},
                    Command::Wait(time) => {
                        self.line += 1;
                    },
                    Command::Clear => {
                        self.line += 1;
                        return TickResult::Clear;
                    },
                    //.ClearLine => {
                    //    self.line += 1;
                    //    return .{ .ClearLine = void{} };
                    //},
                    //.SetTextRate => |r| {
                    //    self.line += 1;
                    //    return .{ .SetTextRate = r };
                    //},
                    //.DumpNoise => |r| {
                    //    self.line += 1;
                    //    return .{ .DumpNoise = r };
                    //},
                    //.DumpShaderAmp => |r| {
                    //    self.line += 1;
                    //    return .{ .DumpShaderAmp = r };
                    //},
                    Command::SetStyling(s) => {
                        self.line += 1;
                        return TickResult::SetStyling(s.clone());
                    },
                    //else => unreachable,
                    _ => {
                        panic!("Unsupported");
                    }
                }
            },
        }

        return TickResult::Noop;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn get_as_styling(chunk: &Chunk) -> &Styling {
        if let Chunk::Command(c) = chunk {
            if let Command::SetStyling(style) = c {
                return style;
            }
        }

        unreachable!()
    }

    fn get_as_wait(chunk: &Chunk) -> u32 {
        if let Chunk::Command(c) = chunk {
            if let Command::Wait(wait_ms) = c {
                return *wait_ms;
            }
        }

        unreachable!()
    }

    fn get_as_text(chunk: &Chunk) -> &str {
        if let Chunk::Text(t) = chunk {
            return &t.text;
        }

        unreachable!()
    }

    fn is_newline(chunk: &Chunk) -> bool {
        if let Chunk::Newline = chunk {
            return true;
        }

        false
    }


    #[test]
    fn test_parse_inline_command()
    {
        //let parsed = DialogueFile::parse_contents("test", "hellow (style jitter)jitter(style) nonjitter");
        let mut i: usize = 0;
        let parsed = DialogueFile::parse_section("test.adlib", "test", &["hellow (style jitter)jitter(style) nonjitter"], &mut i);
        let chunks = &parsed.chunks;

        println!("{:#?}", chunks);
        assert_eq!(chunks.len(), 8);

        assert_eq!(get_as_text(&chunks[0]), "hellow");
        assert!(get_as_styling(&chunks[1]).jitter);
        assert_eq!(get_as_text(&chunks[2]), " ");
        assert_eq!(get_as_text(&chunks[3]), "jitter");
        assert_eq!(get_as_styling(&chunks[4]), &Styling::default());
        assert_eq!(get_as_text(&chunks[5]), " ");
        assert_eq!(get_as_text(&chunks[6]), "nonjitter");
        assert!(is_newline(&chunks[7]));
    }


    #[test]
    fn test_multiline()
    {
        let mut i: usize = 0;
        let parsed = DialogueFile::parse_section("test.adlib", "test", 
            &["hello there",
            "(wait 1s)",
            "who are you?"], &mut i);

        let chunks = &parsed.chunks;

        println!("{:#?}", chunks);
        assert_eq!(chunks.len(), 5);

        assert_eq!(get_as_text(&chunks[0]), "hello there");
        assert!(is_newline(&chunks[1]));
        assert_eq!(60, get_as_wait(&chunks[2]));
        assert_eq!(get_as_text(&chunks[3]), "who are you?");
        assert!(is_newline(&chunks[4]));
    }
}
