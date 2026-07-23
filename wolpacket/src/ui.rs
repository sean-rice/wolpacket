use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(3),    // device list
            Constraint::Length(3), // LAN
            Constraint::Length(4), // status / hints
        ])
        .split(frame.area());

    render_device_list(frame, app, chunks[0]);
    render_lan(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_device_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .devices
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Gray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let text = format!("  {:<18} {:<20}    {}", d.id, d.mac, d.name);
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Devices"))
        .highlight_style(Style::default());

    frame.render_widget(list, area);
}

fn render_lan(frame: &mut Frame, app: &App, area: Rect) {
    let lan_style = if !app.editing_lan {
        Style::default()
    } else if app.is_lan_valid() {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };

    let prefix = if app.editing_lan { "» " } else { "  " };

    let hint = if !app.editing_lan {
        "  [e to edit]"
    } else if app.is_lan_valid() {
        "  (Enter to confirm, Esc to cancel)"
    } else {
        "  (invalid CIDR: Esc to cancel)"
    };

    let text = Text::from(Line::from(vec![
        Span::raw(prefix),
        Span::styled(app.lan_display(), lan_style),
        Span::raw(hint),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .title("LAN Subnet (broadcast derived from this)");

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let hints = vec![
        Span::raw("↑/↓ or j/k: navigate   "),
        Span::raw("Enter: wake device   "),
        Span::raw("e: edit LAN   "),
        Span::raw("q: quit"),
    ];

    let text = Text::from(vec![
        Line::from(hints),
        Line::from(Span::styled(
            &app.status,
            Style::default().fg(app.status_color()),
        )),
    ]);

    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Status")),
        area,
    );
}
