use serde::{Deserialize, Serialize};
use strum::{EnumIter, ToString};
use todomvc_shared::Entry;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub entries: Vec<Entry>,
    pub filter: Filter,
    pub value: String,
    pub edit_value: String,
}

impl State {
    pub fn total(&self) -> usize {
        self.entries.len()
    }

    pub fn total_completed(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| Filter::Completed.fits(e))
            .count()
    }

    pub fn is_all_completed(&self) -> bool {
        let mut filtered_iter = self
            .entries
            .iter()
            .filter(|e| self.filter.fits(e))
            .peekable();

        if filtered_iter.peek().is_none() {
            return false;
        }

        filtered_iter.all(|e| *e.completed())
    }

    pub fn clear_completed(&mut self) {
        let entries = self
            .entries
            .drain(..)
            .filter(|e| Filter::Active.fits(e))
            .collect();
        self.entries = entries;
    }

    pub fn toggle(&mut self, idx: usize) -> Entry {
        let filter = self.filter;
        let entry = self
            .entries
            .iter_mut()
            .filter(|e| filter.fits(e))
            .nth(idx)
            .unwrap();
        entry.set_completed(!*entry.completed());
        entry.clone()
    }

    pub fn toggle_all(&mut self, value: bool) {
        let filter = self.filter;
        self.entries
            .iter_mut()
            .filter(|e| filter.fits(e))
            .for_each(|e| {
                e.set_completed(value);
            });
    }

    pub fn toggle_edit(&mut self, idx: usize) -> Entry {
        let filter = self.filter;
        let entry = self
            .entries
            .iter_mut()
            .filter(|e| filter.fits(e))
            .nth(idx)
            .unwrap();
        entry.set_editing(!*entry.editing());
        entry.clone()
    }

    pub fn clear_all_edit(&mut self) {
        let filter = self.filter;
        self.entries
            .iter_mut()
            .filter(|e| filter.fits(e))
            .for_each(|e| {
                e.set_completed(false);
            });
    }

    pub fn complete_edit(&mut self, idx: usize, val: String) -> (Entry, bool) {
        if val.is_empty() {
            let e = self.remove(idx);
            (e, true)
        } else {
            let filter = self.filter;
            let entry = self
                .entries
                .iter_mut()
                .filter(|e| filter.fits(e))
                .nth(idx)
                .unwrap();
            entry.set_content(val);
            entry.set_editing(!*entry.editing());
            (entry.clone(), false)
        }
    }

    pub fn remove(&mut self, idx: usize) -> Entry {
        let idx = {
            let entries = self
                .entries
                .iter()
                .enumerate()
                .filter(|&(_, e)| self.filter.fits(e))
                .collect::<Vec<_>>();
            let &(idx, _) = entries.get(idx).unwrap();
            idx
        };
        self.entries.remove(idx)
    }
}

#[derive(Clone, Copy, Debug, EnumIter, ToString, PartialEq, Serialize, Deserialize)]
pub enum Filter {
    All,
    Active,
    Completed,
}
impl Filter {
    pub fn fits(&self, entry: &Entry) -> bool {
        match *self {
            Filter::All => true,
            Filter::Active => !*entry.completed(),
            Filter::Completed => *entry.completed(),
        }
    }

    pub fn as_href(&self) -> &'static str {
        match self {
            Filter::All => "#/",
            Filter::Active => "#/active",
            Filter::Completed => "#/completed",
        }
    }
}
