//! A reusable command palette for the same GPUI view on native and WebAssembly.
mod command;
mod registry;
mod search;
mod shortcut;
mod state;
mod theme;
mod widget;
pub use command::*;
pub use registry::*;
pub use search::*;
pub use shortcut::*;
pub use state::*;
pub use theme::*;
pub use widget::*;

use gpui::{App, Entity, EntityId, Global, KeyBinding, Subscription, WeakEntity, Window, WindowId};
use std::{any::Any, collections::HashMap, rc::Rc};

type PaletteRoute = Rc<dyn Fn(PaletteRouteAction, &mut Window, &mut App) -> bool>;

#[derive(Clone, Copy)]
enum PaletteRouteAction {
    Toggle,
    Open,
    Close,
}

struct InstalledPaletteRoute {
    entity_id: EntityId,
    registry: Box<dyn Any>,
    callback: PaletteRoute,
    _release_subscription: Subscription,
}

#[derive(Default)]
struct PaletteRoutes {
    windows: HashMap<WindowId, InstalledPaletteRoute>,
    _window_subscription: Option<Subscription>,
}

impl Global for PaletteRoutes {}

/// Per-window access to the installed shared command palette.
///
/// These methods intentionally target `CommandPalette<()>`, the interoperable
/// palette used by independent downstream crates. Registration handles own
/// command lifetime: dropping a [`Registration`] unregisters its command.
pub trait ActiveCommandPalette {
    /// Register one command with `window`'s installed shared palette.
    ///
    /// Returns `None` when no `CommandPalette<()>` is installed for the window.
    fn register_command_palette_command(
        &mut self,
        window: &Window,
        command: Command<()>,
    ) -> Option<Registration<()>>;

    /// Register several commands with `window`'s installed shared palette.
    ///
    /// The returned handles must be retained for as long as the commands should
    /// remain registered.
    fn register_command_palette_commands(
        &mut self,
        window: &Window,
        commands: impl IntoIterator<Item = Command<()>>,
    ) -> Option<Vec<Registration<()>>>;

    /// Invalidate `window`'s installed shared palette after scoped handles drop.
    ///
    /// Registration through this trait invalidates automatically. Call this
    /// after dropping registrations while an open palette must repaint
    /// immediately.
    fn refresh_command_palette_commands(&mut self, window: &Window) -> bool;
}

impl ActiveCommandPalette for App {
    fn register_command_palette_command(
        &mut self,
        window: &Window,
        command: Command<()>,
    ) -> Option<Registration<()>> {
        let window_id = window.window_handle().window_id();
        let (entity_id, registry) = {
            let route = self
                .try_global::<PaletteRoutes>()?
                .windows
                .get(&window_id)?;
            let registry = route
                .registry
                .downcast_ref::<CommandRegistry<()>>()?
                .clone();
            (route.entity_id, registry)
        };
        let registration = registry.register(command);
        self.notify(entity_id);
        Some(registration)
    }

    fn register_command_palette_commands(
        &mut self,
        window: &Window,
        commands: impl IntoIterator<Item = Command<()>>,
    ) -> Option<Vec<Registration<()>>> {
        let window_id = window.window_handle().window_id();
        let (entity_id, registry) = {
            let route = self
                .try_global::<PaletteRoutes>()?
                .windows
                .get(&window_id)?;
            let registry = route
                .registry
                .downcast_ref::<CommandRegistry<()>>()?
                .clone();
            (route.entity_id, registry)
        };
        let registrations = registry.register_many(commands);
        self.notify(entity_id);
        Some(registrations)
    }

    fn refresh_command_palette_commands(&mut self, window: &Window) -> bool {
        let window_id = window.window_handle().window_id();
        let Some(entity_id) = self
            .try_global::<PaletteRoutes>()
            .and_then(|routes| routes.windows.get(&window_id))
            .and_then(|route| {
                route
                    .registry
                    .is::<CommandRegistry<()>>()
                    .then_some(route.entity_id)
            })
        else {
            return false;
        };
        self.notify(entity_id);
        true
    }
}

