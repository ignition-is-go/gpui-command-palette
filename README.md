# gpui-command-palette

A Rust-first command palette for [Zed GPUI](https://github.com/zed-industries/zed), pinned to `08827f9`. The identical `CommandPalette` render tree is used on native targets and WebAssembly; browser hosts must use GPUI's document-owned canvas.

```rust
let palette = cx.new(CommandPalette::new);
let registration = palette.read(cx).registry().register(
    Command::new("file.open", "Open File", open).shortcut(vec![Modifier::Main], "o")
);
```

`CommandRegistry` preserves registration order, replaces duplicate IDs at the end, and unregisters dynamically when `Registration` drops. `PaletteState` and `search_commands` are renderer-independent and tested directly.

## Leptos reference mapping

| Leptos API/behavior | GPUI API |
|---|---|
| `Command`, submenu, searchable children | source-compatible `Send + Sync` builders; typed metadata and GPUI execution hooks added |
| provider context/register lifecycle | `CommandRegistry` plus RAII `Registration` |
| Cmd/Ctrl+K and command shortcuts | GPUI actions/key bindings and `Shortcut::matches` |
| reactive open/query/selection/navigation | `PaletteState` |
| DOM modal | one GPUI modal render tree, focus capture/restoration, backdrop dismissal |
| CSS default theme | typed panel, backdrop, input, item, and empty-state themes with matching defaults |

## Known platform gaps

The widget emits GPUI accessibility roles and labels for its dialog, search input, list box, and selected options. Pinned GPUI does not expose an explicit `aria-modal` property. The reference has no transition, so the GPUI widget likewise appears immediately. Text entry uses GPUI's entity input handler for native and browser text/IME events. `CommandPalettePosition::Custom` uses typed pixel/fraction lengths and pixel translations because arbitrary CSS strings are not meaningful in a native renderer.
