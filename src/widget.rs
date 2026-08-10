use crate::{
    CommandPaletteBackdropTheme, CommandPaletteEmptyTheme, CommandPaletteInputTheme,
    CommandPaletteItemTheme, CommandPalettePosition, CommandPaletteTheme, CommandRegistry,
    PaletteLength, PaletteState,
};
use gpui::{
    actions, div, fill, point, prelude::*, px, relative, size, App, Bounds, Context, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, GlobalElementId,
    InspectorElementId, InteractiveElement, KeyDownEvent, LayoutId, PaintQuad, Pixels, Render,
    ShapedLine, SharedString, Style, TextAlign, TextRun, UTF16Selection, UnderlineStyle, Window,
};
use std::ops::Range;
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
type ExecuteHandler<M> = std::rc::Rc<dyn Fn(&M, &mut Window, &mut App)>;

pub struct CommandPalette<M: 'static = ()> {
    registry: CommandRegistry<M>,
    state: PaletteState<M>,
    theme: CommandPaletteTheme,
    backdrop_theme: CommandPaletteBackdropTheme,
    input_theme: CommandPaletteInputTheme,
    item_theme: CommandPaletteItemTheme,
    empty_theme: CommandPaletteEmptyTheme,
    position: CommandPalettePosition,
    focus: FocusHandle,
    restore_focus: Option<FocusHandle>,
    on_execute: Option<ExecuteHandler<M>>,
    selected_range: Range<usize>,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_input_bounds: Option<Bounds<Pixels>>,
    _keystroke_subscription: gpui::Subscription,
}
impl<M: Clone + 'static> CommandPalette<M> {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe_keystrokes(|this, event, window, cx| {
            if this.state.is_open() {
                return;
            }
            if let Some(command) = this.registry.commands().into_iter().find(|command| {
                command
                    .shortcut
                    .as_ref()
                    .is_some_and(|shortcut| shortcut.matches(&event.keystroke))
            }) {
                command.execute_in(window, cx);
                if let Some(handler) = &this.on_execute {
                    handler(&command.metadata, window, cx);
                }
            }
        });
        Self {
            registry: CommandRegistry::new(),
            state: PaletteState::new(),
            theme: Default::default(),
            backdrop_theme: Default::default(),
            input_theme: Default::default(),
            item_theme: Default::default(),
            empty_theme: Default::default(),
            position: Default::default(),
            focus: cx.focus_handle(),
            restore_focus: None,
            on_execute: None,
            selected_range: 0..0,
            marked_range: None,
            last_layout: None,
            last_input_bounds: None,
            _keystroke_subscription: subscription,
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
    /// Replace the current query and synchronize the input cursor and result selection.
    pub fn set_query(&mut self, query: impl Into<String>, cx: &mut Context<Self>) {
        self.state.set_query(query);
        let end = self.state.query().len();
        self.selected_range = end..end;
        self.marked_range = None;
        cx.notify();
    }
    /// Clear the current query and synchronize the input cursor and result selection.
    pub fn clear_query(&mut self, cx: &mut Context<Self>) {
        self.set_query(String::new(), cx);
    }
    pub fn with_on_execute(
        mut self,
        handler: impl Fn(&M, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_execute = Some(std::rc::Rc::new(handler));
        self
    }
    pub fn with_theme(mut self, theme: CommandPaletteTheme) -> Self {
        self.theme = theme;
        self
    }
    pub fn with_backdrop_theme(mut self, theme: CommandPaletteBackdropTheme) -> Self {
        self.backdrop_theme = theme;
        self
    }
    pub fn with_input_theme(mut self, theme: CommandPaletteInputTheme) -> Self {
        self.input_theme = theme;
        self
    }
    pub fn with_item_theme(mut self, theme: CommandPaletteItemTheme) -> Self {
        self.item_theme = theme;
        self
    }
    pub fn with_empty_theme(mut self, theme: CommandPaletteEmptyTheme) -> Self {
        self.empty_theme = theme;
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
        self.reset_input_selection();
        self.focus.focus(window, cx);
        cx.notify()
    }
    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.close();
        self.reset_input_selection();
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
    fn reset_input_selection(&mut self) {
        self.selected_range = 0..0;
        self.marked_range = None;
    }
    fn cursor_offset(&self) -> usize {
        self.selected_range.end
    }
    fn previous_boundary(&self, offset: usize) -> usize {
        self.state.query()[..offset]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
    fn next_boundary(&self, offset: usize) -> usize {
        self.state.query()[offset..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| offset + i)
            .unwrap_or(self.state.query().len())
    }
    fn replace_query_range(&mut self, range: Range<usize>, text: &str) {
        let mut query = self.state.query().to_owned();
        query.replace_range(range.clone(), text);
        let at = range.start + text.len();
        self.state.set_query(query);
        self.selected_range = at..at;
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
                if let Some(handler) = &self.on_execute {
                    handler(&command.metadata, window, cx);
                }
                self.close(window, cx)
            } else {
                self.reset_input_selection();
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
                    self.reset_input_selection();
                    cx.notify()
                } else {
                    self.close(window, cx)
                }
            }
            "arrowdown" | "down" => {
                let n = self.state.results(&self.registry.commands()).len();
                self.state.select_next(n);
                cx.notify()
            }
            "arrowup" | "up" => {
                self.state.select_previous();
                cx.notify()
            }
            "enter" => self.confirm(window, cx),
            "backspace" => {
                if !self.selected_range.is_empty() {
                    self.replace_query_range(self.selected_range.clone(), "");
                } else if self.state.query().is_empty() {
                    self.state.backspace();
                    self.reset_input_selection();
                } else {
                    let end = self.cursor_offset();
                    let start = self.previous_boundary(end);
                    self.replace_query_range(start..end, "");
                }
                cx.notify()
            }
            "delete" => {
                if !self.selected_range.is_empty() {
                    self.replace_query_range(self.selected_range.clone(), "");
                } else {
                    let start = self.cursor_offset();
                    let end = self.next_boundary(start);
                    self.replace_query_range(start..end, "");
                }
                cx.notify();
            }
            "arrowleft" | "left" => {
                let at = if self.selected_range.is_empty() {
                    self.previous_boundary(self.cursor_offset())
                } else {
                    self.selected_range.start
                };
                self.selected_range = at..at;
                cx.notify();
            }
            "arrowright" | "right" => {
                let at = if self.selected_range.is_empty() {
                    self.next_boundary(self.cursor_offset())
                } else {
                    self.selected_range.end
                };
                self.selected_range = at..at;
                cx.notify();
            }
            "home" => {
                self.selected_range = 0..0;
                cx.notify();
            }
            "end" => {
                let end = self.state.query().len();
                self.selected_range = end..end;
                cx.notify();
            }
            "a" if event.keystroke.modifiers.platform || event.keystroke.modifiers.control => {
                self.selected_range = 0..self.state.query().len();
                cx.notify();
            }
            "tab" => {
                self.focus.focus(window, cx);
                cx.stop_propagation();
            }
            _ => {}
        }
    }
}
impl<M: Clone + 'static> Render for CommandPalette<M> {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let base = div().id("command-palette-provider");
        if !self.state.is_open() {
            return base;
        }
        let theme = self.theme.clone();
        let input_theme = self.input_theme;
        let item_theme = self.item_theme;
        let empty_theme = self.empty_theme;
        let commands = self.registry.commands();
        let results = self.state.results(&commands);
        self.state.clamp_selection(results.len());
        let selected = self.state.selected_index();
        let mut list = div()
            .id("command-palette-results")
            .role(gpui::Role::ListBox)
            .aria_label("Commands")
            .flex_1()
            .overflow_y_scroll();
        if results.is_empty() {
            list = list.child(
                div()
                    .p(empty_theme.padding)
                    .text_align(empty_theme.text_align)
                    .text_size(empty_theme.font_size)
                    .text_color(empty_theme.color.opacity(empty_theme.opacity))
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
                .role(gpui::Role::ListBoxOption)
                .aria_selected(index == selected)
                .flex()
                .justify_between()
                .items_center()
                .px(item_theme.padding_x)
                .py(item_theme.padding_y)
                .rounded(item_theme.border_radius)
                .cursor_pointer()
                .when(index == selected, |row| {
                    row.bg(item_theme.selected_background)
                        .text_color(item_theme.selected_color)
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
                            if let Some(handler) = &this.on_execute {
                                handler(&command.metadata, window, cx);
                            }
                            this.close(window, cx)
                        } else {
                            this.reset_input_selection();
                            cx.notify()
                        }
                    }
                    cx.stop_propagation()
                }))
                .child(div().child(command.name).when_some(description, |x, d| {
                    x.child(
                        div()
                            .mt(item_theme.description_margin_top)
                            .text_size(item_theme.description_font_size)
                            .text_color(item_theme.description_color)
                            .child(d),
                    )
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .text_size(item_theme.shortcut_font_size)
                        .text_color(
                            item_theme
                                .shortcut_color
                                .opacity(item_theme.shortcut_opacity),
                        )
                        .when_some(shortcut, |x, s| x.child(s))
                        .when(branch, |x| {
                            x.ml(item_theme.shortcut_margin_left)
                                .child(div().text_size(px(16.)).child("›"))
                        }),
                );
            list = list.child(row)
        }
        let navigation = self.state.navigation().to_vec();
        let mut breadcrumbs = div()
            .flex()
            .items_center()
            .flex_wrap()
            .gap(px(4.))
            .mb(px(8.))
            .text_size(px(13.));
        if !navigation.is_empty() {
            breadcrumbs = breadcrumbs.child(
                div()
                    .id("command-palette-breadcrumb-back")
                    .cursor_pointer()
                    .pr(px(2.))
                    .opacity(0.7)
                    .child("‹")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.state.back();
                        this.reset_input_selection();
                        cx.notify();
                        cx.stop_propagation();
                    })),
            );
            for (index, level) in navigation.into_iter().enumerate() {
                let target = index + 1;
                breadcrumbs = breadcrumbs
                    .child(
                        div()
                            .id(("command-palette-breadcrumb", index))
                            .cursor_pointer()
                            .child(level.label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.state.pop_to(target);
                                this.reset_input_selection();
                                cx.notify();
                                cx.stop_propagation();
                            })),
                    )
                    .child(div().opacity(0.5).child("›"));
            }
        }
        let mut panel = div()
            .id("command-palette-dialog")
            .role(gpui::Role::Dialog)
            .aria_label("Command palette")
            .debug_selector(|| "command-palette-dialog".into())
            .track_focus(&self.focus)
            .key_context(KEY_CONTEXT)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.to_ascii_lowercase();
                let handled = matches!(
                    key.as_str(),
                    "escape"
                        | "arrowdown"
                        | "down"
                        | "arrowup"
                        | "up"
                        | "enter"
                        | "backspace"
                        | "delete"
                        | "arrowleft"
                        | "left"
                        | "arrowright"
                        | "right"
                        | "home"
                        | "end"
                        | "tab"
                ) || (key == "a"
                    && (event.keystroke.modifiers.platform || event.keystroke.modifiers.control));
                this.key_down(event, window, cx);
                if handled {
                    cx.stop_propagation();
                }
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
            .overflow_hidden()
            .p(theme.padding)
            .flex()
            .flex_col()
            .rounded(theme.border_radius)
            .border(theme.border_width)
            .border_color(theme.border)
            .bg(theme.background)
            .text_color(theme.color)
            .text_size(theme.font_size)
            .shadow(vec![theme.shadow.clone()])
            .when(self.state.depth() > 0, |x| x.child(breadcrumbs))
            .child(
                div()
                    .id("command-palette-input")
                    .role(gpui::Role::TextInput)
                    .aria_label("Search commands")
                    .aria_placeholder("Type a command...")
                    .aria_value(self.state.query().to_owned())
                    .w_full()
                    .px(input_theme.padding_x)
                    .py(input_theme.padding_y)
                    .mb(input_theme.margin_bottom)
                    .rounded(input_theme.border_radius)
                    .border(input_theme.border_width)
                    .border_color(input_theme.border)
                    .bg(input_theme.background)
                    .text_size(input_theme.font_size)
                    .text_color(if self.state.query().is_empty() {
                        input_theme.placeholder_color
                    } else {
                        input_theme.color
                    })
                    .child(PaletteInputElement {
                        palette: cx.entity(),
                    }),
            )
            .child(list)
            .on_click(|_, _, cx| cx.stop_propagation());
        if let CommandPalettePosition::Custom {
            top,
            right,
            bottom,
            left,
            transform,
        } = self.position
        {
            panel = panel.absolute();
            panel = match top {
                Some(PaletteLength::Pixels(v)) => panel.top(v),
                Some(PaletteLength::Fraction(v)) => panel.top(relative(v)),
                None => panel,
            };
            panel = match right {
                Some(PaletteLength::Pixels(v)) => panel.right(v),
                Some(PaletteLength::Fraction(v)) => panel.right(relative(v)),
                None => panel,
            };
            panel = match bottom {
                Some(PaletteLength::Pixels(v)) => panel.bottom(v),
                Some(PaletteLength::Fraction(v)) => panel.bottom(relative(v)),
                None => panel,
            };
            panel = match left {
                Some(PaletteLength::Pixels(v)) => panel.left(v),
                Some(PaletteLength::Fraction(v)) => panel.left(relative(v)),
                None => panel,
            };
            if let Some(transform) = transform {
                panel = panel.ml(transform.x).mt(transform.y);
            }
        }
        let backdrop = base
            .absolute()
            .size_full()
            .top_0()
            .left_0()
            .bg(self.backdrop_theme.background)
            .flex()
            .when(
                !matches!(self.position, CommandPalettePosition::Custom { .. }),
                |x| x.justify_center(),
            )
            .when(
                matches!(self.position, CommandPalettePosition::TopCenter),
                |x| x.pt(relative(0.2)),
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

fn utf16_to_utf8_offset(text: &str, offset: usize) -> usize {
    let mut utf8_offset = 0;
    let mut utf16_offset = 0;
    for ch in text.chars() {
        if utf16_offset >= offset {
            break;
        }
        utf16_offset += ch.len_utf16();
        utf8_offset += ch.len_utf8();
    }
    utf8_offset
}

impl<M: Clone + 'static> CommandPalette<M> {
    fn utf16_to_utf8(&self, offset: usize) -> usize {
        utf16_to_utf8_offset(self.state.query(), offset)
    }
    fn utf8_to_utf16(&self, offset: usize) -> usize {
        self.state.query()[..offset].encode_utf16().count()
    }
    fn utf16_range_to_utf8(&self, range: Range<usize>) -> Range<usize> {
        self.utf16_to_utf8(range.start)..self.utf16_to_utf8(range.end)
    }
    fn to_utf16(&self, range: Range<usize>) -> Range<usize> {
        self.utf8_to_utf16(range.start)..self.utf8_to_utf16(range.end)
    }
}
impl<M: Clone + 'static> EntityInputHandler for CommandPalette<M> {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.utf16_range_to_utf8(range);
        *adjusted = Some(self.to_utf16(range.clone()));
        Some(self.state.query()[range].to_owned())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.to_utf16(self.selected_range.clone()),
            reversed: false,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.clone().map(|r| self.to_utf16(r))
    }
    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked_range = None;
    }
    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range
            .map(|r| self.utf16_range_to_utf8(r))
            .or_else(|| self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        self.replace_query_range(range, &text.replace(['\n', '\r'], " "));
        self.marked_range = None;
        cx.notify();
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range
            .map(|r| self.utf16_range_to_utf8(r))
            .or_else(|| self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let start = range.start;
        self.replace_query_range(range, text);
        self.marked_range = (!text.is_empty()).then_some(start..start + text.len());
        if let Some(selected) = selected {
            let selected = utf16_to_utf8_offset(text, selected.start)
                ..utf16_to_utf8_offset(text, selected.end);
            self.selected_range = start + selected.start..start + selected.end;
        }
        cx.notify();
    }
    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        let range = self.utf16_range_to_utf8(range);
        Some(Bounds::from_corners(
            point(bounds.left() + line.x_for_index(range.start), bounds.top()),
            point(bounds.left() + line.x_for_index(range.end), bounds.bottom()),
        ))
    }
    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_input_bounds?;
        let line = self.last_layout.as_ref()?;
        let index = line.closest_index_for_x(point.x - bounds.left());
        Some(self.utf8_to_utf16(index))
    }
    fn set_selected_text_range(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_range = self.utf16_range_to_utf8(range);
        cx.notify();
    }
    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.state.query().encode_utf16().count())
    }
}
struct PaletteInputElement<M: 'static> {
    palette: Entity<CommandPalette<M>>,
}
struct PaletteInputPrepaint {
    line: ShapedLine,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}
