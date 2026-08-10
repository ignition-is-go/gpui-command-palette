use gpui::{div, prelude::*, px, App, Bounds, FocusHandle, WindowBounds, WindowOptions};
use gpui_command_palette::{Command, CommandPalette, Modifier};

#[cfg(target_family = "wasm")]
thread_local! {
    static LAST_EXECUTION: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}
#[cfg(target_family = "wasm")]
fn record_execution(id: &str) {
    LAST_EXECUTION.with(|value| *value.borrow_mut() = Some(id.to_owned()));
}
#[cfg(not(target_family = "wasm"))]
fn record_execution(_: &str) {}

#[cfg(target_family = "wasm")]
fn json_string(value: &str) -> String {
    format!(
        r#""{}""#,
        value
            .replace('\\', r"\\")
            .replace('"', r#"\""#)
            .replace('\n', r"\n")
            .replace('\r', r"\r")
    )
}

#[cfg(target_family = "wasm")]
fn publish_test_bridge(palette: &CommandPalette) {
    let state = palette.state();
    let results = state
        .results(&palette.registry().commands())
        .into_iter()
        .map(|result| json_string(&result.entry.id))
        .collect::<Vec<_>>()
        .join(",");
    let executed = LAST_EXECUTION.with(|value| {
        value
            .borrow()
            .as_ref()
            .map(|id| json_string(id))
            .unwrap_or_else(|| "null".to_owned())
    });
    let value = format!(
        r#"{{"open":{},"query":{},"selected":{},"depth":{},"results":[{}],"executed":{}}}"#,
        state.is_open(),
        json_string(state.query()),
        state.selected_index(),
        state.depth(),
        results,
        executed
    );
    // NOTE(ts): this same-document bridge mirrors the real palette entity only
    // for browser CI; it neither drives state nor introduces a second UI tree.
    if let Some(window) = web_sys::window() {
        let _ = window.set_name(&value);
    }
}

fn launch(cx: &mut App) {
    gpui_command_palette::init(cx);
    let bounds = Bounds::centered(None, gpui::size(px(900.), px(600.)), cx);
    let demo_window = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let palette = cx.new(CommandPalette::new);
                gpui_command_palette::install_palette(&palette, window, cx);
                let registrations = palette.read(cx).registry().register_many([
                    Command::new("file.open", "Open File", || record_execution("file.open"))
                        .description("Open a file from disk")
                        .group("File")
                        .shortcut(vec![Modifier::Main], "o"),
                    Command::submenu("theme", "Change Theme", || {
                        vec![
                            Command::new("theme.dark", "Dark", || record_execution("theme.dark")),
                            Command::new("theme.light", "Light", || {
                                record_execution("theme.light")
                            }),
                        ]
                    })
                    .searchable_children(),
                ]);
                for registration in registrations {
                    registration.forget()
                }
                let demo = cx.new(|cx| {
                    cx.observe(&palette, |_, _, cx| cx.notify()).detach();
                    Demo {
                        palette,
                        focus: cx.focus_handle(),
                    }
                });
                let focus = demo.read(cx).focus.clone();
                focus.focus(window, cx);
                demo
            },
        )
        .unwrap();
    demo_window
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.activate(true);
    #[cfg(target_family = "wasm")]
    cx.refresh_windows();
}
struct Demo {
    palette: gpui::Entity<CommandPalette>,
    focus: FocusHandle,
}
impl gpui::Render for Demo {
    fn render(&mut self, _: &mut gpui::Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        #[cfg(target_family = "wasm")]
        publish_test_bridge(self.palette.read(_cx));
        div()
            .track_focus(&self.focus)
            .size_full()
            .bg(gpui::rgb(0x111111))
            .text_color(gpui::white())
            .items_center()
            .justify_center()
            .child("Press Ctrl/⌘+K")
            .child(self.palette.clone())
    }
}
#[cfg(not(target_family = "wasm"))]
fn main() {
    gpui_platform::application().run(launch)
}
#[cfg(target_family = "wasm")]
fn main() {
    gpui_platform::web_init();
    let _application = gpui_platform::application().run_embedded(launch);
    std::mem::forget(_application)
}
