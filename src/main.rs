mod app;
mod client;
mod config;
mod model;
mod ui;

use app::{ActiveModal, App, KinectItem, Tab};
use client::{ClientEvent, HaClient};
use config::Config;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::StreamExt;
use model::VOICE_PROFILES;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load configuration
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration Error: {}", e);
            eprintln!("Please set HASS_URL and HASS_TOKEN via environment variables, CLI, or .env file.");
            std::process::exit(1);
        }
    };

    if config.token.is_empty() {
        eprintln!("Warning: No Home Assistant Long-Lived Access Token provided.");
        eprintln!("Provide via HASS_TOKEN or --token or in .env file.");
    }

    // 2. Setup terminal with panic hook to restore state
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Initialize HA client & state
    let client = Arc::new(HaClient::new(config.url.clone(), config.token.clone())?);
    let mut app = App::new(client.clone());

    // Channel for asynchronous client events (WS updates, REST loads)
    let (tx, mut rx) = mpsc::unbounded_channel::<ClientEvent>();

    // Initial fetch of all entities & Kinect status via REST
    let init_client = client.clone();
    let init_tx = tx.clone();
    tokio::spawn(async move {
        if let Ok(k_status) = init_client.fetch_kinect_status(0.0).await {
            let _ = init_tx.send(ClientEvent::KinectUpdated(k_status));
        }

        match init_client.fetch_states().await {
            Ok(states) => {
                let _ = init_tx.send(ClientEvent::FullStates(states));
            }
            Err(e) => {
                let _ = init_tx.send(ClientEvent::WsDisconnected(format!("Initial fetch failed: {}", e)));
            }
        }
    });

    // Start WebSocket real-time listener
    client.clone().start_websocket_listener(tx.clone());

    // Setup input stream & timers
    let mut reader = EventStream::new();
    let mut poll_interval = tokio::time::interval(tokio::time::Duration::from_secs(config.poll_interval));
    let mut kinect_interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
    let mut toast_timer = tokio::time::interval(tokio::time::Duration::from_millis(500));

    // 4. Main Event Loop
    while !app.should_quit {
        // Draw UI
        terminal.draw(|f| ui::render(f, &app))?;

        tokio::select! {
            // Periodic entity refresh via REST fallback
            _ = poll_interval.tick() => {
                let poll_client = client.clone();
                let poll_tx = tx.clone();
                tokio::spawn(async move {
                    if let Ok(states) = poll_client.fetch_states().await {
                        let _ = poll_tx.send(ClientEvent::FullStates(states));
                    }
                });
            }

            // Periodic Kinect status refresh
            _ = kinect_interval.tick() => {
                let k_client = client.clone();
                let k_tx = tx.clone();
                let cur_tilt = app.current_ha_tilt_angle();
                tokio::spawn(async move {
                    if let Ok(st) = k_client.fetch_kinect_status(cur_tilt).await {
                        let _ = k_tx.send(ClientEvent::KinectUpdated(st));
                    }
                });
            }

            // Toast cleanup timer
            _ = toast_timer.tick() => {
                app.clean_expired_toasts();
            }

            // Async Client events (WebSocket updates, REST responses)
            Some(client_event) = rx.recv() => {
                app.handle_client_event(client_event);
            }

            // Keyboard and terminal events
            Some(Ok(event)) = reader.next() => {
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press {
                        handle_key_event(&mut app, key.code, key.modifiers, &tx);
                    }
                }
            }
        }
    }

    // 5. Cleanup terminal on exit
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn spawn_kinect_refresh(app: &App, tx: &mpsc::UnboundedSender<ClientEvent>) {
    let client = app.client.clone();
    let tx_clone = tx.clone();
    let cur_tilt = app.kinect.tilt_angle;
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;
        if let Ok(st) = client.fetch_kinect_status(cur_tilt).await {
            let _ = tx_clone.send(ClientEvent::KinectUpdated(st));
        }
    });
}

