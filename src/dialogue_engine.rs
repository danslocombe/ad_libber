use crate::dialogue::*;

//const CLEAR_T_MAX : f32 = 35.0;
//const CLEAR_T_SUCK_MIN : f32 = 31.0;

#[derive(Clone, Debug)]
pub struct DialogueEngineOptions
{
    pub line_linger_time : f32,
    pub text_rate : f32,
}

impl Default for DialogueEngineOptions
{
    fn default() -> Self {
        Self {
            line_linger_time : 240.0,
            text_rate : 0.75,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct DialogueEngine
{
    pub options : DialogueEngineOptions,
    cursor : Option<DialogueCursor>,
    annotated_string : AnnotatedString,

    t : f32,

    line_linger_t : f32,
}

impl DialogueEngine {
    pub fn queue(&mut self, dialogue : &Dialogue) {
        if let Some(c) = self.cursor.as_ref() {
            if c.dialogue_name_eq(dialogue) {
                // Already queued
                // Reduce clear time
                self.line_linger_t /= 2.0;
                return;
            }
        }

        self.clear();
        self.cursor = Some(DialogueCursor::new(dialogue));
    }

    pub fn clear(&mut self) {
        self.cursor = None;
        self.annotated_string = Default::default();
        self.t = 0.0;
        self.line_linger_t = 0.0;
    }

    pub fn current_string_iter(&self) -> OwnedAnnotatedStringIterator {
        self.annotated_string.clone().owned_iter()
    }

    pub fn tick(&mut self, dt_norm : f32) {
        if (self.cursor.is_none()) {
            return;
        }

        if (self.line_linger_t > 0.0) {
            self.line_linger_t += dt_norm;
            /*
            let cur_line = self.cursor.as_ref().unwrap().get();
            let mut line_len = cur_line.string.len();

            let mut ix = 0.0;

            if (self.line_linger_t > CLEAR_T_SUCK_MIN) {
                let norm = ((self.line_linger_t - CLEAR_T_SUCK_MIN) / (CLEAR_T_MAX-CLEAR_T_SUCK_MIN))
                    .clamp(0.0, 1.0);

                ix = line_len as f32 * norm;
            }
            */

            if (self.line_linger_t > self.options.line_linger_time) {
                self.clear();
            }

            return;
        }

        self.t += dt_norm;

        while (self.t > self.options.text_rate) {
            self.t -= self.options.text_rate;
            if (!self.cursor.as_mut().unwrap().incr()) {
                // Bit hacky set to one second
                self.line_linger_t = 1.0;
                break;
            }
        }

        self.annotated_string = self.cursor.as_ref().unwrap().get();
    }
}