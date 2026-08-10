//! Platform-aware keyboard shortcut normalization and display.
use gpui::{Keystroke, Modifiers};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Modifier {
    Main,
    Alt,
    Shift,
}

impl Modifier {
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Main => {
                if cfg!(target_os = "macos") {
                    "Cmd"
                } else {
                    "Ctrl"
                }
            }
            Self::Alt => {
                if cfg!(target_os = "macos") {
                    "Opt"
                } else {
                    "Alt"
                }
            }
            Self::Shift => "Shift",
        }
    }
}
impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub modifiers: Vec<Modifier>,
    pub key: String,
}
impl Shortcut {
    pub fn new(modifiers: Vec<Modifier>, key: impl Into<String>) -> Self {
        let mut normalized = Vec::with_capacity(3);
        for modifier in [Modifier::Main, Modifier::Alt, Modifier::Shift] {
            if modifiers.contains(&modifier) {
                normalized.push(modifier);
            }
        }
        Self {
            modifiers: normalized,
            key: normalize_key(&key.into()),
        }
    }
    pub fn matches(&self, stroke: &Keystroke) -> bool {
        let has = |modifier| self.modifiers.contains(&modifier);
        let Modifiers {
            control,
            alt,
            shift,
            platform,
            function: _,
        } = stroke.modifiers;
        control == (has(Modifier::Main) && !cfg!(target_os = "macos"))
            && platform == (has(Modifier::Main) && cfg!(target_os = "macos"))
            && alt == has(Modifier::Alt)
            && shift == has(Modifier::Shift)
            && normalize_key(&stroke.key).eq_ignore_ascii_case(&self.key)
    }
    pub fn gpui_binding(&self) -> String {
        let mut parts = self
            .modifiers
            .iter()
            .map(|m| match m {
                Modifier::Main => {
                    if cfg!(target_os = "macos") {
                        "cmd"
                    } else {
                        "ctrl"
                    }
                }
                Modifier::Alt => "alt",
                Modifier::Shift => "shift",
            })
            .collect::<Vec<_>>();
        parts.push(&self.key);
        parts.join("-")
    }
}
impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, modifier) in self.modifiers.iter().enumerate() {
            if index > 0 {
                f.write_str("+")?;
            }
            write!(f, "{modifier}")?;
        }
        if !self.modifiers.is_empty() {
            f.write_str("+")?;
        }
        f.write_str(&display_key(&self.key))
    }
}
pub fn normalize_key(key: &str) -> String {
    let value = key.trim();
    let value = value
        .strip_prefix("Key")
        .filter(|_| value.len() == 4)
        .or_else(|| value.strip_prefix("Digit").filter(|_| value.len() == 6))
        .unwrap_or(value);
    match value.to_ascii_lowercase().as_str() {
        "esc" => "escape".into(),
        "return" => "enter".into(),
        "spacebar" | " " => "space".into(),
        other => other.into(),
    }
}
fn display_key(key: &str) -> String {
    match key {
        "escape" => "Escape".into(),
        "enter" => "Enter".into(),
        "space" => "Space".into(),
        "arrowup" => "↑".into(),
        "arrowdown" => "↓".into(),
        "arrowleft" => "←".into(),
        "arrowright" => "→".into(),
        value => value.to_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_order_and_keys() {
        assert_eq!(
            Shortcut::new(
                vec![Modifier::Shift, Modifier::Main, Modifier::Main],
                "KeyK"
            )
            .to_string(),
            if cfg!(target_os = "macos") {
                "Cmd+Shift+K"
            } else {
                "Ctrl+Shift+K"
            }
        );
        assert_eq!(normalize_key("Esc"), "escape");
    }
}
