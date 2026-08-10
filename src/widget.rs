use crate::{CommandPalettePosition, CommandPaletteTheme, CommandRegistry, PaletteState};
use gpui::{
    actions, div, prelude::*, px, App, Context, FocusHandle, KeyBinding, KeyDownEvent, Render,
    Window,
};
actions!(
    command_palette,
    [
        ToggleCommandPalette,
        OpenCommandPalette,
        CloseCommandPalette,
        SelectNextCommand,
        SelectPreviousCommand,
        ConfirmCommand
    ]
);
pub const KEY_CONTEXT: &str = "CommandPalette";
pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-k", ToggleCommandPalette, None),
        KeyBinding::new("ctrl-k", ToggleCommandPalette, None),
    ]);
}

pub struct CommandPalette<M: 'static = ()> {
    registry: CommandRegistry<M>,
    state: PaletteState<M>,
    theme: CommandPaletteTheme,
    position: CommandPalettePosition,
    focus: FocusHandle,
    restore_focus: Option<FocusHandle>,
}
impl<M: Clone + 'static> CommandPalette<M> {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            registry: CommandRegistry::new(),
            state: PaletteState::new(),
            theme: Default::default(),
            position: Default::default(),
            focus: cx.focus_handle(),
            restore_focus: None,
        }
    }
    pub fn with_registry(mut self, registry: CommandRegistry<M>) -> Self {
        self.registry = registry;
        self
    }
    pub fn registry(&self) -> &CommandRegistry<M> {
        &self.registry
    }
    pub fn state(&self) -> &PaletteState<M> {
        &self.state
    }
    pub fn with_theme(mut self, theme: CommandPaletteTheme) -> Self {
        self.theme = theme;
        self
    }
    pub fn with_position(mut self, position: CommandPalettePosition) -> Self {
        self.position = position;
        self
    }
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.state.is_open() {
            self.restore_focus = window.focused(cx)
        }
        self.state.open();
        self.focus.focus(window, cx);
        cx.notify()
    }
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.close();
        if let Some(focus) = self.restore_focus.take() {
            focus.focus(window, cx)
        }
        cx.notify()
    }
    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.is_open() {
            self.close(window, cx)
        } else {
            self.open(window, cx)
        }
    }
    fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let commands = self.registry.commands();
        let results = self.state.results(&commands);
        if let Some(command) = results
            .get(self.state.selected_index())
            .map(|r| r.entry.clone())
        {
            if !self.state.enter(&command) {
                command.execute_in(window, cx);
                self.close(window, cx)
            } else {
                cx.notify()
            }
        }
    }
    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.to_ascii_lowercase();
        match key.as_str() {
            "escape" => {
                if self.state.depth() > 0 {
                    self.state.back();
                    cx.notify()
                } else {
                    self.close(window, cx)
                }
            }
            "arrowdown" => {
                let n = self.state.results(&self.registry.commands()).len();
                self.state.select_next(n);
                cx.notify()
            }
            "arrowup" => {
                self.state.select_previous();
                cx.notify()
            }
            "enter" => self.confirm(window, cx),
            "backspace" => {
                self.state.backspace();
                cx.notify()
            }
            _ => {
                if !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                {
                    if let Some(text) = event
                        .keystroke
                        .key_char
                        .as_deref()
                        .filter(|s| !s.chars().any(char::is_control))
                    {
                        self.state.push_text(text);
                        cx.notify()
                    }
                }
            }
        }
    }
}
impl<M: Clone + 'static> Render for CommandPalette<M> {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let base = div()
            .id("command-palette-provider")
            .on_action(cx.listener(|this, _: &ToggleCommandPalette, w, cx| this.toggle(w, cx)))
            .on_action(cx.listener(|this, _: &OpenCommandPalette, w, cx| this.open(w, cx)))
            .on_action(cx.listener(|this, _: &CloseCommandPalette, w, cx| this.close(w, cx)));
        if !self.state.is_open() {
            return base;
        }
        let theme = self.theme;
        let commands = self.registry.commands();
        let results = self.state.results(&commands);
        let selected = self.state.selected_index();
        let query = if self.state.query().is_empty() {
            "Type a command...".to_string()
        } else {
            self.state.query().to_string()
        };
        let mut list = div()
            .id("command-palette-results")
            .flex_1()
            .overflow_y_scroll();
        if results.is_empty() {
            list = list.child(
                div()
                    .p_3()
                    .text_center()
                    .text_color(theme.muted)
                    .child("No commands found"),
            );
        }
        for (index, result) in results.into_iter().enumerate() {
            let command = result.entry;
            let description = command.description.clone();
            let shortcut = command.shortcut.as_ref().map(ToString::to_string);
            let branch = command.is_branch();
            let row = div()
                .id(("command-palette-row", index))
                .flex()
                .justify_between()
                .items_center()
                .px_3()
                .py_2()
                .rounded(px(4.))
                .cursor_pointer()
                .when(index == selected, |row| {
                    row.bg(theme.selected_background)
                        .text_color(theme.selected_color)
                })
                .on_mouse_move(cx.listener(move |this, _, _, cx| {
                    let count = this.state.results(&this.registry.commands()).len();
                    this.state.select(index, count);
                    cx.notify()
                }))
                .on_click(cx.listener(move |this, _, window, cx| {
                    let commands = this.registry.commands();
                    if let Some(command) = this
                        .state
                        .results(&commands)
                        .get(index)
                        .map(|r| r.entry.clone())
                    {
                        if !this.state.enter(&command) {
                            command.execute_in(window, cx);
                            this.close(window, cx)
                        } else {
                            cx.notify()
                        }
                    }
                    cx.stop_propagation()
                }))
                .child(div().child(command.name).when_some(description, |x, d| {
                    x.child(
                        div()
                            .mt(px(2.))
                            .text_size(px(12.))
                            .text_color(theme.muted)
                            .child(d),
                    )
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .text_size(px(12.))
                        .text_color(theme.muted)
                        .when_some(shortcut, |x, s| x.child(s))
                        .when(branch, |x| x.ml_3().child("›")),
                );
            list = list.child(row)
        }
        let crumbs = self
            .state
            .navigation()
            .iter()
            .map(|level| level.label.clone())
            .collect::<Vec<_>>()
            .join(" › ");
        let panel = div()
            .id("command-palette-dialog")
            .debug_selector(|| "command-palette-dialog".into())
            .track_focus(&self.focus)
            .key_context(KEY_CONTEXT)
            .on_key_down(cx.listener(|this, event, window, cx| {
                this.key_down(event, window, cx);
                cx.stop_propagation()
            }))
            .on_action(cx.listener(|this, _: &SelectNextCommand, _, cx| {
                let n = this.state.results(&this.registry.commands()).len();
                this.state.select_next(n);
                cx.notify()
            }))
            .on_action(cx.listener(|this, _: &SelectPreviousCommand, _, cx| {
                this.state.select_previous();
                cx.notify()
            }))
            .on_action(cx.listener(|this, _: &ConfirmCommand, w, cx| this.confirm(w, cx)))
            .w(theme.width)
            .max_h(theme.max_height)
            .p(theme.padding)
            .flex()
            .flex_col()
            .rounded(theme.border_radius)
            .border_1()
            .border_color(theme.border)
            .bg(theme.background)
            .text_color(theme.color)
            .text_size(theme.font_size)
            .shadow_lg()
            .when(!crumbs.is_empty(), |x| {
                x.child(
                    div()
                        .mb_2()
                        .text_size(px(13.))
                        .child(format!("‹  {crumbs}")),
                )
            })
            .child(
                div()
                    .id("command-palette-input")
                    .w_full()
                    .px_3()
                    .py_2()
                    .mb_2()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.input_background)
                    .text_color(if self.state.query().is_empty() {
                        theme.muted
                    } else {
                        theme.color
                    })
                    .child(query),
            )
            .child(list)
            .on_click(|_, _, cx| cx.stop_propagation());
        let backdrop = base
            .absolute()
            .size_full()
            .top_0()
            .left_0()
            .bg(theme.backdrop)
            .flex()
            .justify_center()
            .when(
                matches!(self.position, CommandPalettePosition::TopCenter),
                |x| x.pt(px(80.)),
            )
            .when(
                matches!(self.position, CommandPalettePosition::Center),
                |x| x.items_center(),
            )
            .on_click(cx.listener(|this, _, w, cx| this.close(w, cx)))
            .child(panel);
        backdrop
    }
}