fn handle_key_event(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    tx: &mpsc::UnboundedSender<ClientEvent>,
) {
    // 1. Modals Input Handling
    if app.active_modal != ActiveModal::None {
        handle_modal_key(app, code, tx);
        return;
    }

    // 2. Universal Search / Filter input mode across all tabs
    if app.is_searching {
        match code {
            KeyCode::Esc => {
                app.is_searching = false;
                app.search_query.clear();
                app.clamp_selections();
            }
            KeyCode::Enter => {
                app.is_searching = false;
                app.clamp_selections();
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.clamp_selections();
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.clamp_selections();
            }
            _ => {}
        }
        return;
    }

    // 3. Global Navigation & Hotkeys
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    match code {
        // Esc clears active search filter if one is locked in
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                app.search_query.clear();
                app.clamp_selections();
                app.add_toast("Cleared search filter", false);
            }
        }

        // Quit
        KeyCode::Char('q') => {
            app.should_quit = true;
        }

        // Help
        KeyCode::Char('?') | KeyCode::Char('h') => {
            app.active_modal = ActiveModal::Help;
        }

        // Direct Tab jumps (1-9)
        KeyCode::Char('1') => app.current_tab = Tab::Overview,
        KeyCode::Char('2') => app.current_tab = Tab::Lights,
        KeyCode::Char('3') => app.current_tab = Tab::Kinect,
        KeyCode::Char('4') => app.current_tab = Tab::Switches,
        KeyCode::Char('5') => app.current_tab = Tab::Buttons,
        KeyCode::Char('6') => app.current_tab = Tab::Media,
        KeyCode::Char('7') => app.current_tab = Tab::Sensors,
        KeyCode::Char('8') => app.current_tab = Tab::Entities,
        KeyCode::Char('9') => {
            if app.current_tab == Tab::Lights {
                set_selected_brightness(app, 100);
            } else {
                app.current_tab = Tab::Events;
            }
        }

        // Tab navigation
        KeyCode::Tab => {
            app.current_tab = app.current_tab.next();
        }
        KeyCode::BackTab => {
            app.current_tab = app.current_tab.prev();
        }

        // List item navigation
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev();
        }

        // Manual refresh
        KeyCode::Char('r') => {
            let client = app.client.clone();
            let tx_clone = tx.clone();
            let cur_tilt = app.current_ha_tilt_angle();
            tokio::spawn(async move {
                if let Ok(k_st) = client.fetch_kinect_status(cur_tilt).await {
                    let _ = tx_clone.send(ClientEvent::KinectUpdated(k_st));
                }
                if let Ok(st) = client.fetch_states().await {
                    let _ = tx_clone.send(ClientEvent::FullStates(st));
                }
            });
            app.add_toast("Refreshing all states from Home Assistant & Kinect...", false);
        }

        // Search / Filter current list on ANY tab
        KeyCode::Char('/') => {
            app.is_searching = true;
        }

        // Assist Voice/Text command prompt (Shift+A)
        KeyCode::Char('A') => {
            app.active_modal = ActiveModal::Assist {
                input: String::new(),
                response: None,
                loading: false,
            };
        }

        // Domain filter cycle (Tab 8: Entities)
        KeyCode::Char('d') => {
            if app.current_tab == Tab::Entities {
                let domains = [
                    None,
                    Some("light".into()),
                    Some("switch".into()),
                    Some("sensor".into()),
                    Some("binary_sensor".into()),
                    Some("button".into()),
                    Some("script".into()),
                    Some("automation".into()),
                    Some("media_player".into()),
                    Some("weather".into()),
                ];
                let cur_pos = domains.iter().position(|d| *d == app.domain_filter).unwrap_or(0);
                app.domain_filter = domains[(cur_pos + 1) % domains.len()].clone();
                app.entities_selected = 0;
            }
        }

        // Voice Picker shortcut (on Kinect tab)
        KeyCode::Char('v') => {
            if app.current_tab == Tab::Kinect {
                app.active_modal = ActiveModal::VoicePicker {
                    selected_idx: app.active_voice_index(),
                    announce: true,
                };
            }
        }

        // Toggle action (Space)
        KeyCode::Char(' ') => {
            match app.current_tab {
                Tab::Overview | Tab::Lights => {
                    if let Some(eid) = app.get_selected_entity_id() {
                        let client = app.client.clone();
                        let target = eid.clone();
                        tokio::spawn(async move {
                            let _ = client.toggle_entity(&target).await;
                        });
                        app.add_toast(format!("Toggled {}", eid), false);
                    }
                }
                Tab::Kinect => {
                    execute_selected_kinect_item(app, tx);
                }
                Tab::Switches => {
                    if let Some(eid) = app.get_selected_entity_id() {
                        let client = app.client.clone();
                        let target = eid.clone();
                        tokio::spawn(async move {
                            let _ = client.toggle_entity(&target).await;
                        });
                        app.add_toast(format!("Toggled {}", eid), false);
                    }
                }
                Tab::Buttons => {
                    execute_selected_button_or_script(app);
                }
                Tab::Media => {
                    if let Some(eid) = app.get_selected_entity_id() {
                        let client = app.client.clone();
                        let target = eid.clone();
                        tokio::spawn(async move {
                            let _ = client.media_play_pause(&target).await;
                        });
                        app.add_toast(format!("Play/Pause {}", eid), false);
                    }
                }
                Tab::Entities => {
                    if let Some(eid) = app.get_selected_entity_id() {
                        let client = app.client.clone();
                        let target = eid.clone();
                        tokio::spawn(async move {
                            let _ = client.toggle_entity(&target).await;
                        });
                        app.add_toast(format!("Toggled {}", eid), false);
                    }
                }
                _ => {}
            }
        }

        // Enter action
        KeyCode::Enter => {
            match app.current_tab {
                Tab::Kinect => {
                    execute_selected_kinect_item(app, tx);
                }
                Tab::Buttons => {
                    execute_selected_button_or_script(app);
                }
                Tab::Entities | Tab::Switches | Tab::Lights => {
                    if let Some(eid) = app.get_selected_entity_id() {
                        app.active_modal = ActiveModal::EntityDetail { entity_id: eid };
                    }
                }
                _ => {}
            }
        }

        // Call Service Modal trigger
        KeyCode::Char('s') => {
            if let Some(eid) = app.get_selected_entity_id() {
                let domain = eid.split('.').next().unwrap_or("homeassistant").to_string();
                app.active_modal = ActiveModal::ServiceCall {
                    entity_id: eid,
                    domain,
                    service: "toggle".to_string(),
                    data_json: "{}".to_string(),
                    active_field: 0,
                };
            }
        }

        // Color Picker Modal trigger
        KeyCode::Char('c') => {
            if let Some(eid) = app.get_selected_entity_id() {
                if eid.starts_with("light.") {
                    app.active_modal = ActiveModal::ColorPicker {
                        entity_id: eid,
                        selected_idx: 0,
                    };
                }
            }
        }

        // Master Lights On
        KeyCode::Char('a') => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.call_service("light", "turn_on", Some("light.all_lights"), None).await;
            });
            app.add_toast("Turned ALL lights ON", false);
        }

        // Master Lights Off
        KeyCode::Char('x') => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.call_service("light", "turn_off", Some("light.all_lights"), None).await;
            });
            app.add_toast("Turned ALL lights OFF", false);
        }

        // Find My Phone shortcut
        KeyCode::Char('f') => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.press_button("button.find_my_phone").await;
            });
            app.add_toast("Ringing Beck's Phone...", false);
        }

        // Tilt Kinect Motor Level shortcut
        KeyCode::Char('t') => {
            set_kinect_tilt(app, tx, 0.0);
        }

        // Mute shortcut (Kinect TTS on Kinect tab, or Media Mute on Media tab)
        KeyCode::Char('m') => {
            if app.current_tab == Tab::Kinect {
                let client = app.client.clone();
                let tx_clone = tx.clone();
                let cur_tilt = app.kinect.tilt_angle;
                tokio::spawn(async move {
                    let _ = client.kinect_toggle_speech().await;
                    if let Ok(st) = client.fetch_kinect_status(cur_tilt).await {
                        let _ = tx_clone.send(ClientEvent::KinectUpdated(st));
                    }
                });
                let next_state = if app.kinect.voice_responses_enabled { "MUTED" } else { "ENABLED" };
                app.add_toast(format!("Kinect Voice TTS Responses {}", next_state), false);
            } else if let Some(eid) = app.get_selected_entity_id() {
                if eid.starts_with("media_player.") {
                    let is_muted = app.entities.get(&eid).map(|e| e.is_muted()).unwrap_or(false);
                    let client = app.client.clone();
                    let target = eid.clone();
                    tokio::spawn(async move {
                        let _ = client.media_mute(&target, !is_muted).await;
                    });
                    app.add_toast(format!("Mute toggled for {}", eid), false);
                }
            }
        }

        // Kinect Microphone Array Mute toggle (Shift+M)
        KeyCode::Char('M') => {
            let client = app.client.clone();
            let tx_clone = tx.clone();
            let cur_tilt = app.kinect.tilt_angle;
            tokio::spawn(async move {
                let _ = client.kinect_toggle_mic().await;
                if let Ok(st) = client.fetch_kinect_status(cur_tilt).await {
                    let _ = tx_clone.send(ClientEvent::KinectUpdated(st));
                }
            });
            let next_state = if app.kinect.mic_enabled { "MUTED" } else { "ACTIVE" };
            app.add_toast(format!("Kinect Microphone Array {}", next_state), false);
        }

        // Media Power Toggle
        KeyCode::Char('p') => {
            if let Some(eid) = app.get_selected_entity_id() {
                if eid.starts_with("media_player.") {
                    let client = app.client.clone();
                    let target = eid.clone();
                    tokio::spawn(async move {
                        let _ = client.call_service("media_player", "toggle", Some(&target), None).await;
                    });
                    app.add_toast(format!("Power toggle sent to {}", eid), false);
                }
            }
        }

        // PS5 Shutdown / Rest Mode
        KeyCode::Char('o') => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.press_button("button.turn_off_ps5_now").await;
            });
            app.add_toast("Sent PS5 Shutdown / Rest mode command", false);
        }

        // Brightness / Volume / Kinect Tilt Up
        KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Right => {
            if app.current_tab == Tab::Kinect {
                if app.get_selected_kinect_item() == Some(KinectItem::CameraMode) {
                    cycle_kinect_camera_mode(app, tx);
                } else {
                    let new_angle = (app.kinect.tilt_angle + 5.0).clamp(-30.0, 30.0);
                    set_kinect_tilt(app, tx, new_angle);
                }
            } else {
                adjust_selected_level(app, true);
            }
        }

        // Brightness / Volume / Kinect Tilt Down
        KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Left => {
            if app.current_tab == Tab::Kinect {
                if app.get_selected_kinect_item() == Some(KinectItem::CameraMode) {
                    cycle_kinect_camera_mode(app, tx);
                } else {
                    let new_angle = (app.kinect.tilt_angle - 5.0).clamp(-30.0, 30.0);
                    set_kinect_tilt(app, tx, new_angle);
                }
            } else {
                adjust_selected_level(app, false);
            }
        }

        // Quick numeric preset 0 (Level motor on Kinect tab, or 0% brightness on Lights tab)
        KeyCode::Char('0') => {
            if app.current_tab == Tab::Kinect {
                set_kinect_tilt(app, tx, 0.0);
            } else {
                set_selected_brightness(app, 0);
            }
        }

        // Quick color presets
        KeyCode::Char('w') => {
            // Warm white
            if let Some(eid) = app.get_selected_entity_id() {
                if eid.starts_with("light.") {
                    let client = app.client.clone();
                    let target = eid.clone();
                    tokio::spawn(async move {
                        let _ = client.set_light_color_rgb(&target, 255, 180, 107).await;
                    });
                    app.add_toast(format!("Warm White applied to {}", eid), false);
                }
            }
        }

        _ => {}
    }
}

