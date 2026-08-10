use crate::Command;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RegistrationId(u64);
#[derive(Clone)]
pub struct CommandRegistry<M = ()>(Rc<RefCell<RegistryInner<M>>>);
struct RegistryInner<M> {
    next: u64,
    revision: u64,
    entries: Vec<(RegistrationId, Command<M>)>,
}
impl<M> Default for CommandRegistry<M> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(RegistryInner {
            next: 0,
            revision: 0,
            entries: Vec::new(),
        })))
    }
}
impl<M: Clone> CommandRegistry<M> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register(&self, command: Command<M>) -> Registration<M> {
        let mut inner = self.0.borrow_mut();
        let id = if let Some(index) = inner
            .entries
            .iter()
            .position(|(_, existing)| existing.id == command.id)
        {
            let id = inner.entries[index].0;
            inner.entries[index].1 = command;
            id
        } else {
            inner.next += 1;
            let id = RegistrationId(inner.next);
            inner.entries.push((id, command));
            id
        };
        inner.revision += 1;
        Registration {
            id,
            registry: Rc::downgrade(&self.0),
            active: true,
        }
    }
    pub fn register_many(
        &self,
        commands: impl IntoIterator<Item = Command<M>>,
    ) -> Vec<Registration<M>> {
        commands.into_iter().map(|c| self.register(c)).collect()
    }
    pub fn update(&self, id: RegistrationId, command: Command<M>) -> bool {
        let mut x = self.0.borrow_mut();
        if let Some(slot) = x.entries.iter_mut().find(|e| e.0 == id) {
            slot.1 = command;
            x.revision += 1;
            true
        } else {
            false
        }
    }
    pub fn unregister(&self, id: RegistrationId) -> bool {
        let mut x = self.0.borrow_mut();
        let n = x.entries.len();
        x.entries.retain(|e| e.0 != id);
        let changed = n != x.entries.len();
        if changed {
            x.revision += 1
        };
        changed
    }
    pub fn unregister_command(&self, command_id: &str) -> bool {
        let mut x = self.0.borrow_mut();
        let n = x.entries.len();
        x.entries.retain(|e| e.1.id != command_id);
        let c = n != x.entries.len();
        if c {
            x.revision += 1
        };
        c
    }
    pub fn commands(&self) -> Vec<Command<M>> {
        self.0
            .borrow()
            .entries
            .iter()
            .map(|e| e.1.clone())
            .collect()
    }
    pub fn revision(&self) -> u64 {
        self.0.borrow().revision
    }
    pub fn len(&self) -> usize {
        self.0.borrow().entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
pub struct Registration<M> {
    id: RegistrationId,
    registry: Weak<RefCell<RegistryInner<M>>>,
    active: bool,
}
impl<M> Registration<M> {
    pub const fn id(&self) -> RegistrationId {
        self.id
    }
    pub fn forget(mut self) {
        self.active = false
    }
}
impl<M> Drop for Registration<M> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Some(r) = self.registry.upgrade() {
            let mut x = r.borrow_mut();
            let n = x.entries.len();
            x.entries.retain(|e| e.0 != self.id);
            if n != x.entries.len() {
                x.revision += 1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_update_and_dynamic_unregister() {
        let r = CommandRegistry::new();
        let a = r.register(Command::new("a", "A", || {}));
        let _b = r.register(Command::new("b", "B", || {}));
        assert!(r.update(a.id(), Command::new("a", "AA", || {})));
        assert_eq!(
            r.commands()
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>(),
            ["AA", "B"]
        );
        drop(a);
        assert_eq!(r.commands()[0].id, "b");
    }
    #[test]
    fn duplicate_id_updates_without_reordering() {
        let r = CommandRegistry::new();
        let _a = r.register(Command::new("a", "A", || {}));
        let b = r.register(Command::new("b", "B", || {}));
        let update = r.register(Command::new("a", "A2", || {}));
        assert_eq!(
            r.commands()
                .iter()
                .map(|c| c.id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        update.forget();
        b.forget();
    }
}
