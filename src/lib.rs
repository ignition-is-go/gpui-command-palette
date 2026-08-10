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
use std::{collections::HashMap, rc::Rc};

type PaletteRoute = Rc<dyn Fn(PaletteRouteAction, &mut Window, &mut App) -> bool>;

#[derive(Clone, Copy)]
enum PaletteRouteAction {
    Toggle,
    Open,
    Close,
}

struct InstalledPaletteRoute {
    entity_id: EntityId,
    callback: PaletteRoute,
    _release_subscription: Subscription,
}

#[derive(Default)]
struct PaletteRoutes {
    windows: HashMap<WindowId, InstalledPaletteRoute>,
    _window_subscription: Option<Subscription>,
}

impl Global for PaletteRoutes {}

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