fn dispatch_to_active_palette(action: PaletteRouteAction, cx: &mut App) {
    let Some(window_handle) = cx.active_window() else {
        return;
    };
    let window_id = window_handle.window_id();
    // Global action listeners run while the source window is already on GPUI's update stack.
    // Defer the entity update until dispatch unwinds, while retaining the originating active id.
    cx.defer(move |cx| {
        let callback = cx
            .global::<PaletteRoutes>()
            .windows
            .get(&window_id)
            .map(|route| route.callback.clone());
        let Some(callback) = callback else {
            return;
        };
        let alive = window_handle
            .update(cx, |_, window, cx| callback(action, window, cx))
            .unwrap_or(false);
        if !alive {
            cx.global_mut::<PaletteRoutes>().windows.remove(&window_id);
        }
    });
}

/// Register the palette key bindings and application-level action routing.
///
/// Call this once while initializing the application. Each window's palette must then be
/// registered with [`install_palette`]. Application-level routing is what lets Ctrl/⌘+K open a
/// closed palette even when the palette element is not on the current focus route.
pub fn init(cx: &mut App) {
    if cx.has_global::<PaletteRoutes>() {
        return;
    }

    cx.set_global(PaletteRoutes::default());
    let window_subscription = cx.on_window_closed(|cx, window_id| {
        cx.global_mut::<PaletteRoutes>().windows.remove(&window_id);
    });
    cx.global_mut::<PaletteRoutes>()._window_subscription = Some(window_subscription);

    let binding = if crate::shortcut::is_mac() {
        "cmd-k"
    } else {
        "ctrl-k"
    };
    cx.bind_keys([KeyBinding::new(binding, ToggleCommandPalette, None)]);
    cx.on_action::<ToggleCommandPalette>(|_, cx| {
        dispatch_to_active_palette(PaletteRouteAction::Toggle, cx)
    });
    cx.on_action::<OpenCommandPalette>(|_, cx| {
        dispatch_to_active_palette(PaletteRouteAction::Open, cx)
    });
    cx.on_action::<CloseCommandPalette>(|_, cx| {
        dispatch_to_active_palette(PaletteRouteAction::Close, cx)
    });
}

/// Install `palette` as the action target for `window`.
///
/// One palette is supported per window; installing another atomically replaces the old route.
/// Routes hold only a [`WeakEntity`] and are removed when either the palette is released or the
/// window closes. This function also calls [`init`] so hosts cannot accidentally omit the global
/// action handlers.
pub fn install_palette<M: Clone + 'static>(
    palette: &Entity<CommandPalette<M>>,
    window: &Window,
    cx: &mut App,
) {
    init(cx);
    let window_id = window.window_handle().window_id();
    let entity_id = palette.entity_id();
    let registry: Box<dyn Any> = Box::new(palette.read(cx).registry().clone());
    let weak: WeakEntity<CommandPalette<M>> = palette.downgrade();
    let callback: PaletteRoute = Rc::new(move |action, window, cx| {
        weak.update(cx, |palette, cx| match action {
            PaletteRouteAction::Toggle => palette.toggle(window, cx),
            PaletteRouteAction::Open => palette.open(window, cx),
            PaletteRouteAction::Close => palette.close(window, cx),
        })
        .is_ok()
    });
    let release_subscription = cx.observe_release(palette, move |_, cx| {
        let routes = &mut cx.global_mut::<PaletteRoutes>().windows;
        if routes
            .get(&window_id)
            .is_some_and(|route| route.entity_id == entity_id)
        {
            routes.remove(&window_id);
        }
    });
    cx.global_mut::<PaletteRoutes>().windows.insert(
        window_id,
        InstalledPaletteRoute {
            entity_id,
            registry,
            callback,
            _release_subscription: release_subscription,
        },
    );
}

#[cfg(test)]
mod route_tests {
    use super::*;
    use gpui::{div, prelude::*, Context, FocusHandle, Render, TestAppContext};

    struct PaletteHost {
        palette: Option<Entity<CommandPalette>>,
        outside_focus: FocusHandle,
    }

