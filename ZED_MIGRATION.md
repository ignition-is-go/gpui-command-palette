# Zed-style public visual API audit

This crate follows the ownership classes defined by Pulse GPUI's
`ZED_MIGRATION.md` and the GPUI patterns pinned at Zed `08827f9`.

| Public API | Class | Contract |
|---|---:|---|
| `CommandPaletteState<M>` | C | Explicit caller-owned `Entity`; required `CommandRegistry<M>` in `new`; implements `Render`, `Focusable`, `EventEmitter<CommandPaletteEvent<M>>`, and GPUI text input handling. |
| `Command`, `CommandRegistry`, `Registration`, `PaletteState`, search and shortcut types | D | Typed models, registry/lifetime, semantic algorithms, and platform adapters; not visual components. |
| `CommandPaletteTheme` and typed style/position values | D | Immutable ambient `Arc` snapshot installed through the private global; construction data, not provider components. |

There are no public ordinary controls in this focused crate, so there are no
class A/B function components or hidden semantic-state entities to retain.
Palette rows, breadcrumbs, input painting, and backdrop are private parts of the
single durable palette entity. Interactive element identity is derived from
stable command IDs, not result or breadcrumb indices.

## Migrated API

- `CommandPalette` became `CommandPaletteState`; no compatibility alias is kept.
- `CommandPaletteState::new(registry, cx)` makes registry ownership explicit;
  hidden default construction and `with_registry` were removed.
- `with_on_execute` was replaced by typed `CommandPaletteEvent::Executed`.
- `with_position` became the fluent `.position(...)` builder.
- Theme construction uses `.panel_style(...)`, `.backdrop_style(...)`,
  `.input_style(...)`, `.item_style(...)`, and `.empty_style(...)`.
- Arrow, confirm, dismiss, and global open/close/toggle behavior use structured
  `command_palette` GPUI actions and contextual key bindings. Raw key handling is
  limited to canonical text editing, IME selection, and focus trapping.
- The private ambient `Arc<CommandPaletteTheme>`, typed registry and events,
  accessibility roles, input handler, and identical native/WASM render path are
  preserved.
