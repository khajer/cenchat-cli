# cenchat-cli

Terminal WebSocket client for [cenchat-server](https://github.com/khajer/cenchat).

## Requirements

- Rust (edition 2024)

## Build

```bash
cargo build
```

## Usage

```bash
cargo run -- <ws-url>
```

Example, against a local cenchat-server instance:

```bash
cargo run -- ws://127.0.0.1:9001
```

Once connected, the client switches into a full-screen terminal UI: a status bar (URL / name / room / connection state), a scrollable message pane, a room-members sidebar, and an input line at the bottom.

### Keys

- `Enter` — send the current input line verbatim to the server (chat text or a `/` command)
- `Backspace` / `Delete` / `←` / `→` / `Home` / `End` — edit the input line
- `Ctrl+U` — clear the input line
- `↑` / `↓` — scroll the message pane by one line
- `PageUp` / `PageDown` — scroll the message pane by a page
- `Esc` / `Ctrl+C` — quit (closes the connection gracefully)

### Commands

- `/name <name>` — set your display name
- `/join <room>` — join a room
- `/leave` — leave the current room
- `/who` — list users (also populates the room-members sidebar)
- anything else is sent as chat text

### Message rendering

Incoming lines are parsed by a simple prefix protocol and rendered in the message pane:

| Prefix    | Rendered as         |
|-----------|----------------------|
| `MSG ...`   | plain chat message   |
| `SYS ...`   | `*** ...` (system)   |
| `ERR ...`   | `!!! ...` (error)    |
| `USERS ...` | `--- users: ...` (also updates the members sidebar) |
| anything else | `< ...` (raw)     |

## Dependencies

- `tokio` (full)
- `tokio-tungstenite` (native-tls)
- `futures-util`
- `url`
- `ratatui` / `crossterm` (terminal UI)
- `textwrap` (message word-wrapping)