    impl PaletteHost {
        fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
            set_command_palette_theme(cx, CommandPaletteTheme::default());
            let palette = cx.new(CommandPalette::new);
            install_palette(&palette, window, cx);
            let outside_focus = cx.focus_handle();
            window.focus(&outside_focus, cx);
            Self {
                palette: Some(palette),
                outside_focus,
            }
        }
    }

    impl Render for PaletteHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .track_focus(&self.outside_focus)
                .child("focus is outside the palette")
                .children(self.palette.clone())
        }
    }

    fn is_open(palette: &Entity<CommandPalette>, cx: &impl gpui::AppContext) -> bool {
        palette.read_with(cx, |palette, _| palette.state().is_open())
    }

    #[gpui::test]
    fn global_route_toggles_exactly_once_when_root_has_focus(cx: &mut TestAppContext) {
        let (host, cx) = cx.add_window_view(PaletteHost::new);
        let palette = host.read_with(cx, |host, _| host.palette.as_ref().unwrap().clone());
        assert!(cx.update(|window, cx| host.read(cx).outside_focus.is_focused(window)));
        cx.update(|window, _| window.activate_window());

        cx.dispatch_action(ToggleCommandPalette);
        cx.run_until_parked();
        assert!(is_open(&palette, cx));

        cx.dispatch_action(ToggleCommandPalette);
        cx.run_until_parked();
        assert!(!is_open(&palette, cx));
    }

    #[gpui::test]
    fn routes_two_windows_independently(cx: &mut TestAppContext) {
        cx.update(init);
        let first = cx.add_window(PaletteHost::new);
        let second = cx.add_window(PaletteHost::new);
        let first_palette = first
            .root(cx)
            .unwrap()
            .read_with(cx, |host, _| host.palette.as_ref().unwrap().clone());
        let second_palette = second
            .root(cx)
            .unwrap()
            .read_with(cx, |host, _| host.palette.as_ref().unwrap().clone());

        first
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.dispatch_action(first.into(), ToggleCommandPalette);
        cx.run_until_parked();
        assert!(is_open(&first_palette, cx));
        assert!(!is_open(&second_palette, cx));

        second
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.dispatch_action(second.into(), ToggleCommandPalette);
        cx.run_until_parked();
        assert!(is_open(&first_palette, cx));
        assert!(is_open(&second_palette, cx));
    }

    #[gpui::test]
    fn downstream_context_registration_is_window_scoped_and_raii(cx: &mut TestAppContext) {
        let window = cx.add_window(PaletteHost::new);
        let palette = window
            .root(cx)
            .unwrap()
            .read_with(cx, |host, _| host.palette.as_ref().unwrap().clone());

        let registration = window
            .update(cx, |_, window, cx| {
                cx.register_command_palette_command(
                    window,
                    Command::new("downstream", "Downstream Command", || {}),
                )
            })
            .unwrap()
            .expect("shared palette should be installed");
        assert_eq!(
            palette.read_with(cx, |palette, _| palette.registry().len()),
            1
        );

        drop(registration);
        assert!(palette.read_with(cx, |palette, _| palette.registry().is_empty()));
        assert!(window
            .update(cx, |_, window, cx| {
                cx.refresh_command_palette_commands(window)
            })
            .unwrap());
    }

    #[gpui::test]
    fn replacement_and_release_cleanup_preserve_only_the_current_route(cx: &mut TestAppContext) {
        let window = cx.add_window(PaletteHost::new);
        let window_id = window.window_id();
        let replacement = cx.update(|cx| cx.new(CommandPalette::new));
        window
            .update(cx, |_, window, cx| {
                install_palette(&replacement, window, cx)
            })
            .unwrap();
        let original = window.root(cx).unwrap().update(cx, |host, _| {
            host.palette.replace(replacement.clone()).unwrap()
        });

        drop(original);
        cx.run_until_parked();
        assert_eq!(
            cx.read_global::<PaletteRoutes, _>(|routes, _| routes.windows[&window_id].entity_id),
            replacement.entity_id()
        );

        window
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.dispatch_action(window.into(), ToggleCommandPalette);
        cx.run_until_parked();
        assert!(is_open(&replacement, cx));

        let installed = window.root(cx).unwrap().update(cx, |host, cx| {
            let installed = host.palette.take().unwrap();
            cx.notify();
            installed
        });
        drop(replacement);
        drop(installed);
        cx.refresh().unwrap();
        cx.run_until_parked();

        window
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.dispatch_action(window.into(), ToggleCommandPalette);
        cx.run_until_parked();
        assert!(cx.read_global::<PaletteRoutes, _>(|routes, _| routes.windows.is_empty()));

        window
            .update(cx, |_, window, cx| {
                let palette = cx.new(CommandPalette::<()>::new);
                install_palette(&palette, window, cx);
                window.remove_window();
            })
            .unwrap();
        cx.run_until_parked();
        assert!(cx.read_global::<PaletteRoutes, _>(|routes, _| routes.windows.is_empty()));
    }
}