fn execute_selected_kinect_item(app: &mut App, tx: &mpsc::UnboundedSender<ClientEvent>) {
    let Some(item) = app.get_selected_kinect_item() else {
        return;
    };

    match item {
        KinectItem::VoiceMute => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_toggle_speech().await;
            });
            let next = if app.kinect.voice_responses_enabled { "MUTED" } else { "ENABLED" };
            app.add_toast(format!("Kinect Voice TTS Responses {}", next), false);
            spawn_kinect_refresh(app, tx);
        }
        KinectItem::MicMute => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_toggle_mic().await;
            });
            let next = if app.kinect.mic_enabled { "MUTED" } else { "ACTIVE" };
            app.add_toast(format!("Kinect Microphone Array {}", next), false);
            spawn_kinect_refresh(app, tx);
        }
        KinectItem::SelectVoice => {
            app.active_modal = ActiveModal::VoicePicker {
                selected_idx: app.active_voice_index(),
                announce: true,
            };
        }
        KinectItem::ContinuousListen => {
            let currently_listening = app.kinect.listening_without_wake_word;
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_toggle_listen(currently_listening).await;
            });
            app.add_toast(
                if !currently_listening {
                    "Active conversation window enabled (no wake word needed)"
                } else {
                    "Returned to standard wake-word mode"
                },
                false,
            );
            spawn_kinect_refresh(app, tx);
        }
        KinectItem::SpeakText => {
            app.active_modal = ActiveModal::SpeakPrompt {
                input: String::new(),
            };
        }
        KinectItem::TestVoice => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_speak("Kinect audio check. Voice output is operational.").await;
            });
            app.add_toast("Playing Kinect test voice phrase...", false);
        }
        KinectItem::MotorUp => set_kinect_tilt(app, tx, 20.0),
        KinectItem::MotorLevel => set_kinect_tilt(app, tx, 0.0),
        KinectItem::MotorDown => set_kinect_tilt(app, tx, -20.0),
        KinectItem::MotorAngle => {
            // Cycle -20 -> -10 -> 0 -> 10 -> 20 -> -20 on Enter
            let next = if app.kinect.tilt_angle < -15.0 {
                -10.0
            } else if app.kinect.tilt_angle < -5.0 {
                0.0
            } else if app.kinect.tilt_angle < 5.0 {
                10.0
            } else if app.kinect.tilt_angle < 15.0 {
                20.0
            } else {
                -20.0
            };
            set_kinect_tilt(app, tx, next);
        }
        KinectItem::CameraStream => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_toggle_camera_stream().await;
            });
            let next = if app.kinect.camera_streaming {
                "3D IR Depth Mode"
            } else {
                "Live Camera Feed"
            };
            app.add_toast(format!("Switched Kinect optical sensor to {}", next), false);
            spawn_kinect_refresh(app, tx);
        }
        KinectItem::CameraMode => {
            cycle_kinect_camera_mode(app, tx);
        }
        KinectItem::MotionLights => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_toggle_motion_lights().await;
            });
            app.add_toast("Toggled Kinect Motion Auto-Lights", false);
            spawn_kinect_refresh(app, tx);
        }
        KinectItem::MotionAutoOff => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_toggle_auto_off().await;
            });
            app.add_toast("Toggled 30m Inactivity Auto-Off Timer", false);
            spawn_kinect_refresh(app, tx);
        }
        KinectItem::MotionSpeech => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_toggle_motion_speech().await;
            });
            app.add_toast("Toggled Motion Greeting Speech", false);
            spawn_kinect_refresh(app, tx);
        }
        KinectItem::OcarinaMode => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_toggle_ocarina().await;
            });
            app.add_toast("Toggled Ocarina Audio Recognition", false);
            spawn_kinect_refresh(app, tx);
        }
        KinectItem::SecurityMode => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_toggle_security().await;
            });
            app.add_toast("Toggled Security / Intruder Alert Mode", false);
            spawn_kinect_refresh(app, tx);
        }
        KinectItem::ClearReminders => {
            let client = app.client.clone();
            tokio::spawn(async move {
                let _ = client.kinect_clear_reminders().await;
            });
            app.add_toast("Cleared all pending voice reminders", false);
            spawn_kinect_refresh(app, tx);
        }
    }
}