impl<M: Clone + 'static> IntoElement for PaletteInputElement<M> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl<M: Clone + 'static> Element for PaletteInputElement<M> {
    type RequestLayoutState = ();
    type PrepaintState = PaletteInputPrepaint;
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> PaletteInputPrepaint {
        let palette = self.palette.read(cx);
        let empty = palette.state.query().is_empty();
        let text: SharedString = if empty {
            "Type a command...".into()
        } else {
            palette.state.query().to_owned().into()
        };
        let cursor = palette.cursor_offset();
        let theme = palette.input_theme;
        let base = TextRun {
            len: text.len(),
            font: window.text_style().font(),
            color: if empty {
                theme.placeholder_color
            } else {
                theme.color
            },
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(mark) = palette.marked_range.clone() {
            vec![
                TextRun {
                    len: mark.start,
                    ..base.clone()
                },
                TextRun {
                    len: mark.end - mark.start,
                    underline: Some(UnderlineStyle {
                        color: Some(base.color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..base.clone()
                },
                TextRun {
                    len: text.len() - mark.end,
                    ..base
                },
            ]
            .into_iter()
            .filter(|r| r.len > 0)
            .collect()
        } else {
            vec![base]
        };
        let line = window
            .text_system()
            .shape_line(text, theme.font_size, &runs, None);
        let focused = palette.focus.is_focused(window);
        let (cursor, selection) = if focused && palette.selected_range.is_empty() {
            (
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + line.x_for_index(cursor), bounds.top()),
                        size(px(1.), bounds.size.height),
                    ),
                    theme.color,
                )),
                None,
            )
        } else if focused {
            (
                None,
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(palette.selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(palette.selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    palette.item_theme.selected_background.opacity(0.5),
                )),
            )
        } else {
            (None, None)
        };
        PaletteInputPrepaint {
            line,
            cursor,
            selection,
        }
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        state: &mut PaletteInputPrepaint,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus = self.palette.read(cx).focus.clone();
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.palette.clone()),
            cx,
        );
        if let Some(selection) = state.selection.take() {
            window.paint_quad(selection);
        }
        state
            .line
            .paint(
                bounds.origin,
                window.line_height(),
                TextAlign::Left,
                None,
                window,
                cx,
            )
            .unwrap();
        if let Some(cursor) = state.cursor.take() {
            window.paint_quad(cursor)
        }
        let line = state.line.clone();
        self.palette.update(cx, |p, _| {
            p.last_layout = Some(line);
            p.last_input_bounds = Some(bounds)
        });
    }
}

#[cfg(test)]
mod input_tests {
    use super::utf16_to_utf8_offset;

    #[test]
    fn utf16_offsets_handle_surrogate_pairs_and_relative_composition_ranges() {
        let text = "a😀é";
        assert_eq!(utf16_to_utf8_offset(text, 0), 0);
        assert_eq!(utf16_to_utf8_offset(text, 1), 1);
        assert_eq!(utf16_to_utf8_offset(text, 2), 5);
        assert_eq!(utf16_to_utf8_offset(text, 3), 5);
        assert_eq!(utf16_to_utf8_offset(text, 4), text.len());
        assert_eq!(utf16_to_utf8_offset("😀x", 2), 4);
    }
}
