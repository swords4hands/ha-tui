use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render_events(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.get_filtered_events();
    let items: Vec<ListItem> = filtered
        .iter()
        .skip(app.events_scroll)
        .map(|ev| {
            let time_str = ev.timestamp.format("%H:%M:%S%.3f").to_string();
            let domain = ev.entity_id.split('.').next().unwrap_or("");

            let (domain_tag, tag_color) = match domain {
                "light" => ("[LIGHT] ", Color::Yellow),
                "switch" => ("[SWITCH]", Color::Cyan),
                "sensor" => ("[SENSOR]", Color::Blue),
                "binary_sensor" => ("[B-SENS]", Color::LightBlue),
                "media_player" => ("[MEDIA] ", Color::Magenta),
                "automation" => ("[AUTO]  ", Color::LightMagenta),
                "button" => ("[BUTTON]", Color::LightYellow),
                _ => ("[EVENT] ", Color::Gray),
            };

            let line = Line::from(vec![
                Span::styled(format!("[{}] ", time_str), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} ", domain_tag), Style::default().fg(tag_color)),
                Span::styled(format!("{:<26} ", ev.friendly_name), Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:<32} ", format!("({})", ev.entity_id)),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("{:<10} ", ev.old_state), Style::default().fg(Color::Gray)),
                Span::styled("➔ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &ev.new_state,
                    Style::default()
                        .fg(if ev.new_state == "on" || ev.new_state == "playing" {
                            Color::Green
                        } else if ev.new_state == "off" || ev.new_state == "paused" {
                            Color::Red
                        } else {
                            Color::Cyan
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .title(format!(
            " 📜 Live Event Stream (Showing: {} / {} events | Scroll: {}) ",
            filtered.len(),
            app.events.len(),
            app.events_scroll
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    f.render_widget(List::new(items).block(list_block), area);
}
