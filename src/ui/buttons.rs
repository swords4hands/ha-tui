use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_buttons(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // List of buttons and scripts
            Constraint::Percentage(40), // Info card and trigger guide
        ])
        .split(area);

    let items = app.get_buttons_and_scripts();
    let selected_item = items.get(app.buttons_selected).copied();

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, entity)| {
            let is_sel = idx == app.buttons_selected;
            let prefix = if is_sel { "▶ " } else { "  " };

            let (domain_tag, tag_color) = match entity.domain() {
                "button" => ("[BUTTON]", Color::Yellow),
                "script" => ("[SCRIPT]", Color::Cyan),
                _ => ("[ACTION]", Color::Gray),
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{} ", domain_tag), Style::default().fg(tag_color)),
                Span::styled(
                    format!("{:<34} ", entity.friendly_name()),
                    if is_sel {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(" [Press Enter] ", Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .title(" 🔘 Actionable Buttons & Scripts (Enter or Space: Execute) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(List::new(list_items).block(list_block), chunks[0]);

    // Right pane: Details
    let detail_text = if let Some(entity) = selected_item {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Action: ", Style::default().fg(Color::DarkGray)),
                Span::styled(entity.friendly_name(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Entity ID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entity.entity_id, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                Span::styled(entity.domain().to_uppercase(), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Description / Effect:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
        ];

        let desc = match entity.entity_id.as_str() {
            "button.find_my_phone" => "Rings Beck's phone at maximum volume with notification alert.",
            "button.tilt_motor_up" | "script.kinect_tilt_up" => "Tilts Xbox 360 Kinect physical motorized base upward (+5 deg).",
            "button.tilt_motor_down" | "script.kinect_tilt_down" => "Tilts Xbox 360 Kinect physical motorized base downward (-5 deg).",
            "button.tilt_motor_level" | "script.kinect_tilt_level" => "Centers Xbox 360 Kinect motorized base to perfectly horizontal (0 deg).",
            "button.turn_off_ps5_now" | "script.ps5_turn_off" => "Sends network shutdown command to PlayStation 5 to enter rest mode / power off.",
            "button.test_speaker_voice" => "Plays a test vocal synthesis greeting through the workstation ALC221 analog line-out.",
            "button.test_motion_lights" => "Simulates an IR depth camera motion trigger to toggle ambient lighting on.",
            "button.sample_jellyfin_ambient_color" => "Samples video frame color from active Jellyfin stream and applies to Kauf bulbs.",
            "button.jellyfin_run_full_cleanup" => "Cleans transcode temp files, old log files, and image cache.",
            "button.jellyfin_optimize_database" => "Optimizes and compacts SQLite database on beck-link.",
            "button.jellyfin_rescan_media_library" => "Triggers library scan for newly added TV shows and Movies.",
            _ => "Triggers service call immediately when activated.",
        };
        lines.push(Line::from(format!("  {}", desc)));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Press [Enter] or [Space] to execute this action.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));

        lines
    } else {
        vec![Line::from("No action selected")]
    };

    let detail_block = Block::default()
        .title(" Action Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    f.render_widget(Paragraph::new(detail_text).block(detail_block).wrap(Wrap { trim: true }), chunks[1]);
}
