use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn render_sensors(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Storage pools gauges
            Constraint::Length(10), // Jellyfin & Kinect system stats
            Constraint::Min(8),     // Mobile & Gaming / PSN stats
        ])
        .split(area);

    // 1. Top Section: Storage Pools on beck-link
    let storage_block = Block::default()
        .title(" 💽 Storage Pools & Capacity (Host: beck-link) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let storage_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Media HDD
            Constraint::Length(1), // TV Brick 0
            Constraint::Length(1), // TV Brick 1
            Constraint::Length(1), // TV Brick 2
            Constraint::Length(1), // Combined Pool
            Constraint::Length(1), // NVMe Root
            Constraint::Min(1),
        ])
        .split(storage_block.inner(chunks[0]));

    f.render_widget(storage_block, chunks[0]);

    render_storage_gauge(f, app, "sensor.jellyfin_media_hdd_usage", "Media HDD (/mnt/media_hdd)", storage_chunks[0]);
    render_storage_gauge(f, app, "sensor.jellyfin_tv_brick_0_usage", "TV Brick 0 (/mnt/.tv_bricks/disk0)", storage_chunks[1]);
    render_storage_gauge(f, app, "sensor.jellyfin_tv_brick_1_usage", "TV Brick 1 (/mnt/.tv_bricks/disk1)", storage_chunks[2]);
    render_storage_gauge(f, app, "sensor.jellyfin_tv_brick_2_usage", "TV Brick 2 (/mnt/.tv_bricks/disk2)", storage_chunks[3]);
    render_storage_gauge(f, app, "sensor.jellyfin_tv_combined_pool_usage", "Combined TV Bricks Pool", storage_chunks[4]);
    render_storage_gauge(f, app, "sensor.jellyfin_system_nvme_root_usage", "NVMe Root (/)", storage_chunks[5]);

    // 2. Middle Section: Jellyfin & Kinect Stats
    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Jellyfin Media Stats
            Constraint::Percentage(50), // Kinect AI Hardware Stats
        ])
        .split(chunks[1]);

    // Jellyfin Media stats
    let j_streams = app.entities.get("sensor.jellyfin_active_streams").map(|e| e.state.as_str()).unwrap_or("0");
    let j_transcodes = app.entities.get("sensor.jellyfin_active_transcodes").map(|e| e.state.as_str()).unwrap_or("0");
    let j_movies = app.entities.get("sensor.jellyfin_movies_count").map(|e| e.state.as_str()).unwrap_or("0");
    let j_shows = app.entities.get("sensor.jellyfin_tv_shows_count").map(|e| e.state.as_str()).unwrap_or("0");
    let j_episodes = app.entities.get("sensor.jellyfin_episodes_count").map(|e| e.state.as_str()).unwrap_or("0");
    let j_books = app.entities.get("sensor.jellyfin_books_count").map(|e| e.state.as_str()).unwrap_or("0");

    let jellyfin_text = vec![
        Line::from(vec![
            Span::styled("Active Streams: ", Style::default().fg(Color::DarkGray)),
            Span::styled(j_streams, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("    "),
            Span::styled("Active Transcodes: ", Style::default().fg(Color::DarkGray)),
            Span::styled(j_transcodes, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Movies Count:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(j_movies, Style::default().fg(Color::Cyan)),
            Span::raw("    "),
            Span::styled("TV Shows:          ", Style::default().fg(Color::DarkGray)),
            Span::styled(j_shows, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Total Episodes: ", Style::default().fg(Color::DarkGray)),
            Span::styled(j_episodes, Style::default().fg(Color::White)),
            Span::raw("    "),
            Span::styled("Audiobooks/Books:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(j_books, Style::default().fg(Color::White)),
        ]),
    ];
    let jellyfin_block = Block::default()
        .title(" 🎬 Jellyfin Media Server (beck-link:8096) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(Paragraph::new(jellyfin_text).block(jellyfin_block), mid_chunks[0]);

    // Kinect AI Hardware stats
    let k_health = app.entities.get("sensor.kinect_health_status").map(|e| e.state.as_str()).unwrap_or("Unknown");
    let k_cam = app.entities.get("sensor.kinect_camera_power_status").map(|e| e.state.as_str()).unwrap_or("Standby (IR Depth)");
    let k_voice = app.entities.get("sensor.kinect_voice_status").map(|e| e.state.as_str()).unwrap_or("Listening");
    let k_motion = app.entities.get("sensor.kinect_motion_lights_status").map(|e| e.state.as_str()).unwrap_or("Active");
    let k_reminders = app.entities.get("sensor.kinect_active_reminders").map(|e| e.state.as_str()).unwrap_or("0");

    let kinect_text = vec![
        Line::from(vec![
            Span::styled("Hardware Health:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(k_health, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Camera Sensor Mode:", Style::default().fg(Color::DarkGray)),
            Span::styled(k_cam, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Voice Recognition: ", Style::default().fg(Color::DarkGray)),
            Span::styled(k_voice, Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled("Motion: ", Style::default().fg(Color::DarkGray)),
            Span::styled(k_motion, Style::default().fg(Color::LightGreen)),
        ]),
        Line::from(vec![
            Span::styled("Active Reminders:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(k_reminders, Style::default().fg(Color::White)),
        ]),
    ];
    let kinect_block = Block::default()
        .title(" 🤖 Kinect Home AI (Host: beckassist) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    f.render_widget(Paragraph::new(kinect_text).block(kinect_block), mid_chunks[1]);

    // 3. Bottom Section: Mobile & PSN Gaming Stats
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Mobile Beckphone
            Constraint::Percentage(50), // PlayStation Network
        ])
        .split(chunks[2]);

    // Mobile Phone
    let b_level = app.entities.get("sensor.beckphone_battery_level").and_then(|e| e.numeric_value()).unwrap_or(0.0) as u16;
    let b_state = app.entities.get("sensor.beckphone_battery_state").map(|e| e.state.as_str()).unwrap_or("unknown");
    let b_charger = app.entities.get("sensor.beckphone_charger_type").map(|e| e.state.as_str()).unwrap_or("none");

    let phone_text = vec![
        Line::from(vec![
            Span::styled("Battery Level: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}%", b_level),
                Style::default().fg(if b_level > 20 { Color::Green } else { Color::Red }).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Battery State: ", Style::default().fg(Color::DarkGray)),
            Span::styled(b_state.to_uppercase(), Style::default().fg(Color::White)),
            Span::raw("   "),
            Span::styled("Charger: ", Style::default().fg(Color::DarkGray)),
            Span::styled(b_charger, Style::default().fg(Color::Yellow)),
        ]),
    ];
    let phone_block = Block::default()
        .title(" 📱 Mobile Device (Beck Phone) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightBlue));
    f.render_widget(Paragraph::new(phone_text).block(phone_block), bottom_chunks[0]);

    // PSN Gaming
    let psn_user = app.entities.get("sensor.tohellandbeck_online_id").map(|e| e.state.as_str()).unwrap_or("tohellandbeck");
    let psn_status = app.entities.get("sensor.tohellandbeck_online_status").map(|e| e.state.as_str()).unwrap_or("offline");
    let psn_game = app.entities.get("sensor.tohellandbeck_now_playing").map(|e| e.state.as_str()).unwrap_or("None");
    let plat = app.entities.get("sensor.tohellandbeck_platinum_trophies").map(|e| e.state.as_str()).unwrap_or("0");
    let gold = app.entities.get("sensor.tohellandbeck_gold_trophies").map(|e| e.state.as_str()).unwrap_or("0");
    let silver = app.entities.get("sensor.tohellandbeck_silver_trophies").map(|e| e.state.as_str()).unwrap_or("0");
    let bronze = app.entities.get("sensor.tohellandbeck_bronze_trophies").map(|e| e.state.as_str()).unwrap_or("0");

    let psn_text = vec![
        Line::from(vec![
            Span::styled("PSN ID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(psn_user, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw("  Status: "),
            Span::styled(psn_status.to_uppercase(), Style::default().fg(if psn_status == "online" { Color::Green } else { Color::DarkGray })),
        ]),
        Line::from(vec![
            Span::styled("Now Playing: ", Style::default().fg(Color::DarkGray)),
            Span::styled(psn_game, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Trophies: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("🏆 Plat: {}  ", plat), Style::default().fg(Color::Cyan)),
            Span::styled(format!("🥇 Gold: {}  ", gold), Style::default().fg(Color::Yellow)),
            Span::styled(format!("🥈 Silver: {}  ", silver), Style::default().fg(Color::White)),
            Span::styled(format!("🥉 Bronze: {}", bronze), Style::default().fg(Color::LightRed)),
        ]),
    ];
    let psn_block = Block::default()
        .title(" 🎮 PlayStation Network ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(Paragraph::new(psn_text).block(psn_block), bottom_chunks[1]);
}

fn render_storage_gauge(f: &mut Frame, app: &App, entity_id: &str, label: &str, area: Rect) {
    let pct = app
        .entities
        .get(entity_id)
        .and_then(|e| e.numeric_value())
        .unwrap_or(0.0)
        .clamp(0.0, 100.0) as u16;

    let color = if pct > 90 {
        Color::Red
    } else if pct > 75 {
        Color::Yellow
    } else {
        Color::Green
    };

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
        .percent(pct)
        .label(format!("{}: {}%", label, pct));

    f.render_widget(gauge, area);
}
