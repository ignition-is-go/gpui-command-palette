# gpui-command-palette

A Rust-first command palette for [Zed GPUI](https://github.com/zed-industries/zed), pinned to `08827f9`. The identical `CommandPalette` render tree is used on native targets and WebAssembly; browser hosts must use GPUI's document-owned canvas.

```rust
let palette = cx.new(CommandPalette::new);
let registration = palette.read(cx).registry().register(
    Command::new("file.open", "Open File", open).shortcut(vec![Modifier::Main], "o")
);
```

`CommandRegistry` preserves registration order, replaces duplicate IDs at the end, and unregisters dynamically when `Registration` drops. `PaletteState` and `search_commands` are renderer-independent and tested directly.

## Theming

`CommandPalette` implements and re-exports `gpui_styling::ThemeHost`. Use `with_theme` or `set_theme` for a fixed `CommandPaletteTheme`, and `with_theme_provider` or `set_theme_provider` for a live application-derived theme. A provider temporarily overrides the remembered fixed theme; clearing it with `set_theme_provider(None, cx)` restores that fixed theme. The complete theme is resolved once per root render and passed to retained input elements as one consistent snapshot.

## Leptos reference mapping

| Leptos API/behavior | GPUI API |
|---|---|
| `Command`, submenu, searchable children | source-compatible `Send + Sync` builders; typed metadata and GPUI execution hooks added |
| provider context/register lifecycle | `CommandRegistry` plus RAII `Registration` |
| Cmd/Ctrl+K and command shortcuts | GPUI actions/key bindings and `Shortcut::matches` |
| reactive open/query/selection/navigation | `PaletteState` |
| DOM modal | one GPUI modal render tree, focus capture/restoration, backdrop dismissal |
| CSS default theme | one `CommandPaletteTheme` composed from typed panel, backdrop, input, item, and empty-state styles with matching defaults |

## Known platform gaps

The widget emits GPUI accessibility roles and labels for its dialog, search input, list box, and selected options. Pinned GPUI does not expose an explicit `aria-modal` property. The reference has no transition, so the GPUI widget likewise appears immediately. Text entry uses GPUI's entity input handler for native and browser text/IME events. `CommandPalettePosition::Custom` uses typed pixel/fraction lengths and pixel translations because arbitrary CSS strings are not meaningful in a native renderer.

## Production proof

`.github/workflows/production.yml` is the release-grade proof for this shared widget. Every push and pull request runs build, tests, formatting, and warning-denied Clippy on Linux, macOS, and Windows. Linux additionally compiles the enabled Wayland and X11 backends and keeps the native demo healthy inside a nested headless Weston compositor.

The web job uses nightly Rust and Trunk to make a release wasm build from `examples/web/main.rs`, which includes the native `examples/demo.rs` rather than maintaining a browser copy. Real Google Chrome runs under Xvfb with SwiftShader at a fixed 900×600 viewport. Chrome DevTools Protocol reads a wasm-only bridge derived from the real palette entity to prove open, query, selection, results, submenu depth, and execution state while Playwright sends real keyboard and pointer input for Ctrl/Cmd+K, query, arrows, submenu enter/back, confirmation, Escape, and backdrop dismissal. The test also verifies that GPUI's sole document-owned canvas retains its identity throughout the flow and uploads a non-blank fixed-viewport screenshot.

The demo's wasm-only `window.name` value is deliberately a narrow read-only bridge derived from the palette entity. Its execution field comes from the actual portable command callback, so CI can distinguish command confirmation from merely closing the dialog without introducing duplicate state that drives the UI.
