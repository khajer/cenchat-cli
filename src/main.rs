use std::env;

use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

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

    // Task that prints messages received from the server.
    let mut read_task = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => println!("< {text}"),
                Ok(Message::Binary(data)) => println!("< [binary {} bytes]", data.len()),
                Ok(Message::Close(frame)) => {
                    println!("Connection closed by server: {frame:?}");
                    break;
                }
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error reading message: {err}");
                    break;
                }
            }
        }
    });

    // Read lines from stdin and send them as text messages.
    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    println!("Type a message and press enter to send. Ctrl+D to quit.");
    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        if let Err(err) = write.send(Message::Text(line.into())).await {
                            eprintln!("Error sending message: {err}");
                            break;
                        }
                    }
                    Ok(None) => {
                        // stdin closed (Ctrl+D); close the connection gracefully.
                        let _ = write.send(Message::Close(None)).await;
                        break;
                    }
                    Err(err) => {
                        eprintln!("Error reading stdin: {err}");
                        break;
                    }
                }
            }
            _ = &mut read_task => {
                break;
            }
        }
    }
}
