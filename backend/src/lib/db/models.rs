use super::schema::task;
use getset::*;
use todomvc_shared::Entry;
use uuid::Uuid;

#[derive(Debug, Insertable, Queryable, Identifiable, AsChangeset, Getters, Setters, Clone)]
#[table_name = "task"]
#[getset(get = "pub", set = "pub")]
pub struct Task {
    id: Uuid,
    content: String,
    completed: bool,
    editing: bool,
}

impl Task {
    pub fn new(content: String) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            content,
            completed: false,
            editing: false,
        }
    }
    pub fn to_entry(&self) -> Entry {
        let mut e = Entry::default();
        e.set_id(*self.id());
        e.set_content(self.content().clone());
        e.set_completed(*self.completed());
        e.set_editing(*self.editing());
        e
    }
}

impl From<Entry> for Task {
    fn from(e: Entry) -> Self {
        Self {
            id: *e.id(),
            content: e.content().to_string(),
            completed: *e.completed(),
            editing: *e.editing(),
        }
    }
}
impl From<&Entry> for Task {
    fn from(e: &Entry) -> Self {
        Self {
            id: *e.id(),
            content: e.content().to_string(),
            completed: *e.completed(),
            editing: *e.editing(),
        }
    }
}