fn set_kinect_tilt(app: &mut App, tx: &mpsc::UnboundedSender<ClientEvent>, angle: f64) {
    let clamped = angle.clamp(-30.0, 30.0);
    app.kinect.tilt_angle = clamped;
    let client = app.client.clone();
    tokio::spawn(async move {
        let _ = client.kinect_set_tilt(clamped).await;
    });
    app.add_toast(format!("Tilting Kinect Motor to {:+.0}°", clamped), false);
    spawn_kinect_refresh(app, tx);
}

fn cycle_kinect_camera_mode(app: &mut App, tx: &mpsc::UnboundedSender<ClientEvent>) {
    let next_mode = match app.kinect.camera_mode.as_str() {
        "rgb" => "thermal",
        "thermal" => "depth",
        _ => "rgb",
    };
    app.kinect.camera_mode = next_mode.to_string();
    let client = app.client.clone();
    let mode_str = next_mode.to_string();
    tokio::spawn(async move {
        let _ = client.kinect_set_camera_mode(&mode_str).await;
    });
    app.add_toast(format!("Set Kinect Camera View Mode to {}", next_mode.to_uppercase()), false);
    spawn_kinect_refresh(app, tx);
}

fn handle_modal_key(
    app: &mut App,
    code: KeyCode,
    tx: &mpsc::UnboundedSender<ClientEvent>,
) {
    let mut close_modal = false;
    let mut toast_to_add: Option<(String, bool)> = None;
    let mut trigger_kinect_refresh = false;

    match &mut app.active_modal {
        ActiveModal::Help | ActiveModal::EntityDetail { .. } => {
            if matches!(code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter) {
                close_modal = true;
            }
        }
        ActiveModal::ColorPicker { entity_id, selected_idx } => {
            match code {
                KeyCode::Esc => {
                    close_modal = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if *selected_idx == 0 {
                        *selected_idx = ui::modals::COLOR_PRESETS.len() - 1;
                    } else {
                        *selected_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected_idx = (*selected_idx + 1) % ui::modals::COLOR_PRESETS.len();
                }
                KeyCode::Enter => {
                    let (_, (r, g, b)) = ui::modals::COLOR_PRESETS[*selected_idx];
                    let eid = entity_id.clone();
                    let client = app.client.clone();
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = client.set_light_color_rgb(&eid, r, g, b).await {
                            let _ = tx_clone.send(ClientEvent::WsDisconnected(format!("Color failed: {}", e)));
                        }
                    });
                    toast_to_add = Some((format!("Applied color to {}", entity_id), false));
                    close_modal = true;
                }
                _ => {}
            }
        }
        ActiveModal::VoicePicker {
            selected_idx,
            announce,
        } => {
            match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    close_modal = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if *selected_idx == 0 {
                        *selected_idx = VOICE_PROFILES.len() - 1;
                    } else {
                        *selected_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *selected_idx = (*selected_idx + 1) % VOICE_PROFILES.len();
                }
                KeyCode::Char(' ') => {
                    *announce = !*announce;
                }
                KeyCode::Enter => {
                    let should_announce = *announce;
                    let (key, name, _) = VOICE_PROFILES[*selected_idx];
                    app.kinect.active_voice = key.to_string();
                    app.kinect.active_voice_name = name.to_string();
                    let client = app.client.clone();
                    let voice_key = key.to_string();
                    tokio::spawn(async move {
                        let _ = client.kinect_set_voice(&voice_key, should_announce).await;
                    });
                    toast_to_add = Some((
                        format!(
                            "Kinect Voice set to {} ({})",
                            name,
                            if should_announce { "with greeting" } else { "silent switch" }
                        ),
                        false,
                    ));
                    trigger_kinect_refresh = true;
                    close_modal = true;
                }
                _ => {}
            }
        }
        ActiveModal::SpeakPrompt { input } => {
            match code {
                KeyCode::Esc => {
                    close_modal = true;
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Enter => {
                    if !input.trim().is_empty() {
                        let text = input.clone();
                        let client = app.client.clone();
                        tokio::spawn(async move {
                            let _ = client.kinect_speak(&text).await;
                        });
                        toast_to_add = Some((format!("Kinect speaking: \"{}\"", input), false));
                        close_modal = true;
                    }
                }
                _ => {}
            }
        }
        ActiveModal::Assist {
            input,
            response,
            loading,
        } => {
            match code {
                KeyCode::Esc => {
                    close_modal = true;
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Enter => {
                    if !input.trim().is_empty() && !*loading {
                        *loading = true;
                        let text = input.clone();
                        let client = app.client.clone();
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            let res = client.assist(&text).await;
                            match res {
                                Ok(_speech) => {
                                    if let Ok(st) = client.fetch_states().await {
                                        let _ = tx_clone.send(ClientEvent::FullStates(st));
                                    }
                                }
                                Err(e) => {
                                    let _ = tx_clone.send(ClientEvent::WsDisconnected(format!("Assist failed: {}", e)));
                                }
                            }
                        });
                        *response = Some(format!("Executed: \"{}\"", input));
                        *loading = false;
                        toast_to_add = Some((format!("Assist command sent: {}", input), false));
                    }
                }
                _ => {}
            }
        }
        ActiveModal::ServiceCall {
            entity_id,
            domain,
            service,
            data_json,
            active_field,
        } => {
            match code {
                KeyCode::Esc => {
                    close_modal = true;
                }
                KeyCode::Tab => {
                    *active_field = (*active_field + 1) % 2;
                }
                KeyCode::Backspace => {
                    if *active_field == 0 {
                        service.pop();
                    } else {
                        data_json.pop();
                    }
                }
                KeyCode::Char(c) => {
                    if *active_field == 0 {
                        service.push(c);
                    } else {
                        data_json.push(c);
                    }
                }
                KeyCode::Enter => {
                    let d = domain.clone();
                    let s = service.clone();
                    let eid = entity_id.clone();
                    let payload: Option<serde_json::Value> = serde_json::from_str(data_json).ok();
                    let client = app.client.clone();
                    tokio::spawn(async move {
                        let _ = client.call_service(&d, &s, Some(&eid), payload).await;
                    });
                    toast_to_add = Some((format!("Called {}.{} on {}", domain, service, entity_id), false));
                    close_modal = true;
                }
                _ => {}
            }
        }
        ActiveModal::None => {}
    }

    if close_modal {
        app.active_modal = ActiveModal::None;
    }
    if let Some((msg, is_err)) = toast_to_add {
        app.add_toast(msg, is_err);
    }
    if trigger_kinect_refresh {
        spawn_kinect_refresh(app, tx);
    }
}

