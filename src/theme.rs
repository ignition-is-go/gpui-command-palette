use gpui::{point, px, rgb, rgba, BoxShadow, Hsla, Pixels, TextAlign};

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
pub struct CommandPaletteBackdropTheme {
    pub background: Hsla,
    pub z_index: u32,
}
impl Default for CommandPaletteBackdropTheme {
    fn default() -> Self {
        Self {
            background: rgba(0x00000080).into(),
            z_index: 9999,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CommandPaletteTheme {
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
impl Default for CommandPaletteTheme {
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
pub struct CommandPaletteInputTheme {
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
impl Default for CommandPaletteInputTheme {
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
pub struct CommandPaletteItemTheme {
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
impl Default for CommandPaletteItemTheme {
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
pub struct CommandPaletteStyles {
    pub palette: CommandPaletteTheme,
    pub backdrop: CommandPaletteBackdropTheme,
    pub input: CommandPaletteInputTheme,
    pub item: CommandPaletteItemTheme,
    pub empty: CommandPaletteEmptyTheme,
}
#[derive(Clone, Copy, Debug)]
pub struct CommandPaletteEmptyTheme {
    pub padding: Pixels,
    pub text_align: TextAlign,
    pub color: Hsla,
    pub opacity: f32,
    pub font_size: Pixels,
}
impl Default for CommandPaletteEmptyTheme {
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
