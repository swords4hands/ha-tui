use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_entities(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search & Domain Filter Bar
            Constraint::Min(10),   // Split view: List & Inspector
        ])
        .split(area);

    // Search & Domain Bar
    let search_style = if app.is_searching {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let domain_tag = app.domain_filter.as_deref().unwrap_or("ALL");
    let search_bar = Paragraph::new(Line::from(vec![
        Span::styled(" Filter: [/]: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if app.search_query.is_empty() && !app.is_searching {
                "<press / to search>"
            } else {
                &app.search_query
            },
            search_style,
        ),
        Span::raw("    "),
        Span::styled("Domain [d]: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("[{}]", domain_tag), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  (Press 'd' to cycle domains, Esc to clear search)"),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if app.is_searching { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
    );
    f.render_widget(search_bar, main_chunks[0]);

    // Split view
    let split_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55), // Entity list
            Constraint::Percentage(45), // Attribute inspector
        ])
        .split(main_chunks[1]);

    let filtered = app.get_filtered_entities();
    let selected_entity = filtered.get(app.entities_selected).copied();

    let list_items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(idx, entity)| {
            let is_sel = idx == app.entities_selected;
            let prefix = if is_sel { "▶ " } else { "  " };

            let domain_badge = format!("[{:<7}]", entity.domain());
            let badge_color = match entity.domain() {
                "light" => Color::Yellow,
                "switch" => Color::Cyan,
                "sensor" => Color::Blue,
                "binary_sensor" => Color::LightBlue,
                "button" => Color::LightYellow,
                "script" => Color::LightCyan,
                "automation" => Color::Magenta,
                "media_player" => Color::LightMagenta,
                _ => Color::DarkGray,
            };

            let state_str = if entity.state.len() > 14 {
                format!("{}…", &entity.state[..13])
            } else {
                entity.state.clone()
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{} ", domain_badge), Style::default().fg(badge_color)),
                Span::styled(
                    format!("{:<28} ", entity.friendly_name()),
                    if is_sel {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(
                    state_str,
                    Style::default().fg(if entity.is_on() { Color::Green } else { Color::White }),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .title(format!(" Entities Explorer ({} matches) ", filtered.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    f.render_widget(List::new(list_items).block(list_block), split_chunks[0]);

    // Attribute Inspector pane
    let inspector_text = if let Some(entity) = selected_entity {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Entity ID:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entity.entity_id, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Friendly Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(entity.friendly_name(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Domain:        ", Style::default().fg(Color::DarkGray)),
                Span::styled(entity.domain(), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("Current State: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &entity.state,
                    Style::default().fg(if entity.is_on() { Color::Green } else { Color::Yellow }).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Last Changed:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    entity
                        .last_changed
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![
                Span::styled("Last Updated:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    entity
                        .last_updated
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Attributes JSON:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
        ];

        // Format attributes
        for (k, v) in &entity.attributes {
            let val_str = serde_json::to_string(v).unwrap_or_default();
            lines.push(Line::from(vec![
                Span::styled(format!("  {}: ", k), Style::default().fg(Color::LightCyan)),
                Span::styled(val_str, Style::default().fg(Color::White)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from("  [Space] Toggle  |  [s] Call Service  |  [c] Color"));

        lines
    } else {
        vec![Line::from("No entity selected")]
    };

    let inspector_block = Block::default()
        .title(" Entity Attribute Inspector ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightBlue));
    f.render_widget(Paragraph::new(inspector_text).block(inspector_block).wrap(Wrap { trim: false }), split_chunks[1]);
}
