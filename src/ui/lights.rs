use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_lights(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65), // Lights list
            Constraint::Percentage(35), // Details & Presets
        ])
        .split(area);

    let lights = app.get_lights();
    let selected_light = lights.get(app.lights_selected).copied();

    // Left pane: Lights list
    let list_items: Vec<ListItem> = lights
        .iter()
        .enumerate()
        .map(|(idx, l)| {
            let is_sel = idx == app.lights_selected;
            let on = l.is_on();
            let b_pct = l.brightness_pct().unwrap_or(0);

            let prefix = if is_sel { "▶ " } else { "  " };
            let bulb_icon = if on { "💡" } else { "⚫" };
            let state_str = if on { "ON " } else { "OFF" };
            let state_color = if on { Color::Yellow } else { Color::DarkGray };

            // Draw a progress bar
            let bar_len = (b_pct / 5).clamp(0, 20) as usize;
            let bar = format!("[{}{}] {:>3}%", "█".repeat(bar_len), "░".repeat(20 - bar_len), b_pct);

            // Color indication
            let color_indicator = if let Some((r, g, b)) = l.rgb_color() {
                format!(" RGB({},{},{})", r, g, b)
            } else if let Some(ct) = l.attributes.get("color_temp_kelvin").and_then(|v| v.as_u64()) {
                format!(" {}K", ct)
            } else {
                "".to_string()
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::raw(format!("{} ", bulb_icon)),
                Span::styled(
                    format!("{:<24} ", l.friendly_name()),
                    if is_sel {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(state_str, Style::default().fg(state_color).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(bar, Style::default().fg(if on { Color::Cyan } else { Color::DarkGray })),
                Span::styled(color_indicator, Style::default().fg(Color::Magenta)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .title(" 💡 Lights Management (Space: Toggle | +/-: Brightness | c: Colors) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(List::new(list_items).block(list_block), chunks[0]);

    // Right pane: Selected Light Detail & Controls
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Selected Light Info
            Constraint::Min(8),     // Presets & Color shortcuts
        ])
        .split(chunks[1]);

    let detail_text = if let Some(l) = selected_light {
        let on = l.is_on();
        let b_pct = l.brightness_pct().unwrap_or(0);
        let rgb_str = l
            .rgb_color()
            .map(|(r, g, b)| format!("rgb({}, {}, {})", r, g, b))
            .unwrap_or_else(|| "N/A".to_string());
        let effect_str = l
            .attributes
            .get("effect")
            .and_then(|e| e.as_str())
            .unwrap_or("None");

        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(l.friendly_name(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Entity ID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&l.entity_id, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if on { "ON" } else { "OFF" },
                    Style::default().fg(if on { Color::Green } else { Color::Red }).add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled("Brightness: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}%", b_pct), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Color / RGB: ", Style::default().fg(Color::DarkGray)),
                Span::styled(rgb_str, Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("Active Effect: ", Style::default().fg(Color::DarkGray)),
                Span::styled(effect_str, Style::default().fg(Color::LightBlue)),
            ]),
        ]
    } else {
        vec![Line::from("No light selected")]
    };

    let detail_block = Block::default()
        .title(" Selected Light Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    f.render_widget(Paragraph::new(detail_text).block(detail_block), detail_chunks[0]);

    // Color & Quick Controls card
    let controls_text = vec![
        Line::from(vec![
            Span::styled("Brightness Adjust:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  [+] or [Right] : Increase brightness (+10%)"),
        Line::from("  [-] or [Left]  : Decrease brightness (-10%)"),
        Line::from("  [0] / [5] / [9] : Set 0% / 50% / 100%"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Color Presets (Selected Light):", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  [w] Warm White  (2700K)"),
        Line::from("  [k] Cool White  (5000K)"),
        Line::from("  [1] Red         (255, 0, 0)"),
        Line::from("  [2] Green       (0, 255, 0)"),
        Line::from("  [3] Blue        (0, 0, 255)"),
        Line::from("  [4] Purple      (180, 0, 255)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Master Commands:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  [a] Turn ALL lights ON"),
        Line::from("  [x] Turn ALL lights OFF"),
    ];

    let controls_block = Block::default()
        .title(" Controls & Color Presets ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightCyan));
    f.render_widget(
        Paragraph::new(controls_text)
            .block(controls_block)
            .wrap(Wrap { trim: true }),
        detail_chunks[1],
    );
}
