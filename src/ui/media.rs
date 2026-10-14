use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_media(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Media players list
            Constraint::Percentage(50), // Controller panel
        ])
        .split(area);

    let players = app.get_media_players();
    let selected_player = players.get(app.media_selected).copied();

    let list_items: Vec<ListItem> = players
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let is_sel = idx == app.media_selected;
            let prefix = if is_sel { "▶ " } else { "  " };

            let state_str = p.state.to_uppercase();
            let state_color = match p.state.as_str() {
                "playing" => Color::Green,
                "paused" => Color::Yellow,
                "idle" | "standby" => Color::Blue,
                "off" => Color::DarkGray,
                _ => Color::White,
            };

            let app_name = p.app_name().unwrap_or("None");

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{:<26} ", p.friendly_name()),
                    if is_sel {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(format!("{:<8} ", state_str), Style::default().fg(state_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("({})", app_name), Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .title(" 📺 Media Players (Space: Play/Pause | +/-: Volume | m: Mute | p: Power) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(List::new(list_items).block(list_block), chunks[0]);

    // Right pane: Media Player Controller & Now Playing Info
    let ctrl_block = Block::default()
        .title(" Media Controller & Now Playing ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightMagenta));

    let ctrl_text = if let Some(p) = selected_player {
        let title = p.media_title().unwrap_or("No media active");
        let artist = p.media_artist().unwrap_or("");
        let app_name = p.app_name().unwrap_or("None");
        let vol_opt = p.volume_level();
        let muted = p.is_muted();

        let vol_bar = if let Some(vol) = vol_opt {
            let bar_len = (vol / 10).clamp(0, 10) as usize;
            format!("[{}{}] {}%", "█".repeat(bar_len), "░".repeat(10 - bar_len), vol)
        } else {
            "Volume: N/A".to_string()
        };

        vec![
            Line::from(vec![
                Span::styled("Device: ", Style::default().fg(Color::DarkGray)),
                Span::styled(p.friendly_name(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    p.state.to_uppercase(),
                    Style::default().fg(match p.state.as_str() {
                        "playing" => Color::Green,
                        "paused" => Color::Yellow,
                        _ => Color::DarkGray,
                    }).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Source / App: ", Style::default().fg(Color::DarkGray)),
                Span::styled(app_name, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Now Playing: ", Style::default().fg(Color::DarkGray)),
                Span::styled(title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            if !artist.is_empty() {
                Line::from(vec![
                    Span::styled("Artist: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(artist, Style::default().fg(Color::White)),
                ])
            } else {
                Line::from("")
            },
            Line::from(vec![
                Span::styled("Volume: ", Style::default().fg(Color::DarkGray)),
                Span::styled(vol_bar, Style::default().fg(Color::LightCyan)),
                if muted {
                    Span::styled(" [MUTED]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                } else {
                    Span::raw("")
                },
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Controller Commands:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("  [Space]   Play / Pause Toggle"),
            Line::from("  [+] / [→] Volume Up"),
            Line::from("  [-] / [←] Volume Down"),
            Line::from("  [m]       Mute / Unmute Toggle"),
            Line::from("  [p]       Power Toggle"),
            Line::from("  [o]       PS5 Rest Mode / Shutdown"),
        ]
    } else {
        vec![Line::from("No media player selected")]
    };

    f.render_widget(Paragraph::new(ctrl_text).block(ctrl_block).wrap(Wrap { trim: true }), chunks[1]);
}
