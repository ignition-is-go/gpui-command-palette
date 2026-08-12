use gpui::{point, px, rgb, rgba, App, BoxShadow, Global, Hsla, Pixels, TextAlign};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaletteLength {
    Pixels(Pixels),
    Fraction(f32),
}
impl PaletteLength {
    pub fn px(value: f32) -> Self {
        Self::Pixels(px(value))
    }
    pub const fn percent(value: f32) -> Self {
        Self::Fraction(value / 100.0)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaletteTransform {
    pub x: Pixels,
    pub y: Pixels,
}
impl PaletteTransform {
    pub fn pixels(x: f32, y: f32) -> Self {
        Self { x: px(x), y: px(y) }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CommandPalettePosition {
    #[default]
    TopCenter,
    Center,
    Custom {
        top: Option<PaletteLength>,
        right: Option<PaletteLength>,
        bottom: Option<PaletteLength>,
        left: Option<PaletteLength>,
        transform: Option<PaletteTransform>,
    },
}
#[derive(Clone, Copy, Debug)]
pub struct CommandPaletteBackdropStyle {
    pub background: Hsla,
    pub z_index: u32,
}
impl Default for CommandPaletteBackdropStyle {
    fn default() -> Self {
        Self {
            background: rgba(0x00000080).into(),
            z_index: 9999,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CommandPalettePanelStyle {
    pub background: Hsla,
    pub color: Hsla,
    pub border: Hsla,
    pub border_width: Pixels,
    pub border_radius: Pixels,
    pub width: Pixels,
    pub max_height: Pixels,
    pub shadow: BoxShadow,
    pub font_size: Pixels,
    pub padding: Pixels,
}
impl Default for CommandPalettePanelStyle {
    fn default() -> Self {
        Self {
            background: rgb(0x1e1e1e).into(),
            color: rgb(0xcccccc).into(),
            border: rgb(0x3c3c3c).into(),
            border_width: px(1.),
            border_radius: px(8.),
            width: px(500.),
            max_height: px(400.),
            shadow: BoxShadow {
                color: rgba(0x00000080).into(),
                offset: point(px(0.), px(8.)),
                blur_radius: px(30.),
                spread_radius: px(0.),
                inset: false,
            },
            font_size: px(14.),
            padding: px(8.),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CommandPaletteInputStyle {
    pub background: Hsla,
    pub color: Hsla,
    pub border: Hsla,
    pub border_width: Pixels,
    pub border_radius: Pixels,
    pub font_size: Pixels,
    pub padding_y: Pixels,
    pub padding_x: Pixels,
    pub placeholder_color: Hsla,
    pub margin_bottom: Pixels,
}
impl Default for CommandPaletteInputStyle {
    fn default() -> Self {
        Self {
            background: rgb(0x2a2a2a).into(),
            color: rgb(0xcccccc).into(),
            border: rgb(0x3c3c3c).into(),
            border_width: px(1.),
            border_radius: px(4.),
            font_size: px(14.),
            padding_y: px(8.),
            padding_x: px(12.),
            placeholder_color: rgb(0x666666).into(),
            margin_bottom: px(8.),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CommandPaletteItemStyle {
    pub padding_y: Pixels,
    pub padding_x: Pixels,
    pub border_radius: Pixels,
    pub selected_background: Hsla,
    pub selected_color: Hsla,
    pub description_color: Hsla,
    pub description_font_size: Pixels,
    pub description_margin_top: Pixels,
    pub shortcut_color: Hsla,
    pub shortcut_font_size: Pixels,
    pub shortcut_opacity: f32,
    pub shortcut_margin_left: Pixels,
}
impl Default for CommandPaletteItemStyle {
    fn default() -> Self {
        Self {
            padding_y: px(8.),
            padding_x: px(12.),
            border_radius: px(4.),
            selected_background: rgb(0x094771).into(),
            selected_color: rgb(0xffffff).into(),
            description_color: rgb(0x888888).into(),
            description_font_size: px(12.),
            description_margin_top: px(2.),
            shortcut_color: rgb(0x888888).into(),
            shortcut_font_size: px(12.),
            shortcut_opacity: 0.7,
            shortcut_margin_left: px(12.),
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct CommandPaletteTheme {
    pub palette: CommandPalettePanelStyle,
    pub backdrop: CommandPaletteBackdropStyle,
    pub input: CommandPaletteInputStyle,
    pub item: CommandPaletteItemStyle,
    pub empty: CommandPaletteEmptyStyle,
}
impl CommandPaletteTheme {
    pub fn with_panel_style(mut self, style: CommandPalettePanelStyle) -> Self {
        self.palette = style;
        self
    }
    pub fn with_backdrop_style(mut self, style: CommandPaletteBackdropStyle) -> Self {
        self.backdrop = style;
        self
    }
    pub fn with_input_style(mut self, style: CommandPaletteInputStyle) -> Self {
        self.input = style;
        self
    }
    pub fn with_item_style(mut self, style: CommandPaletteItemStyle) -> Self {
        self.item = style;
        self
    }
    pub fn with_empty_style(mut self, style: CommandPaletteEmptyStyle) -> Self {
        self.empty = style;
        self
    }
}

struct GlobalCommandPaletteTheme(Arc<CommandPaletteTheme>);
impl Global for GlobalCommandPaletteTheme {}

/// Install or replace the application-wide command-palette theme snapshot.
///
/// This does not refresh windows. Applications updating live themes should
/// install all crate theme globals, then call `App::refresh_windows` once.
pub fn set_command_palette_theme(cx: &mut App, theme: impl Into<Arc<CommandPaletteTheme>>) {
    cx.set_global(GlobalCommandPaletteTheme(theme.into()));
}

/// Access the explicitly installed application-wide command-palette theme.
pub trait ActiveCommandPaletteTheme {
    /// Return the active immutable theme snapshot.
    ///
    /// Panics when [`set_command_palette_theme`] has not been called.
    fn command_palette_theme(&self) -> &Arc<CommandPaletteTheme>;
}

impl ActiveCommandPaletteTheme for App {
    fn command_palette_theme(&self) -> &Arc<CommandPaletteTheme> {
        &self.global::<GlobalCommandPaletteTheme>().0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CommandPaletteEmptyStyle {
    pub padding: Pixels,
    pub text_align: TextAlign,
    pub color: Hsla,
    pub opacity: f32,
    pub font_size: Pixels,
}
impl Default for CommandPaletteEmptyStyle {
    fn default() -> Self {
        Self {
            padding: px(12.),
            text_align: TextAlign::Center,
            color: rgb(0xcccccc).into(),
            opacity: 0.5,
            font_size: px(14.),
        }
    }
}
