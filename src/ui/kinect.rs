use crate::app::{App, KinectItem};
use crate::model::VOICE_PROFILES;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_kinect(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(54), // Interactive Kinect controls list
            Constraint::Percentage(46), // Telemetry & Voice Transcripts
        ])
        .split(area);

    let items = app.get_kinect_items();
    let selected_item = items.get(app.kinect_selected).copied();
    let k = &app.kinect;

    let reminders_count = app
        .entities
        .get("sensor.kinect_active_reminders")
        .map(|e| e.state.as_str())
        .unwrap_or("0");

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_sel = idx == app.kinect_selected;
            let prefix = if is_sel { "▶ " } else { "  " };

            let cat_color = match item.category() {
                "[VOICE]" => Color::Cyan,
                "[MOTOR]" => Color::Yellow,
                "[CAM]  " => Color::Magenta,
                "[MOTN] " => Color::Green,
                _ => Color::LightBlue,
            };

            let (val_str, val_color) = match item {
                KinectItem::VoiceMute => {
                    if k.voice_responses_enabled {
                        ("🔊 ENABLED".to_string(), Color::Green)
                    } else {
                        ("🔇 MUTED".to_string(), Color::Red)
                    }
                }
                KinectItem::MicMute => {
                    if k.mic_enabled {
                        ("🎙️ ACTIVE".to_string(), Color::Green)
                    } else {
                        ("🚫 MUTED".to_string(), Color::Red)
                    }
                }
                KinectItem::SelectVoice => (
                    format!("🗣️ {} ({})", k.active_voice_name, k.active_voice),
                    Color::Yellow,
                ),
                KinectItem::SpeakText => ("[Enter to type]".to_string(), Color::Cyan),
                KinectItem::TestVoice => ("[Enter to test]".to_string(), Color::LightCyan),
                KinectItem::ContinuousListen => {
                    if k.listening_without_wake_word {
                        (format!("ON ({}s)", k.seconds_remaining), Color::Green)
                    } else {
                        ("OFF (Wake Word)".to_string(), Color::DarkGray)
                    }
                }
                KinectItem::MotorAngle => {
                    let angle = k.tilt_angle;
                    // Map -30..+30 to 0..12
                    let pos = (((angle + 30.0) / 60.0) * 10.0).round().clamp(0.0, 10.0) as usize;
                    let mut bar = String::from("[");
                    for i in 0..=10 {
                        if i == pos {
                            bar.push('●');
                        } else if i == 5 {
                            bar.push('┼');
                        } else {
                            bar.push('─');
                        }
                    }
                    bar.push(']');
                    (format!("{:>+3.0}° {}", angle, bar), Color::Yellow)
                }
                KinectItem::MotorUp => ("[+20° Up]".to_string(), Color::Yellow),
                KinectItem::MotorLevel => ("[  0° Level]".to_string(), Color::Cyan),
                KinectItem::MotorDown => ("[-20° Down]".to_string(), Color::Yellow),
                KinectItem::CameraStream => {
                    if k.camera_streaming {
                        ("📹 LIVE WEBCAM".to_string(), Color::Yellow)
                    } else {
                        ("🛡️ 3D IR DEPTH".to_string(), Color::Green)
                    }
                }
                KinectItem::CameraMode => {
                    let mode_label = match k.camera_mode.as_str() {
                        "thermal" => "🔥 Thermal Heatmap",
                        "depth" => "📐 Depth Grayscale",
                        _ => "🌈 Normal RGB",
                    };
                    (mode_label.to_string(), Color::Magenta)
                }
                KinectItem::MotionLights => {
                    if k.motion_lights_enabled {
                        ("ON".to_string(), Color::Green)
                    } else {
                        ("OFF".to_string(), Color::Red)
                    }
                }
                KinectItem::MotionAutoOff => {
                    if k.auto_off_enabled {
                        ("ON".to_string(), Color::Green)
                    } else {
                        ("OFF".to_string(), Color::Red)
                    }
                }
                KinectItem::MotionSpeech => {
                    if k.motion_speech_enabled {
                        ("ON".to_string(), Color::Green)
                    } else {
                        ("OFF".to_string(), Color::Red)
                    }
                }
                KinectItem::OcarinaMode => {
                    if k.ocarina_enabled {
                        ("🎵 ON".to_string(), Color::Green)
                    } else {
                        ("OFF".to_string(), Color::DarkGray)
                    }
                }
                KinectItem::SecurityMode => {
                    if k.security_mode_enabled {
                        ("🚨 ARMED".to_string(), Color::Red)
                    } else {
                        ("OFF".to_string(), Color::DarkGray)
                    }
                }
                KinectItem::ClearReminders => (
                    format!("{} pending", reminders_count),
                    if reminders_count != "0" {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    },
                ),
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{} ", item.category()), Style::default().fg(cat_color)),
                Span::styled(
                    format!("{:<28} ", item.label()),
                    if is_sel {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(
                    val_str,
                    Style::default().fg(val_color).add_modifier(Modifier::BOLD),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .title(" 🤖 Kinect Management (Space/Enter: Toggle/Act | ←/→: Adjust | v: Voices | m: Mute) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(List::new(list_items).block(list_block), chunks[0]);

    // Right pane: Telemetry + Voice Transcripts
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14), // Telemetry & Selected Item Details
            Constraint::Min(8),     // Live Voice Transcripts & Shortcuts
        ])
        .split(chunks[1]);

    let voice_desc = VOICE_PROFILES
        .iter()
        .find(|(key, _, _)| *key == k.active_voice)
        .map(|(_, _, desc)| *desc)
        .unwrap_or("Neural TTS Voice");

    let mut telemetry_lines = vec![
        Line::from(vec![
            Span::styled("Voice Output: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if k.voice_responses_enabled {
                    "🔊 ENABLED"
                } else {
                    "🔇 MUTED"
                },
                Style::default()
                    .fg(if k.voice_responses_enabled {
                        Color::Green
                    } else {
                        Color::Red
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Mic Array: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if k.mic_enabled {
                    "🎙️ LISTENING"
                } else {
                    "🚫 MUTED"
                },
                Style::default()
                    .fg(if k.mic_enabled { Color::Green } else { Color::Red })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Active Voice: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &k.active_voice_name,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" — {}", voice_desc), Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Camera Tilt:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:+.0}°", k.tilt_angle),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Sensor Mode: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if k.camera_streaming {
                    format!("Live Stream ({})", k.camera_mode.to_uppercase())
                } else {
                    "3D IR Depth (0 Lux Motion)".to_string()
                },
                Style::default().fg(if k.camera_streaming {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Motion Mode:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if k.is_night_mode {
                    format!("Night ({} px)", k.effective_min_pixels)
                } else {
                    format!("Day ({} px)", k.effective_min_pixels)
                },
                Style::default().fg(Color::LightBlue),
            ),
            Span::raw("   "),
            Span::styled("Auto-Off In: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                k.seconds_until_auto_off
                    .map(|s| format!("{:.0}m", s / 60.0))
                    .unwrap_or_else(|| "N/A".to_string()),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(sel) = selected_item {
        telemetry_lines.push(Line::from(vec![
            Span::styled("Selected: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(sel.label(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));
        telemetry_lines.push(Line::from(Span::styled(
            format!("  {}", sel.description()),
            Style::default().fg(Color::Gray),
        )));
    }

    telemetry_lines.push(Line::from(""));
    telemetry_lines.push(Line::from(vec![
        Span::styled("Quick Keys: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("[m] Mute TTS  [M] Mute Mic  [v] Pick Voice  [+/-] Tilt ±5°  [0] Level 0°"),
    ]));

    let telemetry_block = Block::default()
        .title(" 📊 Kinect Hardware & Voice Telemetry ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(
        Paragraph::new(telemetry_lines)
            .block(telemetry_block)
            .wrap(Wrap { trim: true }),
        right_chunks[0],
    );

    // Bottom right: Live Voice Transcripts
    let mut transcript_lines = vec![
        Line::from(vec![
            Span::styled("Last Heard:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if k.last_heard.is_empty() {
                    "<none>"
                } else {
                    &k.last_heard
                },
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Matched:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if k.last_matched_line.is_empty() {
                    "<none>"
                } else {
                    &k.last_matched_line
                },
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled(
                if k.last_intent.is_empty() {
                    "".to_string()
                } else {
                    format!("({})", k.last_intent)
                },
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Recent Speech Transcripts:",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )]),
    ];

    if k.transcript_lines.is_empty() {
        transcript_lines.push(Line::from(Span::styled(
            "  No transcripts recorded yet.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for t_line in k.transcript_lines.iter().take(8) {
            let color = if t_line.contains("wake_word_required") {
                Color::DarkGray
            } else {
                Color::LightGreen
            };
            transcript_lines.push(Line::from(Span::styled(
                format!("  {}", t_line),
                Style::default().fg(color),
            )));
        }
    }

    let transcript_block = Block::default()
        .title(" 🎙️ Live Voice Recognition Feed ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(
        Paragraph::new(transcript_lines)
            .block(transcript_block)
            .wrap(Wrap { trim: true }),
        right_chunks[1],
    );
}
