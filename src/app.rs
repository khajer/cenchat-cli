//! TUI application state: chat history, input line, and the client-side
//! tracking of name/room/members inferred from the server's line-prefix
//! protocol (see cenchat-server's `protocol.rs`).

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// `MSG name: text` — a chat message.
    Chat,
    /// `SYS ...` — a server system notice.
    System,
    /// `ERR ...` — a server error.
    Error,
    /// `USERS ...` — the member list of the current room.
    Users,
    /// Anything else the server sent, printed verbatim.
    Raw,
    /// Locally generated status line (connect/disconnect/etc).
    Info,
}

pub struct ChatLine {
    pub kind: LineKind,
    pub text: String,
}

pub struct App {
    pub url: String,
    pub messages: Vec<ChatLine>,
    pub input: String,
    pub cursor: usize,
    /// Lines scrolled up from the live bottom; 0 means auto-follow new messages.
    pub scroll_offset: usize,
    /// Cached from the last draw, used to clamp scrolling from key handlers.
    pub view_height: usize,
    pub content_lines: usize,
    pub name: Option<String>,
    pub room: Option<String>,
    pub members: Vec<String>,
    pub connected: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(url: String) -> Self {
        Self {
            url,
            messages: Vec::new(),
            input: String::new(),
            cursor: 0,
            scroll_offset: 0,
            view_height: 0,
            content_lines: 0,
            name: None,
            room: None,
            members: Vec::new(),
            connected: false,
            should_quit: false,
        }
    }

    pub fn push(&mut self, kind: LineKind, text: impl Into<String>) {
        self.messages.push(ChatLine { kind, text: text.into() });
    }

    /// Client-side half of the server's line-prefix protocol: turns one
    /// incoming line into a styled `ChatLine`, and updates the locally
    /// tracked name/room/members from the well-known SYS/USERS wordings the
    /// server emits (see cenchat-server's `protocol.rs` and `connection.rs`).
    pub fn handle_server_line(&mut self, line: &str) {
        match line.split_once(' ') {
            Some(("MSG", rest)) => self.push(LineKind::Chat, rest),
            Some(("SYS", rest)) => {
                self.apply_sys(rest);
                self.push(LineKind::System, rest);
            }
            Some(("ERR", rest)) => self.push(LineKind::Error, rest),
            Some(("USERS", rest)) => {
                self.members = rest.split_whitespace().map(String::from).collect();
                self.push(LineKind::Users, rest);
            }
            _ => self.push(LineKind::Raw, line),
        }
    }

    fn apply_sys(&mut self, text: &str) {
        if let Some(name) = text.strip_prefix("name set to ") {
            self.name = Some(name.to_string());
        } else if let Some(rest) = text.strip_prefix("joined room ") {
            if let Some((room, _)) = rest.split_once(" (") {
                self.room = Some(room.to_string());
            }
        } else if let Some(room) = text.strip_prefix("left room ") {
            if self.room.as_deref() == Some(room) {
                self.room = None;
                self.members.clear();
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let idx = self.byte_index();
        self.input.insert(idx, c);
        self.cursor += 1;
    }

    pub fn delete_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let idx = self.byte_index();
        let prev = self.input[..idx]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.input.drain(prev..idx);
        self.cursor -= 1;
    }

    pub fn delete_at_cursor(&mut self) {
        let idx = self.byte_index();
        if idx < self.input.len() {
            let next = self.input[idx..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| idx + i)
                .unwrap_or(self.input.len());
            self.input.drain(idx..next);
        }
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.input.chars().count() {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.input.chars().count();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.input.len())
    }

    /// Clears and returns the current input line, resetting the cursor.
    pub fn take_input(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.input)
    }

    pub fn scroll_up(&mut self, by: usize) {
        let max = self.content_lines.saturating_sub(self.view_height);
        self.scroll_offset = (self.scroll_offset + by).min(max);
    }

    pub fn scroll_down(&mut self, by: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(by);
    }
}
