use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::{App, ChatLine, LineKind};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let root = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .split(frame.area());

    draw_status(frame, root[0], app);

    let body = Layout::horizontal([Constraint::Min(20), Constraint::Length(22)]).split(root[1]);

    draw_messages(frame, body[0], app);
    draw_members(frame, body[1], app);

    draw_input(frame, root[2], app);
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let name = app.name.as_deref().unwrap_or("(no name)");
    let room = app.room.as_deref().unwrap_or("(no room)");
    let conn = if app.connected {
        "connected"
    } else {
        "disconnected"
    };
    let text = format!(" cenchat — {} | name: {name} | room: {room} | {conn}", app.url);
    let style = Style::default()
        .fg(Color::Black)
        .bg(if app.connected { Color::Cyan } else { Color::Red })
        .add_modifier(Modifier::BOLD);
    frame.render_widget(Paragraph::new(text).style(style), area);
}

fn draw_messages(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default().borders(Borders::ALL).title("Messages");
    let inner_width = block.inner(area).width;
    let inner_height = block.inner(area).height as usize;

    let lines = wrap_messages(&app.messages, inner_width);
    app.content_lines = lines.len();
    app.view_height = inner_height;

    let max_scroll = lines.len().saturating_sub(inner_height);
    let scroll_offset = app.scroll_offset.min(max_scroll);
    let top = lines.len().saturating_sub(inner_height).saturating_sub(scroll_offset);

    let paragraph = Paragraph::new(lines).block(block).scroll((top as u16, 0));
    frame.render_widget(paragraph, area);
}

fn draw_members(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!("Room ({})", app.members.len());
    let block = Block::default().borders(Borders::ALL).title(title);

    let items: Vec<ListItem> = if app.members.is_empty() {
        vec![ListItem::new(Line::styled(
            "(no one here)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        app.members
            .iter()
            .map(|m| ListItem::new(Line::raw(m.clone())))
            .collect()
    };

    frame.render_widget(List::new(items).block(block), area);
}

fn draw_input(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Message — Enter send · Esc quit · ↑↓ PgUp/PgDn scroll");
    let inner = block.inner(area);

    let cursor_col = app.cursor as u16;
    let offset = cursor_col.saturating_sub(inner.width.saturating_sub(1));
    let visible: String = app.input.chars().skip(offset as usize).collect();

    frame.render_widget(Paragraph::new(visible).block(block), area);
    frame.set_cursor_position((inner.x + cursor_col.saturating_sub(offset), inner.y));
}

fn styled_text(line: &ChatLine) -> (Style, String) {
    match line.kind {
        LineKind::Chat => (Style::default(), line.text.clone()),
        LineKind::System => (
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
            format!("*** {}", line.text),
        ),
        LineKind::Error => (
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            format!("!!! {}", line.text),
        ),
        LineKind::Users => (
            Style::default().fg(Color::Cyan),
            format!("--- users: {}", line.text),
        ),
        LineKind::Raw => (Style::default().fg(Color::DarkGray), format!("< {}", line.text)),
        LineKind::Info => (
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::DIM),
            format!("» {}", line.text),
        ),
    }
}

fn wrap_messages(messages: &[ChatLine], width: u16) -> Vec<Line<'static>> {
    let width = width.max(1) as usize;
    let mut out = Vec::new();
    for line in messages {
        let (style, text) = styled_text(line);
        if text.is_empty() {
            out.push(Line::default());
            continue;
        }
        let options = textwrap::Options::new(width).subsequent_indent("  ");
        for wrapped in textwrap::wrap(&text, &options) {
            out.push(Line::styled(wrapped.into_owned(), style));
        }
    }
    out
}
