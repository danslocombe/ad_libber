use std::collections::HashSet;

use crate::dialogue_engine::DialogueEngine;
use crate::dialogue::{Dialogue, DialogueCache};
use crate::interop::iter_wrapper::IterWrapper;
use crate::interop::queue_params::QueueParams;


#[derive(Default)]
pub struct GlobalState
{
    pub path : String,
    pub engine : DialogueEngine,
    pub cache : DialogueCache,
    pub one_shot_cache : HashSet<FilenameSectionPair>,
    pub iter_wrapper : Option<IterWrapper>,
}

impl GlobalState
{
    pub fn full_filename(&self, filename : &str) -> String {
        self.path.clone() + filename + ".adlib"
    }

    pub fn preload(&mut self, filename : &str) {
        self.cache.preload(&self.full_filename(filename));
    }

}

impl<'a> GlobalState
{
    pub fn queue(&mut self, queue_args: QueueParams<'a>) {
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
                self.engine.queue(dialogue);
            }
            else {
                self.engine.queue(&Dialogue::from_error(&format!("Could not find dialogue section {}", queue_args.section)));
            }
        }
        else {
            self.engine.queue(&Dialogue::from_error(&format!("Could not find dialogue file {}", queue_args.filename)));
        }
    }
}

#[derive(Debug)]
pub struct FilenameSectionPair
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