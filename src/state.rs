use crate::{search_commands, Command, CommandId, SearchResult};
#[derive(Clone, Debug)]
pub struct NavigationLevel<M: 'static = ()> {
    pub id: CommandId,
    pub label: String,
    pub items: Vec<Command<M>>,
}
#[derive(Clone, Debug)]
pub struct PaletteState<M: 'static = ()> {
    open: bool,
    query: String,
    selected: usize,
    stack: Vec<NavigationLevel<M>>,
}
impl<M: 'static> Default for PaletteState<M> {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected: 0,
            stack: Vec::new(),
        }
    }
}
impl<M: Clone + 'static> PaletteState<M> {
    pub fn new() -> Self {
        Self::default()
    }
    pub const fn is_open(&self) -> bool {
        self.open
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub const fn selected_index(&self) -> usize {
        self.selected
    }
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
    pub fn navigation(&self) -> &[NavigationLevel<M>] {
        &self.stack
    }
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.stack.clear()
    }
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.selected = 0;
        self.stack.clear()
    }
    pub fn toggle(&mut self) {
        if self.open {
            self.close()
        } else {
            self.open()
        }
    }
    pub fn set_query(&mut self, q: impl Into<String>) {
        self.query = q.into();
        self.selected = 0
    }
    pub fn push_text(&mut self, text: &str) {
        self.query.push_str(text);
        self.selected = 0
    }
    pub fn backspace(&mut self) {
        if self.query.pop().is_none() {
            self.back();
        }
        self.selected = 0
    }
    pub fn results(&self, root: &[Command<M>]) -> Vec<SearchResult<M>> {
        search_commands(
            self.stack
                .last()
                .map(|l| l.items.as_slice())
                .unwrap_or(root),
            &self.query,
        )
    }
    pub fn clamp_selection(&mut self, count: usize) {
        self.selected = self.selected.min(count.saturating_sub(1));
    }
    pub fn select_next(&mut self, count: usize) {
        if count > 0 {
            self.selected = (self.selected + 1).min(count - 1)
        }
    }
    pub fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1)
    }
    pub fn select(&mut self, index: usize, count: usize) {
        self.selected = index.min(count.saturating_sub(1))
    }
    pub fn enter(&mut self, command: &Command<M>) -> bool {
        if let Some(items) = command.resolve_children() {
            self.stack.push(NavigationLevel {
                id: command.id.clone(),
                label: command.name.clone(),
                items,
            });
            self.query.clear();
            self.selected = 0;
            true
        } else {
            false
        }
    }
    pub fn back(&mut self) -> bool {
        let popped = self.stack.pop().is_some();
        if popped {
            self.query.clear();
            self.selected = 0
        }
        popped
    }
    pub fn pop_to(&mut self, depth: usize) {
        self.stack.truncate(depth);
        self.query.clear();
        self.selected = 0
    }
    pub fn back_or_close(&mut self) {
        if !self.back() {
            self.close()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_keyboard_state() {
        let mut s = PaletteState::new();
        let root = vec![
            Command::new("a", "Alpha", || {}),
            Command::new("b", "Beta", || {}),
        ];
        s.open();
        s.select_next(2);
        assert_eq!(s.selected_index(), 1);
        s.set_query("alpha");
        assert_eq!(s.results(&root)[0].entry.id, "a");
        s.back_or_close();
        assert!(!s.is_open())
    }
    #[test]
    fn submenu_snapshots() {
        let mut s = PaletteState::new();
        let branch = Command::submenu("x", "X", || vec![Command::new("y", "Y", || {})]);
        assert!(s.enter(&branch));
        assert_eq!(s.depth(), 1);
        s.backspace();
        assert_eq!(s.depth(), 0);
    }
}
