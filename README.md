# gpui-command-palette

A Rust-first command palette for [Zed GPUI](https://github.com/zed-industries/zed), pinned to `08827f9`. The identical `CommandPalette` render tree is used on native targets and WebAssembly; browser hosts must use GPUI's document-owned canvas.

```rust
let palette = cx.new(CommandPalette::new);
let registration = palette.read(cx).registry().register(
    Command::new("file.open", "Open File", open).shortcut(vec![Modifier::Main], "o")
);
```

`CommandRegistry` preserves insertion order, updates registrations in place, and unregisters dynamically when `Registration` drops. `PaletteState` and `search_commands` are renderer-independent and tested directly.

## Leptos reference mapping

| Leptos API/behavior | GPUI API |
|---|---|
| `Command`, submenu, searchable children | source-compatible builders; GPUI-aware `with_handler`/`with_metadata` added |
| provider context/register lifecycle | `CommandRegistry` plus RAII `Registration` |
| Cmd/Ctrl+K and command shortcuts | GPUI actions/key bindings and `Shortcut::matches` |
| reactive open/query/selection/navigation | `PaletteState` |
| DOM modal | one GPUI modal render tree, focus capture/restoration, backdrop dismissal |
| CSS default theme | typed `CommandPaletteTheme`, same colors, 500×400 geometry, typography, padding and scrolling |

## Known platform gaps

The widget emits GPUI accessibility roles and labels for its dialog, search input, list box, and selected options. Pinned GPUI does not expose an explicit `aria-modal` property. GPUI also has no CSS transform or DOM transition API; positioning/appearance use GPUI layout/paint and the current release appears immediately. Text entry uses GPUI key events; full IME composition requires a host editor/input handler. `CommandPalettePosition::Custom` CSS strings cannot be source-compatible in a native renderer and is intentionally not exposed.
