use gpui::{div, prelude::*, px, App, Bounds, WindowBounds, WindowOptions};
use gpui_command_palette::{Command, CommandPalette, Modifier};
fn launch(cx: &mut App) {
    gpui_command_palette::init(cx);
    let bounds = Bounds::centered(None, gpui::size(px(900.), px(600.)), cx);
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        },
        |_, cx| {
            let palette = cx.new(CommandPalette::new);
            let registrations = palette.read(cx).registry().register_many([
                Command::new("file.open", "Open File", || {})
                    .description("Open a file from disk")
                    .group("File")
                    .shortcut(vec![Modifier::Main], "o"),
                Command::submenu("theme", "Change Theme", || {
                    vec![
                        Command::new("theme.dark", "Dark", || {}),
                        Command::new("theme.light", "Light", || {}),
                    ]
                })
                .searchable_children(),
            ]);
            for registration in registrations {
                registration.forget()
            }
            cx.new(|_| Demo { palette })
        },
    )
    .unwrap();
    cx.activate(true);
    #[cfg(target_family = "wasm")]
    cx.refresh_windows();
}
struct Demo {
    palette: gpui::Entity<CommandPalette>,
}
impl gpui::Render for Demo {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
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
