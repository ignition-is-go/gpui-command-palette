use crate::{Modifier, Shortcut};
use gpui::{App, Window};
use std::{fmt, rc::Rc};

pub type CommandId = String;
type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
type Children<M> = Rc<dyn Fn() -> Vec<Command<M>>>;

#[derive(Clone)]
pub struct Command<M = ()> {
    pub id: CommandId,
    pub name: String,
    pub description: Option<String>,
    pub group: Option<String>,
    pub shortcut: Option<Shortcut>,
    pub metadata: M,
    handler: Handler,
    children: Option<Children<M>>,
    search_children: bool,
}
impl Command<()> {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        action: impl Fn() + 'static,
    ) -> Self {
        Self::with_metadata(id, name, (), move |_, _| action())
    }
    pub fn submenu(
        id: impl Into<String>,
        name: impl Into<String>,
        children: impl Fn() -> Vec<Self> + 'static,
    ) -> Self {
        Self::new(id, name, || {}).children(children)
    }
}
impl<M: 'static> Command<M> {
    pub fn with_metadata(
        id: impl Into<String>,
        name: impl Into<String>,
        metadata: M,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            group: None,
            shortcut: None,
            metadata,
            handler: Rc::new(handler),
            children: None,
            search_children: false,
        }
    }
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }
    pub fn group(mut self, value: impl Into<String>) -> Self {
        self.group = Some(value.into());
        self
    }
    pub fn shortcut(mut self, modifiers: Vec<Modifier>, key: impl Into<String>) -> Self {
        self.shortcut = Some(Shortcut::new(modifiers, key));
        self
    }
    pub fn children(mut self, children: impl Fn() -> Vec<Self> + 'static) -> Self {
        self.children = Some(Rc::new(children));
        self
    }
    pub fn searchable_children(mut self) -> Self {
        self.search_children = true;
        self
    }
    pub const fn searches_children(&self) -> bool {
        self.search_children
    }
    pub fn is_branch(&self) -> bool {
        self.children.is_some()
    }
    pub fn resolve_children(&self) -> Option<Vec<Self>> {
        self.children.as_ref().map(|f| f())
    }
    pub fn execute_in(&self, window: &mut Window, cx: &mut App) {
        (self.handler)(window, cx)
    }
}
impl<M: Default + 'static> Command<M> {
    pub fn with_handler(
        id: impl Into<String>,
        name: impl Into<String>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self::with_metadata(id, name, M::default(), handler)
    }
}
impl<M> PartialEq for Command<M> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<M: fmt::Debug + 'static> fmt::Debug for Command<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("group", &self.group)
            .field("shortcut", &self.shortcut)
            .field("metadata", &self.metadata)
            .field("is_branch", &self.is_branch())
            .finish()
    }
}
