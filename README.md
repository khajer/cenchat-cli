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

Once connected, lines typed on stdin are sent to the server; incoming messages are printed to stdout. Ctrl+D closes the connection gracefully.

### Commands

- `/name <name>` — set your display name
- `/join <room>` — join a room
- `/leave` — leave the current room
- `/who` — list users
- anything else is sent as chat text

### Message rendering

Incoming lines are parsed by a simple prefix protocol:

| Prefix    | Rendered as         |
|-----------|----------------------|
| `MSG ...`   | plain chat message   |
| `SYS ...`   | `*** ...` (system)   |
| `ERR ...`   | `!!! ...` (error)    |
| `USERS ...` | `--- users: ...`     |
| anything else | `< ...` (raw)     |

## Dependencies

- `tokio` (full)
- `tokio-tungstenite` (native-tls)
- `futures-util`
- `url`
