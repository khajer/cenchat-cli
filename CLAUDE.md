# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository

`cenchat-cli` is a standalone Rust crate (its own git repo, remote `git@github.com:khajer/cenchat-cli.git`) — the terminal WebSocket client half of the cenchat project. The server half (`cenchat-server`) lives in a sibling directory as a **separate** git repo; there is no Cargo workspace linking them. Build, run, and commit from within this directory only.

## Commands

```bash
cargo build
cargo run -- <ws-url>              # e.g. cargo run -- ws://127.0.0.1:9001
cargo check                        # fast type/borrow check without producing a binary
```

No test suite, lints, or CI config exist yet (no `#[test]`/`tests/`, no `clippy.toml`/`rustfmt.toml`, no `.github/workflows`).

## Architecture

The client is a `ratatui`/`crossterm` full-screen TUI driven by a single `tokio::select!` event loop, split across three modules:

- `src/main.rs` — parses the WS URL from `argv[1]`, connects via `tokio_tungstenite::connect_async` (printed to plain stdout before the TUI takes over the screen), then splits the stream into `write`/`read` halves. A spawned `read_task` loops on `read.next()` and forwards each `Message::Text`/`Message::Binary`/close/error as a `NetEvent` over an `mpsc` channel — it never touches the terminal directly. `ratatui::init()`/`ratatui::restore()` bracket the session (raw mode + alt screen), and the main loop `select!`s between that `NetEvent` channel and a `crossterm::event::EventStream` of key events, calling `terminal.draw` once per iteration. `handle_key` maps key codes to `App` mutations and, on Enter, sends the input line verbatim as a `Message::Text` (Esc/Ctrl+C set `app.should_quit` and the loop sends a `Message::Close` on the way out).
- `src/app.rs` — `App` holds chat history (`Vec<ChatLine>`), the input line/cursor, scroll state, and the locally tracked `name`/`room`/`members`. `App::handle_server_line` is the client-side half of the line-prefix wire protocol: `MSG `/`SYS `/`ERR `/`USERS ` prefixes map to `LineKind` variants, and `apply_sys` additionally pattern-matches the exact SYS wordings the server emits (`"name set to "`, `"joined room "`, `"left room "` — see `connection.rs` in the sibling `cenchat-server` repo) to keep the status bar and room-members sidebar in sync. Anything else prints as `LineKind::Raw`. Chat text is never echoed locally — the server's `broadcast` includes the sender, so the CLI only ever renders what comes back over the socket.
- `src/ui.rs` — pure rendering: a one-line status bar (url/name/room/connection state), a bordered `Messages` pane (word-wrapped via `textwrap`, manually scrolled by raw line count — not `Paragraph`'s unstable `line_count` API — and cached into `app.content_lines`/`app.view_height` each frame so key handlers can clamp `PageUp`/`PageDown`), a bordered `Room` members sidebar, and a bordered input line with a manually tracked cursor position (`frame.set_cursor_position`).

There is no client-side parsing/validation of the `/name`, `/join`, `/leave`, `/who` commands mentioned in the README — the CLI sends whatever the user typed as-is and relies entirely on the server to interpret slash-commands and to emit correctly prefixed (`MSG`/`SYS`/`ERR`/`USERS`) responses. When adding new commands or protocol prefixes, `App::handle_server_line`/`apply_sys` in `src/app.rs` and the corresponding server-side handling must be kept in sync (see the sibling `cenchat-server` repo).
