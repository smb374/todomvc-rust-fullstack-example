use getset::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, CopyGetters, Serialize, Deserialize)]
#[getset(get_copy = "pub")]
pub struct Query {
    id: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TaskRequest<'a> {
    pub content: &'a str,
}

#[derive(Debug, Clone, Getters, Setters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Entry {
    id: Uuid,
    content: String,
    completed: bool,
    editing: bool,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            content: String::new(),
            completed: false,
            editing: false,
        }
    }
}
impl Entry {
    pub fn new(content: String) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            content,
            completed: false,
            editing: false,
        }
    }
    pub fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
    pub fn set_completed(&mut self, completed: bool) {
        self.completed = completed;
    }
    pub fn set_editing(&mut self, editing: bool) {
        self.editing = editing;
    }
}

pub type UpdateRequest = Entry;
pub type UpdateAll = Vec<UpdateRequest>;
pub type Entries = Vec<Entry>;
