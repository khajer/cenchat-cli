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

Everything lives in `src/main.rs` — a single `tokio` task structure with no other modules:

- `main` parses the WS URL from `argv[1]`, connects via `tokio_tungstenite::connect_async`, then splits the stream into `write`/`read` halves.
- A spawned `read_task` loops on `read.next()`, dispatching each incoming `Message::Text` to `render()` and printing binary sizes / close frames directly.
- The main loop uses `tokio::select!` to race `read_task` against `stdin` lines (via `tokio::io::AsyncBufReadExt`): each stdin line is sent verbatim as a `Message::Text` to the server, and Ctrl+D (EOF) sends a `Message::Close` and exits.
- `render(line)` implements the client-side half of a simple line-prefix wire protocol used to format incoming server text: `MSG `, `SYS `, `ERR `, `USERS ` prefixes map to distinct print styles; anything else prints with a `< ` prefix as raw/unrecognized output.

There is no client-side parsing/validation of the `/name`, `/join`, `/leave`, `/who` commands mentioned in the README — the CLI sends whatever the user types as-is and relies entirely on the server to interpret slash-commands and to emit correctly prefixed (`MSG`/`SYS`/`ERR`/`USERS`) responses. When adding new commands or protocol prefixes, this file's `render()` function and the corresponding server-side handling must be kept in sync (see the sibling `cenchat-server` repo).
