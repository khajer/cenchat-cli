mod app;
mod ui;

use std::env;

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use app::{App, LineKind};

/// Messages handed from the websocket read task to the TUI event loop.
enum NetEvent {
    Line(String),
    Closed,
    Error(String),
}

#[tokio::main]
async fn main() {
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            eprintln!("Usage: cenchat-cli <ws-url>");
            eprintln!("Example: cenchat-cli wss://echo.websocket.org");
            std::process::exit(1);
        }
    };

    println!("Connecting to {url}...");
    let (ws_stream, response) = match connect_async(&url).await {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Failed to connect: {err}");
            std::process::exit(1);
        }
    };
    println!("Connected (HTTP status: {})", response.status());

    let (mut write, mut read) = ws_stream.split();
    let (net_tx, mut net_rx) = mpsc::unbounded_channel::<NetEvent>();

    let read_task = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            let event = match msg {
                Ok(Message::Text(text)) => NetEvent::Line(text.to_string()),
                Ok(Message::Binary(data)) => NetEvent::Line(format!("[binary {} bytes]", data.len())),
                Ok(Message::Close(_)) => {
                    let _ = net_tx.send(NetEvent::Closed);
                    break;
                }
                Ok(_) => continue,
                Err(err) => {
                    let _ = net_tx.send(NetEvent::Error(err.to_string()));
                    break;
                }
            };
            if net_tx.send(event).is_err() {
                break;
            }
        }
        let _ = net_tx.send(NetEvent::Closed);
    });

    let mut terminal = ratatui::init();
    let mut app = App::new(url);
    app.connected = true;
    app.push(
        LineKind::Info,
        "Connected. Commands: /name <name>  /join <room>  /leave  /who  (anything else is chat text)",
    );

    let mut events = EventStream::new();

    loop {
        if let Err(err) = terminal.draw(|frame| ui::draw(frame, &mut app)) {
            eprintln!("draw error: {err}");
            break;
        }

        if app.should_quit {
            break;
        }

        tokio::select! {
            net = net_rx.recv() => {
                match net {
                    Some(NetEvent::Line(line)) => app.handle_server_line(&line),
                    Some(NetEvent::Closed) => {
                        app.connected = false;
                        app.push(LineKind::Info, "Connection closed by server.");
                    }
                    Some(NetEvent::Error(err)) => {
                        app.connected = false;
                        app.push(LineKind::Info, format!("Connection error: {err}"));
                    }
                    None => {
                        app.connected = false;
                    }
                }
            }
            ev = events.next() => {
                match ev {
                    Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                        handle_key(&mut app, &mut write, key.code, key.modifiers).await;
                    }
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        app.push(LineKind::Info, format!("Input error: {err}"));
                    }
                    None => app.should_quit = true,
                }
            }
        }
    }

    let _ = write.send(Message::Close(None)).await;
    ratatui::restore();
    read_task.abort();
    println!("Disconnected.");
}

async fn handle_key(
    app: &mut App,
    write: &mut (impl futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    code: KeyCode,
    modifiers: KeyModifiers,
) {
    match code {
        KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.take_input();
        }
        KeyCode::Enter => {
            let line = app.take_input();
            app.scroll_offset = 0;
            if !line.is_empty() {
                if let Err(err) = write.send(Message::Text(line.into())).await {
                    app.push(LineKind::Info, format!("Error sending message: {err}"));
                    app.connected = false;
                }
            }
        }
        KeyCode::Char(c) => app.insert_char(c),
        KeyCode::Backspace => app.delete_before_cursor(),
        KeyCode::Delete => app.delete_at_cursor(),
        KeyCode::Left => app.move_left(),
        KeyCode::Right => app.move_right(),
        KeyCode::Home => app.move_home(),
        KeyCode::End => app.move_end(),
        KeyCode::Up => app.scroll_up(1),
        KeyCode::Down => app.scroll_down(1),
        KeyCode::PageUp => {
            let by = app.view_height.max(1);
            app.scroll_up(by);
        }
        KeyCode::PageDown => {
            let by = app.view_height.max(1);
            app.scroll_down(by);
        }
        _ => {}
    }
}
