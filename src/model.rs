use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const VOICE_PROFILES: [(&str, &str, &str); 11] = [
    ("rachel", "Rachel", "Calm, friendly American female (standard assistant)"),
    ("daniel", "Daniel", "Deep, authoritative British male (Jarvis style)"),
    ("adam", "Adam", "Deep, narrative American male"),
    ("george", "George", "Warm, captivating British male"),
    ("bella", "Bella", "Soft, warm American female"),
    ("charlie", "Charlie", "Deep, energetic Australian male"),
    ("karen", "Karen", "Expressive American female (ElevenLabs)"),
    ("hal", "HAL 9000", "HAL 9000 AI Core (Heuristically Programmed Computer)"),
    ("glados", "GLaDOS", "Aperture Science AI Core (Local Neural TTS)"),
    ("cave", "Cave Johnson", "Aperture Science CEO & Founder"),
    ("espeak", "Espeak", "Local offline robotic speech"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinectStatus {
    pub online: bool,
    pub mic_enabled: bool,
    pub voice_responses_enabled: bool,
    pub active_voice: String,
    pub active_voice_name: String,
    pub listening_without_wake_word: bool,
    pub seconds_remaining: u64,
    pub ocarina_enabled: bool,
    pub last_heard: String,
    pub last_matched_line: String,
    pub last_intent: String,
    pub transcript_lines: Vec<String>,
    pub motion_lights_enabled: bool,
    pub auto_off_enabled: bool,
    pub motion_speech_enabled: bool,
    pub security_mode_enabled: bool,
    pub camera_streaming: bool,
    pub camera_mode: String,
    pub is_night_mode: bool,
    pub effective_min_pixels: u64,
    pub depth_delta_mm: u64,
    pub seconds_since_last_motion: Option<f64>,
    pub seconds_until_auto_off: Option<f64>,
    pub tilt_angle: f64,
}

impl Default for KinectStatus {
    fn default() -> Self {
        Self {
            online: false,
            mic_enabled: true,
            voice_responses_enabled: true,
            active_voice: "rachel".to_string(),
            active_voice_name: "Rachel".to_string(),
            listening_without_wake_word: false,
            seconds_remaining: 0,
            ocarina_enabled: false,
            last_heard: String::new(),
            last_matched_line: String::new(),
            last_intent: String::new(),
            transcript_lines: Vec::new(),
            motion_lights_enabled: true,
            auto_off_enabled: true,
            motion_speech_enabled: true,
            security_mode_enabled: false,
            camera_streaming: false,
            camera_mode: "rgb".to_string(),
            is_night_mode: false,
            effective_min_pixels: 450,
            depth_delta_mm: 40,
            seconds_since_last_motion: None,
            seconds_until_auto_off: None,
            tilt_angle: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_id: String,
    pub state: String,
    #[serde(default)]
    pub attributes: serde_json::Map<String, Value>,
    #[serde(default)]
    pub last_changed: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_updated: Option<DateTime<Utc>>,
}

impl Entity {
    pub fn domain(&self) -> &str {
        self.entity_id.split('.').next().unwrap_or("")
    }

    pub fn object_id(&self) -> &str {
        self.entity_id.split('.').nth(1).unwrap_or(&self.entity_id)
    }

    pub fn friendly_name(&self) -> String {
        if let Some(name) = self.attributes.get("friendly_name").and_then(|v| v.as_str()) {
            name.to_string()
        } else {
            self.object_id().replace('_', " ")
        }
    }

    pub fn is_on(&self) -> bool {
        matches!(
            self.state.to_lowercase().as_str(),
            "on" | "playing" | "open" | "active" | "home" | "unlocked" | "true"
        )
    }

    pub fn brightness_pct(&self) -> Option<u8> {
        let b = self.attributes.get("brightness")?.as_f64()?;
        Some(((b / 255.0) * 100.0).round().clamp(0.0, 100.0) as u8)
    }

    pub fn rgb_color(&self) -> Option<(u8, u8, u8)> {
        let arr = self.attributes.get("rgb_color")?.as_array()?;
        if arr.len() >= 3 {
            let r = arr[0].as_u64()? as u8;
            let g = arr[1].as_u64()? as u8;
            let b = arr[2].as_u64()? as u8;
            Some((r, g, b))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn unit_of_measurement(&self) -> Option<&str> {
        self.attributes.get("unit_of_measurement").and_then(|v| v.as_str())
    }

    pub fn media_title(&self) -> Option<&str> {
        self.attributes.get("media_title").and_then(|v| v.as_str())
    }

    pub fn media_artist(&self) -> Option<&str> {
        self.attributes.get("media_artist").and_then(|v| v.as_str())
    }

    pub fn app_name(&self) -> Option<&str> {
        self.attributes
            .get("app_name")
            .or_else(|| self.attributes.get("source"))
            .and_then(|v| v.as_str())
    }

    pub fn volume_level(&self) -> Option<u8> {
        let v = self.attributes.get("volume_level")?.as_f64()?;
        Some((v * 100.0).round().clamp(0.0, 100.0) as u8)
    }

    pub fn is_muted(&self) -> bool {
        self.attributes.get("is_volume_muted").and_then(|v| v.as_bool()).unwrap_or(false)
    }

    pub fn numeric_value(&self) -> Option<f64> {
        self.state.parse::<f64>().ok()
    }
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub timestamp: DateTime<Utc>,
    pub entity_id: String,
    pub friendly_name: String,
    pub old_state: String,
    pub new_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Reconnecting,
    #[allow(dead_code)]
    Disconnected(String),
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub is_error: bool,
    pub expires_at: DateTime<Utc>,
}
