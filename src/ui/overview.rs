use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_overview(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Glance cards (Weather, System, Phone)
            Constraint::Min(10),    // Quick Lights & Switches
            Constraint::Length(8),  // Recent Activity Feed
        ])
        .split(area);

    // Top Glance row
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33), // Weather & Sun
            Constraint::Percentage(34), // Media / PS5 Glance
            Constraint::Percentage(33), // Phone & System Health
        ])
        .split(chunks[0]);

    // 1. Weather & Sun
    let weather = app.entities.get("weather.forecast_home");
    let sun = app.entities.get("sun.sun");
    let temp = weather
        .and_then(|w| w.attributes.get("temperature"))
        .and_then(|t| t.as_f64())
        .map(|t| format!("{:.0}°F", t))
        .unwrap_or_else(|| "--°F".to_string());
    let condition = weather
        .map(|w| w.state.as_str())
        .unwrap_or("Unknown");
    let sun_state = sun.map(|s| s.state.as_str()).unwrap_or("Unknown");

    let weather_text = vec![
        Line::from(vec![
            Span::styled("Condition: ", Style::default().fg(Color::DarkGray)),
            Span::styled(condition, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("Temp: ", Style::default().fg(Color::DarkGray)),
            Span::styled(temp, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Sun Phase: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if sun_state == "above_horizon" { "☀ Above Horizon (Day)" } else { "☾ Below Horizon (Night)" },
                Style::default().fg(if sun_state == "above_horizon" { Color::Yellow } else { Color::Blue }),
            ),
        ]),
    ];
    let weather_block = Block::default()
        .title(" ⛅ Weather & Environment ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(Paragraph::new(weather_text).block(weather_block), top_chunks[0]);

    // 2. Media / Gaming Glance
    let ps5 = app.entities.get("media_player.playstation_5");
    let roku = app.entities.get("media_player.roku_ultra");
    let ps5_state = ps5.map(|p| p.state.as_str()).unwrap_or("off");
    let roku_state = roku.map(|r| r.state.as_str()).unwrap_or("off");
    let roku_app = roku.and_then(|r| r.app_name()).unwrap_or("Home");

    let media_text = vec![
        Line::from(vec![
            Span::styled("PlayStation 5: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                ps5_state.to_uppercase(),
                Style::default().fg(if ps5_state == "playing" || ps5_state == "on" { Color::Green } else { Color::DarkGray }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Roku Ultra: ", Style::default().fg(Color::DarkGray)),
            Span::styled(roku_state.to_uppercase(), Style::default().fg(if roku_state != "off" { Color::Green } else { Color::DarkGray })),
            Span::styled(format!(" ({})", roku_app), Style::default().fg(Color::White)),
        ]),
    ];
    let media_block = Block::default()
        .title(" 🎮 Entertainment ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(Paragraph::new(media_text).block(media_block), top_chunks[1]);

    // 3. Phone & Kinect
    let phone_battery = app.entities.get("sensor.beckphone_battery_level");
    let phone_battery_pct = phone_battery.and_then(|b| b.numeric_value()).unwrap_or(0.0) as u16;
    let kinect_health = app.entities.get("sensor.kinect_health_status").map(|k| k.state.as_str()).unwrap_or("unknown");
    let ambient_lights = app.entities.get("switch.jellyfin_ambient_lights").map(|s| s.state.as_str()).unwrap_or("off");

    let phone_text = vec![
        Line::from(vec![
            Span::styled("Beck Phone: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}%", phone_battery_pct), Style::default().fg(if phone_battery_pct > 20 { Color::Green } else { Color::Red })),
        ]),
        Line::from(vec![
            Span::styled("Kinect Health: ", Style::default().fg(Color::DarkGray)),
            Span::styled(kinect_health, Style::default().fg(if kinect_health == "healthy" || kinect_health == "online" { Color::Green } else { Color::Yellow })),
            Span::styled(" | Ambient: ", Style::default().fg(Color::DarkGray)),
            Span::styled(ambient_lights.to_uppercase(), Style::default().fg(if ambient_lights == "on" { Color::Green } else { Color::DarkGray })),
        ]),
    ];
    let phone_block = Block::default()
        .title(" 📱 Devices & Sensors ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    f.render_widget(Paragraph::new(phone_text).block(phone_block), top_chunks[2]);

    // Middle: Quick Lights & Quick Controls
    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Quick Lighting
            Constraint::Percentage(40), // Key Quick Actions
        ])
        .split(chunks[1]);

    // Quick Lighting list
    let lights = app.get_lights();
    let light_items: Vec<ListItem> = lights
        .iter()
        .enumerate()
        .map(|(idx, l)| {
            let is_sel = idx == app.overview_selected;
            let on = l.is_on();
            let b_pct = l.brightness_pct().unwrap_or(0);

            let prefix = if is_sel { "▶ " } else { "  " };
            let bulb_icon = if on { "💡" } else { "⚫" };
            let state_str = if on { "ON " } else { "OFF" };
            let state_color = if on { Color::Yellow } else { Color::DarkGray };

            let bar_len = (b_pct / 10).clamp(0, 10) as usize;
            let bar = format!("[{}{}] {:>3}%", "█".repeat(bar_len), "░".repeat(10 - bar_len), b_pct);

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::raw(format!("{} ", bulb_icon)),
                Span::styled(
                    format!("{:<26} ", l.friendly_name()),
                    if is_sel {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(state_str, Style::default().fg(state_color).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(bar, Style::default().fg(if on { Color::Cyan } else { Color::DarkGray })),
            ]);

            ListItem::new(line)
        })
        .collect();

    let lights_block = Block::default()
        .title(" 💡 Quick Lights (Space: Toggle, +/-: Brightness, Tab 2 for more) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(List::new(light_items).block(lights_block), mid_chunks[0]);

    // Key Quick Actions
    let quick_actions = vec![
        Line::from(vec![
            Span::styled("[Space] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle selected light"),
        ]),
        Line::from(vec![
            Span::styled("[a]     ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Turn ALL lights ON"),
        ]),
        Line::from(vec![
            Span::styled("[x]     ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("Turn ALL lights OFF"),
        ]),
        Line::from(vec![
            Span::styled("[f]     ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Find My Phone (Ring sound)"),
        ]),
        Line::from(vec![
            Span::styled("[t]     ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw("Tilt Kinect Motor (Cycle angle)"),
        ]),
        Line::from(vec![
            Span::styled("[/]     ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw("Search / filter current list"),
        ]),
        Line::from(vec![
            Span::styled("[Sft+A] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Assist voice/text command"),
        ]),
        Line::from(vec![
            Span::styled("[r]     ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            Span::raw("Refresh states from HA & Kinect"),
        ]),
        Line::from(vec![
            Span::styled("[1-9]   ", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            Span::raw("Jump directly to specific view tab"),
        ]),
    ];
    let actions_block = Block::default()
        .title(" ⚡ Quick Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightBlue));
    f.render_widget(Paragraph::new(quick_actions).block(actions_block).wrap(Wrap { trim: true }), mid_chunks[1]);

    // Bottom: Recent Activity Log
    let log_items: Vec<ListItem> = app
        .events
        .iter()
        .take(5)
        .map(|ev| {
            let time_str = ev.timestamp.format("%H:%M:%S").to_string();
            let line = Line::from(vec![
                Span::styled(format!("[{}] ", time_str), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<28} ", ev.friendly_name), Style::default().fg(Color::White)),
                Span::styled(format!("{} → ", ev.old_state), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &ev.new_state,
                    Style::default().fg(if ev.new_state == "on" || ev.new_state == "playing" {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let logs_block = Block::default()
        .title(" 📜 Live Activity Stream (Recent Events) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(List::new(log_items).block(logs_block), chunks[2]);
}
