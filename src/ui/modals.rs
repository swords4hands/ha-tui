use crate::app::{ActiveModal, App};
use crate::model::VOICE_PROFILES;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_modals(f: &mut Frame, app: &App) {
    match &app.active_modal {
        ActiveModal::None => {}
        ActiveModal::Help => render_help_modal(f),
        ActiveModal::Assist {
            input,
            response,
            loading,
        } => render_assist_modal(f, input, response.as_deref(), *loading),
        ActiveModal::VoicePicker {
            selected_idx,
            announce,
        } => render_voice_picker_modal(f, app, *selected_idx, *announce),
        ActiveModal::SpeakPrompt { input } => render_speak_prompt_modal(f, app, input),
        ActiveModal::ColorPicker {
            entity_id,
            selected_idx,
        } => render_color_picker_modal(f, entity_id, *selected_idx),
        ActiveModal::ServiceCall {
            entity_id,
            domain,
            service,
            data_json,
            active_field,
        } => render_service_modal(f, entity_id, domain, service, data_json, *active_field),
        ActiveModal::EntityDetail { entity_id } => render_entity_detail_modal(f, app, entity_id),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_help_modal(f: &mut Frame) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let text = vec![
        Line::from(vec![Span::styled(
            "  ha-tui Keyboard Shortcuts  ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation & Search:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  [1..9]        Switch directly to view tabs 1 through 9"),
        Line::from("  [Tab]         Next tab  |  [Shift+Tab] Previous tab"),
        Line::from("  [↑ / k]       Select previous item"),
        Line::from("  [↓ / j]       Select next item"),
        Line::from("  [/]           Search / Filter the current list (Enter: keep filter, Esc: clear)"),
        Line::from("  [Shift+A]     Open Home Assistant Assist (Natural language prompt)"),
        Line::from("  [r]           Force refresh all states & Kinect telemetry"),
        Line::from("  [?] or [h]    Toggle this help modal"),
        Line::from("  [q]           Quit application"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Kinect Management (Tab 3):",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  [Space/Enter] Toggle selected Kinect switch or run selected action"),
        Line::from("  [m]           Toggle Voice Responses (Mute/Unmute TTS speech)"),
        Line::from("  [M]           Toggle Microphone Array (Mute/Unmute mic input)"),
        Line::from("  [v]           Open Assistant Voice Picker (Rachel, GLaDOS, HAL, Cave, etc.)"),
        Line::from("  [+ / →]       Tilt Kinect camera UP (+5°) or cycle voice/mode"),
        Line::from("  [- / ←]       Tilt Kinect camera DOWN (-5°) or cycle voice/mode"),
        Line::from("  [0]           Level Kinect camera to 0° horizontal"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "General Device Controls:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  [Space]       Toggle selected entity ON / OFF (or Play/Pause)"),
        Line::from("  [Enter]       Execute button / Run script / Inspect details"),
        Line::from("  [+ / →]       Increase light brightness / media volume"),
        Line::from("  [- / ←]       Decrease light brightness / media volume"),
        Line::from("  [a] / [x]     Turn ALL lights ON / Turn ALL lights OFF"),
        Line::from("  [c]           Open Color Preset Picker (Lights view)"),
        Line::from("  [f]           Trigger Find My Phone"),
        Line::from("  [s]           Open Call Service dialog for selected entity"),
        Line::from("  [d]           Cycle domain filters (Entities Tab 8)"),
    ];

    let block = Block::default()
        .title(" 📖 Help & Keybindings (Press Esc to close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    f.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn render_voice_picker_modal(f: &mut Frame, app: &App, selected_idx: usize, announce: bool) {
    let area = centered_rect(62, 65, f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Voices list
            Constraint::Length(3), // Footer options
        ])
        .split(area);

    let items: Vec<ListItem> = VOICE_PROFILES
        .iter()
        .enumerate()
        .map(|(idx, (key, name, desc))| {
            let is_sel = idx == selected_idx;
            let is_active = *key == app.kinect.active_voice;
            let prefix = if is_sel { "▶ " } else { "  " };
            let active_badge = if is_active { "★ ACTIVE" } else { "        " };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{:<14} ", name),
                    if is_sel {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    },
                ),
                Span::styled(
                    format!("{} ", active_badge),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(*desc, Style::default().fg(Color::Gray)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let block = Block::default()
        .title(" 🗣️ Select Kinect Assistant Voice (Enter: Apply | Space: Toggle Announce | Esc: Cancel) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(List::new(items).block(block), chunks[0]);

    let announce_line = Line::from(vec![
        Span::styled(" Vocal Announcement on Switch [Space]: ", Style::default().fg(Color::Gray)),
        Span::styled(
            if announce {
                "🔊 YES (Speak greeting)"
            } else {
                "🔇 NO (Silent switch)"
            },
            Style::default()
                .fg(if announce { Color::Green } else { Color::Yellow })
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(Paragraph::new(announce_line).block(footer_block), chunks[1]);
}

fn render_speak_prompt_modal(f: &mut Frame, app: &App, input: &str) {
    let area = centered_rect(58, 30, f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input box
            Constraint::Min(2),    // Info
        ])
        .split(area);

    let input_block = Block::default()
        .title(format!(
            " 🔊 Speak via Kinect ({}) — Type phrase & press Enter ",
            app.kinect.active_voice_name
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let input_p = Paragraph::new(format!("> {}_", input)).block(input_block);
    f.render_widget(input_p, chunks[0]);

    let footer = Paragraph::new("Press Enter to synthesize & play through speakers, Esc to cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[1]);
}

fn render_assist_modal(f: &mut Frame, input: &str, response: Option<&str>, loading: bool) {
    let area = centered_rect(60, 45, f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input box
            Constraint::Min(4),    // Response box
            Constraint::Length(1), // Footer instruction
        ])
        .split(area);

    let input_block = Block::default()
        .title(" 🗣 Home Assistant Assist [Shift+A] (e.g. 'turn off bedroom light') ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_p = Paragraph::new(format!("> {}_", input)).block(input_block);
    f.render_widget(input_p, chunks[0]);

    let response_block = Block::default()
        .title(" Assistant Response ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let resp_text = if loading {
        vec![Line::from(vec![Span::styled(
            "Processing command with Home Assistant...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )])]
    } else if let Some(resp) = response {
        vec![Line::from(vec![Span::styled(
            resp,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )])]
    } else {
        vec![Line::from("Type a command and press Enter.")]
    };

    f.render_widget(
        Paragraph::new(resp_text)
            .block(response_block)
            .wrap(Wrap { trim: true }),
        chunks[1],
    );

    let footer = Paragraph::new("Press Enter to send, Esc to close")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

pub const COLOR_PRESETS: [(&str, (u8, u8, u8)); 9] = [
    ("Warm White (2700K)", (255, 180, 107)),
    ("Soft White (3000K)", (255, 206, 166)),
    ("Cool White (5000K)", (255, 255, 255)),
    ("Crimson Red", (255, 20, 20)),
    ("Lime Green", (20, 255, 40)),
    ("Royal Blue", (20, 80, 255)),
    ("Cyan / Aqua", (0, 240, 255)),
    ("Magenta / Pink", (255, 20, 180)),
    ("Deep Purple", (160, 20, 255)),
];

fn render_color_picker_modal(f: &mut Frame, entity_id: &str, selected_idx: usize) {
    let area = centered_rect(45, 55, f.area());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = COLOR_PRESETS
        .iter()
        .enumerate()
        .map(|(idx, (name, (r, g, b)))| {
            let is_sel = idx == selected_idx;
            let prefix = if is_sel { "▶ " } else { "  " };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled("● ", Style::default().fg(Color::Rgb(*r, *g, *b))),
                Span::styled(
                    format!("{:<20} ", name),
                    if is_sel {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(format!("({},{},{})", r, g, b), Style::default().fg(Color::DarkGray)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let block = Block::default()
        .title(format!(
            " 🎨 Pick Color: {} (Enter: Apply | Esc: Cancel) ",
            entity_id
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    f.render_widget(List::new(items).block(block), area);
}

fn render_service_modal(
    f: &mut Frame,
    entity_id: &str,
    domain: &str,
    service: &str,
    data_json: &str,
    active_field: usize,
) {
    let area = centered_rect(55, 50, f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Info
            Constraint::Length(3), // Domain & Service input
            Constraint::Min(4),    // JSON payload input
            Constraint::Length(1), // Footer
        ])
        .split(area);

    let block = Block::default()
        .title(" 🛠 Call Service ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    let info = Paragraph::new(format!("Target Entity: {}", entity_id))
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(info, chunks[0]);

    let service_box = Paragraph::new(format!("Service: {}.{}_", domain, service))
        .block(Block::default().borders(Borders::ALL).title(" Domain & Service "))
        .style(if active_field == 0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });
    f.render_widget(service_box, chunks[1]);

    let data_box = Paragraph::new(format!("Payload: {}_", data_json))
        .block(Block::default().borders(Borders::ALL).title(" JSON Data (Optional) "))
        .style(if active_field == 1 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        });
    f.render_widget(data_box, chunks[2]);

    let footer = Paragraph::new("Press Tab to switch fields, Enter to execute, Esc to cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[3]);
}

fn render_entity_detail_modal(f: &mut Frame, app: &App, entity_id: &str) {
    let area = centered_rect(65, 75, f.area());
    f.render_widget(Clear, area);

    let entity_opt = app.entities.get(entity_id);
    let text = if let Some(entity) = entity_opt {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Entity ID:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &entity.entity_id,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
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
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Full Attributes:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
        ];

        for (k, v) in &entity.attributes {
            lines.push(Line::from(vec![
                Span::styled(format!("  • {}: ", k), Style::default().fg(Color::LightCyan)),
                Span::styled(
                    serde_json::to_string(v).unwrap_or_default(),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        lines
    } else {
        vec![Line::from("Entity not found")]
    };

    let block = Block::default()
        .title(format!(
            " 🔍 Entity Inspection: {} (Esc: Close) ",
            entity_id
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightCyan));

    f.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}
