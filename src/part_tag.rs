//! Names for the palette's parts, for a harness that measures where each part
//! lands on screen.
//!
//! The palette records nothing itself. An application that wants the parts
//! named installs a [`PartTagger`] with [`set_part_tagger`]; the palette then
//! passes each part through it with a stable id. With no tagger installed the
//! parts render untouched and no id string is built.
//!
//! | id | part |
//! | --- | --- |
//! | `command-palette` | the panel |
//! | `command-palette.input` | the query input |
//! | `command-palette.row[n]` | the `n`th result row, counting from 0 |
//! | `command-palette.group[n]` | the `n`th group heading, counting from 0 |

use gpui::{AnyElement, App, Global, IntoElement, SharedString};

/// Wraps a part's element with its id. It must not change the part's layout.
pub type PartTagger = fn(AnyElement, SharedString) -> AnyElement;

struct GlobalPartTagger(PartTagger);
impl Global for GlobalPartTagger {}

/// Installs the tagger every palette passes its parts through.
pub fn set_part_tagger(cx: &mut App, tagger: PartTagger) {
    cx.set_global(GlobalPartTagger(tagger));
}

/// Passes `element` through the installed tagger as `id`.
pub(crate) fn tag_part(
    cx: &App,
    element: impl IntoElement,
    id: impl FnOnce() -> SharedString,
) -> AnyElement {
    let element = element.into_any_element();
    match cx.try_global::<GlobalPartTagger>() {
        Some(tagger) => (tagger.0)(element, id()),
        None => element,
    }
}
