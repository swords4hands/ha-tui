use crate::model::{Entity, KinectStatus};
use anyhow::{anyhow, Context, Result};
use futures_util::{SinkExt, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub enum ClientEvent {
    WsConnected,
    WsDisconnected(String),
    StateChanged {
        entity_id: String,
        new_entity: Entity,
        old_state: String,
    },
    FullStates(Vec<Entity>),
    KinectUpdated(KinectStatus),
}

#[derive(Clone)]
pub struct HaClient {
    http: reqwest::Client,
    kinect_http: reqwest::Client,
    base_url: String,
    kinect_url: String,
    token: String,
}

impl HaClient {
    pub fn new(base_url: String, token: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let auth_val = format!("Bearer {}", token);
        let mut auth_header = HeaderValue::from_str(&auth_val)
            .context("Invalid authorization token characters")?;
        auth_header.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_header);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let kinect_http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()?;

        let kinect_url = if base_url.contains(":8123") {
            base_url.replace(":8123", ":8080")
        } else {
            "http://100.91.201.64:8080".to_string()
        };

        Ok(Self {
            http,
            kinect_http,
            base_url,
            kinect_url,
            token,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[allow(dead_code)]
    pub fn kinect_url(&self) -> &str {
        &self.kinect_url
    }

    /// Fetch all entity states via REST
    pub async fn fetch_states(&self) -> Result<Vec<Entity>> {
        let url = format!("{}/api/states", self.base_url);
        let res = self.http.get(&url).send().await?;
        if !res.status().is_success() {
            return Err(anyhow!("Failed to fetch states: HTTP {}", res.status()));
        }
        let entities = res.json::<Vec<Entity>>().await?;
        Ok(entities)
    }

    /// Fetch live Kinect status from port 8080 (/voice/status and /motion/status)
    pub async fn fetch_kinect_status(&self, fallback_tilt: f64) -> Result<KinectStatus> {
        let voice_url = format!("{}/voice/status", self.kinect_url);
        let motion_url = format!("{}/motion/status", self.kinect_url);

        let (voice_res, motion_res) = tokio::join!(
            self.kinect_http.get(&voice_url).send(),
            self.kinect_http.get(&motion_url).send()
        );

        let mut status = KinectStatus {
            tilt_angle: fallback_tilt,
            ..Default::default()
        };

        if let Ok(vr) = voice_res {
            if vr.status().is_success() {
                if let Ok(v) = vr.json::<Value>().await {
                    status.online = true;
                    status.mic_enabled = v.get("mic_enabled").and_then(|x| x.as_bool()).unwrap_or(true);
                    status.voice_responses_enabled = v
                        .get("voice_responses_enabled")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(true);
                    status.active_voice = v
                        .get("active_voice")
                        .and_then(|x| x.as_str())
                        .unwrap_or("rachel")
                        .to_string();
                    status.active_voice_name = v
                        .get("active_voice_name")
                        .and_then(|x| x.as_str())
                        .unwrap_or("Rachel")
                        .to_string();
                    status.listening_without_wake_word = v
                        .get("listening_without_wake_word")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(false);
                    status.seconds_remaining = v
                        .get("seconds_remaining")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0);
                    status.ocarina_enabled = v
                        .get("ocarina_enabled")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(false);
                    status.last_heard = v
                        .get("last_heard")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    status.last_matched_line = v
                        .get("last_matched_line")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    status.last_intent = v
                        .get("last_intent")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();

                    if let Some(arr) = v.get("transcript_lines").and_then(|x| x.as_array()) {
                        status.transcript_lines = arr
                            .iter()
                            .filter_map(|line| line.as_str().map(|s| s.to_string()))
                            .collect();
                    }
                }
            }
        }

        if let Ok(mr) = motion_res {
            if mr.status().is_success() {
                if let Ok(m) = mr.json::<Value>().await {
                    status.online = true;
                    status.motion_lights_enabled = m
                        .get("motion_lights_enabled")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(true);
                    status.auto_off_enabled = m
                        .get("auto_off_enabled")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(true);
                    status.motion_speech_enabled = m
                        .get("motion_speech_enabled")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(true);
                    status.security_mode_enabled = m
                        .get("security_mode_enabled")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(false);
                    status.camera_streaming = m
                        .get("camera_streaming")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(false);
                    status.camera_mode = m
                        .get("camera_mode")
                        .and_then(|x| x.as_str())
                        .unwrap_or("rgb")
                        .to_string();
                    status.is_night_mode = m
                        .get("is_night_mode")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(false);
                    status.effective_min_pixels = m
                        .get("effective_min_pixels")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(450);
                    status.depth_delta_mm = m
                        .get("depth_delta_mm")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(40);
                    status.seconds_since_last_motion = m
                        .get("seconds_since_last_motion")
                        .and_then(|x| x.as_f64());
                    status.seconds_until_auto_off = m
                        .get("seconds_until_auto_off")
                        .and_then(|x| x.as_f64());
                }
            }
        }

        Ok(status)
    }

    /// Call an arbitrary Kinect endpoint via POST
    pub async fn kinect_post(&self, path: &str, body: Option<Value>) -> Result<Value> {
        let url = format!("{}{}", self.kinect_url, path);
        let req = if let Some(b) = body {
            self.kinect_http.post(&url).json(&b)
        } else {
            self.kinect_http.post(&url).json(&json!({}))
        };
        let res = req.send().await?;
        let status = res.status();
        if !status.is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(anyhow!("Kinect API HTTP {} - {}", status, err_text));
        }
        let val = res.json::<Value>().await.unwrap_or_else(|_| json!({"ok": true}));
        Ok(val)
    }

    /// Toggle Kinect TTS voice responses (Mute/Unmute spoken output)
    pub async fn kinect_toggle_speech(&self) -> Result<bool> {
        let v = self.kinect_post("/voice/speech/toggle", None).await?;
        Ok(v.get("voice_responses_enabled").and_then(|x| x.as_bool()).unwrap_or(true))
    }

    /// Toggle Kinect Microphone Array (Mute/Unmute mic input)
    pub async fn kinect_toggle_mic(&self) -> Result<bool> {
        let v = self.kinect_post("/voice/mic/toggle", None).await?;
        Ok(v.get("mic_enabled").and_then(|x| x.as_bool()).unwrap_or(true))
    }

    /// Set Kinect Assistant Voice
    pub async fn kinect_set_voice(&self, voice_key: &str, announce: bool) -> Result<String> {
        let body = json!({
            "voice": voice_key,
            "announce": announce
        });
        let v = self.kinect_post("/voice/set_voice", Some(body)).await?;
        let name = v
            .get("voice")
            .and_then(|vo| vo.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or(voice_key)
            .to_string();
        Ok(name)
    }

    /// Speak custom text through Kinect / Workstation speaker
    pub async fn kinect_speak(&self, text: &str) -> Result<()> {
        let body = json!({ "text": text });
        let _ = self.kinect_post("/speak", Some(body)).await?;
        Ok(())
    }

    /// Tilt Kinect Motor to specific angle (-30 to +30)
    pub async fn kinect_set_tilt(&self, angle: f64) -> Result<f64> {
        let clamped = angle.clamp(-30.0, 30.0);
        let body = json!({ "angle": clamped });
        let _ = self.kinect_post("/motor/tilt", Some(body)).await?;
        // Also update input_number in HA so slider stays in sync
        let _ = self
            .call_service(
                "input_number",
                "set_value",
                Some("input_number.kinect_tilt_angle"),
                Some(json!({ "value": clamped })),
            )
            .await;
        Ok(clamped)
    }

    /// Toggle continuous listen mode (60s no wake word)
    pub async fn kinect_toggle_listen(&self, currently_listening: bool) -> Result<()> {
        if currently_listening {
            let _ = self.kinect_post("/voice/stop_listening", None).await?;
        } else {
            let _ = self.kinect_post("/voice/listen", Some(json!({"seconds": 60}))).await?;
        }
        Ok(())
    }

    /// Toggle Live Camera Stream
    pub async fn kinect_toggle_camera_stream(&self) -> Result<bool> {
        let v = self.kinect_post("/camera/power/toggle", None).await?;
        Ok(v.get("camera_streaming").and_then(|x| x.as_bool()).unwrap_or(false))
    }

    /// Set Camera View Mode ("rgb", "thermal", "depth")
    pub async fn kinect_set_camera_mode(&self, mode: &str) -> Result<String> {
        let path = format!("/camera/mode/{}", mode);
        let v = self.kinect_post(&path, None).await?;
        Ok(v.get("mode").and_then(|x| x.as_str()).unwrap_or(mode).to_string())
    }

    /// Toggle Motion Auto-On Lights
    pub async fn kinect_toggle_motion_lights(&self) -> Result<bool> {
        let v = self.kinect_post("/motion/lights/toggle", None).await?;
        Ok(v.get("motion_lights_enabled").and_then(|x| x.as_bool()).unwrap_or(true))
    }

    /// Toggle Empty Room Auto-Off
    pub async fn kinect_toggle_auto_off(&self) -> Result<bool> {
        let v = self.kinect_post("/motion/auto_off/toggle", None).await?;
        Ok(v.get("auto_off_enabled").and_then(|x| x.as_bool()).unwrap_or(true))
    }

    /// Toggle Motion Greeting Speech
    pub async fn kinect_toggle_motion_speech(&self) -> Result<bool> {
        let v = self.kinect_post("/motion/speech/toggle", None).await?;
        Ok(v.get("motion_speech_enabled").and_then(|x| x.as_bool()).unwrap_or(true))
    }

    /// Toggle Ocarina Detection
    pub async fn kinect_toggle_ocarina(&self) -> Result<bool> {
        let v = self.kinect_post("/ocarina/toggle", None).await?;
        Ok(v.get("ocarina_enabled").and_then(|x| x.as_bool()).unwrap_or(false))
    }

    /// Toggle Security Alarm Mode
    pub async fn kinect_toggle_security(&self) -> Result<bool> {
        let v = self.kinect_post("/security/toggle", None).await?;
        Ok(v.get("security_mode_enabled").and_then(|x| x.as_bool()).unwrap_or(false))
    }

    /// Clear all pending voice reminders
    pub async fn kinect_clear_reminders(&self) -> Result<u64> {
        let v = self.kinect_post("/reminders/clear", None).await?;
        Ok(v.get("cleared_count").and_then(|x| x.as_u64()).unwrap_or(0))
    }

    /// Call an arbitrary Home Assistant service
    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        entity_id: Option<&str>,
        service_data: Option<Value>,
    ) -> Result<()> {
        let url = format!("{}/api/services/{}/{}", self.base_url, domain, service);
        let mut body = service_data.unwrap_or_else(|| json!({}));

        if let Some(eid) = entity_id {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("entity_id".to_string(), json!(eid));
            }
        }

        let res = self.http.post(&url).json(&body).send().await?;
        let status = res.status();
        if !status.is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(anyhow!("Service call failed: HTTP {} - {}", status, err_text));
        }
        Ok(())
    }

    /// Toggle any toggleable entity
    pub async fn toggle_entity(&self, entity_id: &str) -> Result<()> {
        let domain = entity_id.split('.').next().unwrap_or("homeassistant");
        match domain {
            "light" => self.call_service("light", "toggle", Some(entity_id), None).await,
            "switch" => self.call_service("switch", "toggle", Some(entity_id), None).await,
            "automation" => self.call_service("automation", "toggle", Some(entity_id), None).await,
            "input_boolean" => self.call_service("input_boolean", "toggle", Some(entity_id), None).await,
            "media_player" => self.call_service("media_player", "toggle", Some(entity_id), None).await,
            _ => self.call_service("homeassistant", "toggle", Some(entity_id), None).await,
        }
    }

    /// Turn entity ON
    #[allow(dead_code)]
    pub async fn turn_on(&self, entity_id: &str) -> Result<()> {
        let domain = entity_id.split('.').next().unwrap_or("homeassistant");
        let service_domain = match domain {
            "light" | "switch" | "automation" | "input_boolean" => domain,
            _ => "homeassistant",
        };
        self.call_service(service_domain, "turn_on", Some(entity_id), None).await
    }

    /// Turn entity OFF
    pub async fn turn_off(&self, entity_id: &str) -> Result<()> {
        let domain = entity_id.split('.').next().unwrap_or("homeassistant");
        let service_domain = match domain {
            "light" | "switch" | "automation" | "input_boolean" => domain,
            _ => "homeassistant",
        };
        self.call_service(service_domain, "turn_off", Some(entity_id), None).await
    }

    /// Set light brightness (0..100)
    pub async fn set_light_brightness(&self, entity_id: &str, pct: u8) -> Result<()> {
        let brightness_255 = ((pct as f32 / 100.0) * 255.0).round() as u8;
        let data = json!({
            "brightness": brightness_255
        });
        self.call_service("light", "turn_on", Some(entity_id), Some(data)).await
    }

    /// Set light RGB color
    pub async fn set_light_color_rgb(&self, entity_id: &str, r: u8, g: u8, b: u8) -> Result<()> {
        let data = json!({
            "rgb_color": [r, g, b]
        });
        self.call_service("light", "turn_on", Some(entity_id), Some(data)).await
    }

    /// Press a button entity
    pub async fn press_button(&self, entity_id: &str) -> Result<()> {
        self.call_service("button", "press", Some(entity_id), None).await
    }

    /// Trigger a script entity
    pub async fn run_script(&self, entity_id: &str) -> Result<()> {
        let object_id = entity_id.split('.').nth(1).unwrap_or(entity_id);
        self.call_service("script", object_id, None, None).await
    }

    /// Media player play/pause toggle
    pub async fn media_play_pause(&self, entity_id: &str) -> Result<()> {
        self.call_service("media_player", "media_play_pause", Some(entity_id), None).await
    }

    /// Media player volume up/down
    pub async fn media_volume_step(&self, entity_id: &str, up: bool) -> Result<()> {
        let service = if up { "volume_up" } else { "volume_down" };
        self.call_service("media_player", service, Some(entity_id), None).await
    }

    /// Media player mute toggle
    pub async fn media_mute(&self, entity_id: &str, mute: bool) -> Result<()> {
        let data = json!({
            "is_volume_muted": mute
        });
        self.call_service("media_player", "volume_mute", Some(entity_id), Some(data)).await
    }

    /// Natural language Assist via /api/conversation/process
    pub async fn assist(&self, text: &str) -> Result<String> {
        let url = format!("{}/api/conversation/process", self.base_url);
        let body = json!({
            "text": text,
            "language": "en"
        });

        let res = self.http.post(&url).json(&body).send().await?;
        if !res.status().is_success() {
            return Err(anyhow!("Assist failed: HTTP {}", res.status()));
        }

        let resp_json: Value = res.json().await?;
        if let Some(speech) = resp_json
            .get("response")
            .and_then(|r| r.get("speech"))
            .and_then(|s| s.get("plain"))
            .and_then(|p| p.get("speech"))
            .and_then(|sp| sp.as_str())
        {
            Ok(speech.to_string())
        } else {
            Ok("Command executed.".to_string())
        }
    }

    /// Start background WebSocket listener for live real-time state changes
    pub fn start_websocket_listener(
        self: Arc<Self>,
        tx: UnboundedSender<ClientEvent>,
    ) {
        tokio::spawn(async move {
            loop {
                let ws_url = if self.base_url.starts_with("https://") {
                    format!("wss://{}/api/websocket", &self.base_url[8..])
                } else if self.base_url.starts_with("http://") {
                    format!("ws://{}/api/websocket", &self.base_url[7..])
                } else {
                    format!("ws://{}/api/websocket", self.base_url)
                };

                match connect_async(&ws_url).await {
                    Ok((mut ws_stream, _)) => {
                        let mut _authenticated = false;
                        let mut _subscribed = false;

                        while let Some(msg_result) = ws_stream.next().await {
                            let msg = match msg_result {
                                Ok(m) => m,
                                Err(e) => {
                                    let _ = tx.send(ClientEvent::WsDisconnected(e.to_string()));
                                    break;
                                }
                            };

                            if let Message::Text(text) = msg {
                                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                                    let msg_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                    match msg_type {
                                        "auth_required" => {
                                            let auth_msg = json!({
                                                "type": "auth",
                                                "access_token": self.token
                                            });
                                            let _ = ws_stream.send(Message::Text(auth_msg.to_string().into())).await;
                                        }
                                        "auth_ok" => {
                                            _authenticated = true;
                                            let _ = tx.send(ClientEvent::WsConnected);

                                            // Subscribe to state changes
                                            let sub_msg = json!({
                                                "id": 1,
                                                "type": "subscribe_events",
                                                "event_type": "state_changed"
                                            });
                                            let _ = ws_stream.send(Message::Text(sub_msg.to_string().into())).await;
                                            _subscribed = true;
                                        }
                                        "auth_invalid" => {
                                            let reason = v.get("message").and_then(|m| m.as_str()).unwrap_or("Invalid authentication token");
                                            let _ = tx.send(ClientEvent::WsDisconnected(reason.to_string()));
                                            break;
                                        }
                                        "event" => {
                                            if let Some(event_data) = v.get("event").and_then(|e| e.get("data")) {
                                                if let (Some(entity_id), Some(new_state_val)) = (
                                                    event_data.get("entity_id").and_then(|id| id.as_str()),
                                                    event_data.get("new_state"),
                                                ) {
                                                    let old_state_str = event_data
                                                        .get("old_state")
                                                        .and_then(|os| os.get("state"))
                                                        .and_then(|s| s.as_str())
                                                        .unwrap_or("unknown")
                                                        .to_string();

                                                    if let Ok(new_entity) = serde_json::from_value::<Entity>(new_state_val.clone()) {
                                                        let _ = tx.send(ClientEvent::StateChanged {
                                                            entity_id: entity_id.to_string(),
                                                            new_entity,
                                                            old_state: old_state_str,
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            } else if msg.is_close() {
                                let _ = tx.send(ClientEvent::WsDisconnected("Closed by server".into()));
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(ClientEvent::WsDisconnected(e.to_string()));
                    }
                }

                // Wait before reconnecting
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        });
    }
}
