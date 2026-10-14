pub mod buttons;
pub mod entities;
pub mod events;
pub mod kinect;
pub mod lights;
pub mod media;
pub mod modals;
pub mod overview;
pub mod sensors;
pub mod switches;

use crate::app::{App, Tab};
use crate::model::ConnectionStatus;
use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let show_search_banner = (app.is_searching || !app.search_query.is_empty())
        && app.current_tab != Tab::Entities;

    let chunks = if show_search_banner {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Top Header
                Constraint::Length(3), // Tab navigation bar
                Constraint::Length(3), // Active Search / Filter Bar
                Constraint::Min(10),   // Active Tab View Content
                Constraint::Length(3), // Bottom Footer / Toast bar
            ])
            .split(f.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Top Header
                Constraint::Length(3), // Tab navigation bar
                Constraint::Min(10),   // Active Tab View Content
                Constraint::Length(3), // Bottom Footer / Toast bar
            ])
            .split(f.area())
    };

    render_header(f, app, chunks[0]);
    render_tabs(f, app, chunks[1]);

    let content_area = if show_search_banner {
        render_search_banner(f, app, chunks[2]);
        chunks[3]
    } else {
        chunks[2]
    };

    // Active View
    match app.current_tab {
        Tab::Overview => overview::render_overview(f, app, content_area),
        Tab::Lights => lights::render_lights(f, app, content_area),
        Tab::Kinect => kinect::render_kinect(f, app, content_area),
        Tab::Switches => switches::render_switches(f, app, content_area),
        Tab::Buttons => buttons::render_buttons(f, app, content_area),
        Tab::Media => media::render_media(f, app, content_area),
        Tab::Sensors => sensors::render_sensors(f, app, content_area),
        Tab::Entities => entities::render_entities(f, app, content_area),
        Tab::Events => events::render_events(f, app, content_area),
    }

    let footer_area = if show_search_banner { chunks[4] } else { chunks[3] };
    render_footer(f, app, footer_area);

    // Overlays / Modals
    modals::render_modals(f, app);
}

fn render_search_banner(f: &mut Frame, app: &App, area: Rect) {
    let cursor = if app.is_searching { "█" } else { "" };
    let border_color = if app.is_searching {
        Color::Yellow
    } else {
        Color::Cyan
    };

    let line = Line::from(vec![
        Span::styled(
            " 🔍 Filter Current List: ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}{}", app.search_query, cursor),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "   (Press [Enter] to lock filter, [Esc] to clear)",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    f.render_widget(Paragraph::new(line).block(block), area);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let now = Local::now().format("%H:%M:%S").to_string();
    let entity_count = app.entities.len();

    let (status_badge, status_color) = match &app.status {
        ConnectionStatus::Connected => ("● CONNECTED", Color::Green),
        ConnectionStatus::Connecting => ("○ CONNECTING", Color::Yellow),
        ConnectionStatus::Reconnecting => ("◌ RECONNECTING", Color::Yellow),
        ConnectionStatus::Disconnected(_) => ("✕ OFFLINE", Color::Red),
    };

    let server_url = app.client.base_url();

    let header_line = Line::from(vec![
        Span::styled(" 🏠 HA-TUI ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("│ Home Assistant Terminal Dashboard ", Style::default().fg(Color::White)),
        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} ", status_badge), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!("({}) ", server_url), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("│ Entities: {} ", entity_count), Style::default().fg(Color::Cyan)),
        Span::styled(format!("│ Clock: {} ", now), Style::default().fg(Color::White)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    f.render_widget(Paragraph::new(header_line).block(block), area);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .select(app.current_tab as usize)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        );

    f.render_widget(tabs, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    // Show toast if present, otherwise keyboard shortcuts
    let content = if let Some(toast) = app.toasts.last() {
        let (prefix, color) = if toast.is_error {
            ("⚠ ERROR: ", Color::Red)
        } else {
            ("✓ NOTIFICATION: ", Color::Green)
        };
        Line::from(vec![
            Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(&toast.message, Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" [Tab/1-9]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Tabs  "),
            Span::styled("[Space]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Toggle  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Action  "),
            Span::styled("[/]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Search  "),
            Span::styled("[Shift+A]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Assist  "),
            Span::styled("[r]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Refresh  "),
            Span::styled("[?]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Help  "),
            Span::styled("[q]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    f.render_widget(Paragraph::new(content).block(block), area);
}
