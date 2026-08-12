use crate::{Modifier, Shortcut};
use gpui::{App, Window};
use std::{fmt, sync::Arc};

pub type CommandId = String;
type Action = Arc<dyn Fn() + Send + Sync>;
type Handler = Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>;
type Children<M> = Arc<dyn Fn() -> Vec<Command<M>> + Send + Sync>;

#[derive(Clone)]
pub struct Command<M = ()> {
    pub id: CommandId,
    pub name: String,
    pub description: Option<String>,
    pub group: Option<String>,
    pub shortcut: Option<Shortcut>,
    pub metadata: M,
    action: Action,
    handler: Option<Handler>,
    children: Option<Children<M>>,
    search_children: bool,
}
impl Command<()> {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        action: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self::with_metadata(id, name, (), action)
    }
    pub fn submenu(
        id: impl Into<String>,
        name: impl Into<String>,
        children: impl Fn() -> Vec<Self> + Send + Sync + 'static,
    ) -> Self {
        Self::new(id, name, || {}).children(children)
    }
}
impl<M: 'static> Command<M> {
    /// Construct portable command data. The action and child producer retain the
    /// reference crate's `Send + Sync` guarantee; GPUI-aware execution is an
    /// optional, separately installed hook.
    pub fn with_metadata(
        id: impl Into<String>,
        name: impl Into<String>,
        metadata: M,
        action: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            group: None,
            shortcut: None,
            metadata,
            action: Arc::new(action),
            handler: None,
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
    pub fn children(mut self, children: impl Fn() -> Vec<Self> + Send + Sync + 'static) -> Self {
        self.children = Some(Arc::new(children));
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
    pub fn execute(&self) {
        (self.action)()
    }
    pub fn execute_in(&self, window: &mut Window, cx: &mut App) {
        (self.action)();
        if let Some(handler) = &self.handler {
            handler(window, cx);
        }
    }
}
impl<M: Default + 'static> Command<M> {
    /// Add a GPUI-aware execution hook. Prefer portable `with_metadata` and subscribe to
    /// [`crate::CommandPaletteEvent`] when coordinating caller-owned GPUI state.
    pub fn with_handler(
        id: impl Into<String>,
        name: impl Into<String>,
        handler: impl Fn(&mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        let mut command = Self::with_metadata(id, name, M::default(), || {});
        command.handler = Some(Arc::new(handler));
        command
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

#[cfg(test)]
mod tests {
    use super::*;
    fn assert_send_sync<T: Send + Sync>() {}
    #[test]
    fn portable_commands_are_send_sync() {
        assert_send_sync::<Command>();
    }
    #[test]
    fn live_children_and_execute() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let c = Command::submenu("x", "X", || {
            let n = N.fetch_add(1, Ordering::SeqCst) + 1;
            (0..n)
                .map(|i| Command::new(i.to_string(), "item", || {}))
                .collect()
        });
        assert_eq!(c.resolve_children().unwrap().len(), 1);
        assert_eq!(c.resolve_children().unwrap().len(), 2);
    }
}
