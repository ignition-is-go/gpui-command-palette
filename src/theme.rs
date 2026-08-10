use gpui::{px, rgb, rgba, Hsla, Pixels};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CommandPalettePosition {
    #[default]
    TopCenter,
    Center,
}
#[derive(Clone, Copy, Debug)]
pub struct CommandPaletteTheme {
    pub backdrop: Hsla,
    pub background: Hsla,
    pub color: Hsla,
    pub border: Hsla,
    pub selected_background: Hsla,
    pub selected_color: Hsla,
    pub input_background: Hsla,
    pub muted: Hsla,
    pub width: Pixels,
    pub max_height: Pixels,
    pub border_radius: Pixels,
    pub font_size: Pixels,
    pub padding: Pixels,
}
impl Default for CommandPaletteTheme {
    fn default() -> Self {
        Self {
            backdrop: rgba(0x00000080).into(),
            background: rgb(0x1e1e1e).into(),
            color: rgb(0xcccccc).into(),
            border: rgb(0x3c3c3c).into(),
            selected_background: rgb(0x094771).into(),
            selected_color: rgb(0xffffff).into(),
            input_background: rgb(0x2a2a2a).into(),
            muted: rgb(0x888888).into(),
            width: px(500.),
            max_height: px(400.),
            border_radius: px(8.),
            font_size: px(14.),
            padding: px(8.),
        }
    }
}