fn execute_selected_button_or_script(app: &mut App) {
    if let Some(eid) = app.get_selected_entity_id() {
        let client = app.client.clone();
        let target = eid.clone();
        tokio::spawn(async move {
            if target.starts_with("button.") {
                let _ = client.press_button(&target).await;
            } else if target.starts_with("script.") {
                let _ = client.run_script(&target).await;
            }
        });
        app.add_toast(format!("Executed {}", eid), false);
    }
}

fn adjust_selected_level(app: &mut App, up: bool) {
    if let Some(eid) = app.get_selected_entity_id() {
        if eid.starts_with("light.") {
            let cur_pct = app
                .entities
                .get(&eid)
                .and_then(|e| e.brightness_pct())
                .unwrap_or(50);
            let new_pct = if up {
                (cur_pct + 10).min(100)
            } else {
                cur_pct.saturating_sub(10)
            };
            let client = app.client.clone();
            let target = eid.clone();
            tokio::spawn(async move {
                let _ = client.set_light_brightness(&target, new_pct).await;
            });
            app.add_toast(format!("{} brightness set to {}%", eid, new_pct), false);
        } else if eid.starts_with("media_player.") {
            let client = app.client.clone();
            let target = eid.clone();
            tokio::spawn(async move {
                let _ = client.media_volume_step(&target, up).await;
            });
            app.add_toast(format!("{} volume {}", eid, if up { "up" } else { "down" }), false);
        }
    }
}

fn set_selected_brightness(app: &mut App, pct: u8) {
    if let Some(eid) = app.get_selected_entity_id() {
        if eid.starts_with("light.") {
            let client = app.client.clone();
            let target = eid.clone();
            tokio::spawn(async move {
                if pct == 0 {
                    let _ = client.turn_off(&target).await;
                } else {
                    let _ = client.set_light_brightness(&target, pct).await;
                }
            });
            app.add_toast(format!("{} brightness set to {}%", eid, pct), false);
        }
    }
}
