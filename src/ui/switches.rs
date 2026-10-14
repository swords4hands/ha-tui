use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_switches(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // List of switches & automations
            Constraint::Percentage(40), // Info card & metadata
        ])
        .split(area);

    let items = app.get_switches_and_automations();
    let selected_item = items.get(app.switches_selected).copied();

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, entity)| {
            let is_sel = idx == app.switches_selected;
            let on = entity.is_on();
            let prefix = if is_sel { "▶ " } else { "  " };

            let (domain_tag, tag_color) = match entity.domain() {
                "switch" => ("[SWITCH]", Color::Cyan),
                "automation" => ("[AUTO]  ", Color::Magenta),
                "input_boolean" => ("[BOOL]  ", Color::LightGreen),
                _ => ("[MISC]  ", Color::Gray),
            };

            let state_str = if on { "ON " } else { "OFF" };
            let state_style = if on {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Red)
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{} ", domain_tag), Style::default().fg(tag_color)),
                Span::styled(
                    format!("{:<30} ", entity.friendly_name()),
                    if is_sel {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(state_str, state_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .title(" 🎛 Switches & Automations (Space: Toggle | Enter: Details) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(List::new(list_items).block(list_block), chunks[0]);

    // Right pane: details
    let detail_text = if let Some(entity) = selected_item {
        let on = entity.is_on();
        let last_changed = entity
            .last_changed
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(entity.friendly_name(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Entity ID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entity.entity_id, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Domain: ", Style::default().fg(Color::DarkGray)),
                Span::styled(entity.domain(), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if on { "ON" } else { "OFF" },
                    Style::default().fg(if on { Color::Green } else { Color::Red }).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Last Changed: ", Style::default().fg(Color::DarkGray)),
                Span::styled(last_changed, Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Quick Description & Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
        ];

        // Entity specific notes
        if entity.entity_id.contains("kinect_continuous_listen") {
            lines.push(Line::from("  Enables persistent listening without requiring wake words."));
        } else if entity.entity_id.contains("kinect_live_camera_stream") {
            lines.push(Line::from("  Switches Kinect from IR Depth motion mode to live RGB video."));
        } else if entity.entity_id.contains("kinect_motion_lights") {
            lines.push(Line::from("  Automated lighting trigger based on 3D depth human sensing."));
        } else if entity.entity_id.contains("jellyfin_ambient_lights") {
            lines.push(Line::from("  Synchronizes room lighting color to current active Jellyfin media."));
        } else if entity.entity_id.contains("kinect_ocarina_detection") {
            lines.push(Line::from("  Listens for Zelda Ocarina note melodies to trigger smart home macros."));
        } else if entity.entity_id.contains("ps5_curfew") {
            lines.push(Line::from("  Automatic sleep timer and shutdown enforcement for PlayStation 5."));
        }

        lines.push(Line::from(""));
        lines.push(Line::from("  Press [Space] to toggle ON/OFF"));
        lines.push(Line::from("  Press [Enter] to inspect all raw JSON attributes"));

        lines
    } else {
        vec![Line::from("No entity selected")]
    };

    let detail_block = Block::default()
        .title(" Selected Switch Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    f.render_widget(Paragraph::new(detail_text).block(detail_block).wrap(Wrap { trim: true }), chunks[1]);
}
