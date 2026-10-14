use crate::client::{ClientEvent, HaClient};
use crate::model::{ConnectionStatus, Entity, KinectStatus, LogEvent, Toast, VOICE_PROFILES};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview = 0,
    Lights = 1,
    Kinect = 2,
    Switches = 3,
    Buttons = 4,
    Media = 5,
    Sensors = 6,
    Entities = 7,
    Events = 8,
}

impl Tab {
    pub const ALL: [Tab; 9] = [
        Tab::Overview,
        Tab::Lights,
        Tab::Kinect,
        Tab::Switches,
        Tab::Buttons,
        Tab::Media,
        Tab::Sensors,
        Tab::Entities,
        Tab::Events,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Overview => "1: Overview",
            Tab::Lights => "2: Lights",
            Tab::Kinect => "3: Kinect",
            Tab::Switches => "4: Switches",
            Tab::Buttons => "5: Actions",
            Tab::Media => "6: Media",
            Tab::Sensors => "7: System",
            Tab::Entities => "8: Entities",
            Tab::Events => "9: Events",
        }
    }

    pub fn next(&self) -> Self {
        let idx = *self as usize;
        Tab::ALL[(idx + 1) % Tab::ALL.len()]
    }

    pub fn prev(&self) -> Self {
        let idx = *self as usize;
        if idx == 0 {
            Tab::ALL[Tab::ALL.len() - 1]
        } else {
            Tab::ALL[idx - 1]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KinectItem {
    VoiceMute,
    MicMute,
    SelectVoice,
    SpeakText,
    TestVoice,
    ContinuousListen,
    MotorAngle,
    MotorUp,
    MotorLevel,
    MotorDown,
    CameraStream,
    CameraMode,
    MotionLights,
    MotionAutoOff,
    MotionSpeech,
    OcarinaMode,
    SecurityMode,
    ClearReminders,
}

impl KinectItem {
    pub const ALL: [KinectItem; 18] = [
        KinectItem::VoiceMute,
        KinectItem::MicMute,
        KinectItem::SelectVoice,
        KinectItem::SpeakText,
        KinectItem::TestVoice,
        KinectItem::ContinuousListen,
        KinectItem::MotorAngle,
        KinectItem::MotorUp,
        KinectItem::MotorLevel,
        KinectItem::MotorDown,
        KinectItem::CameraStream,
        KinectItem::CameraMode,
        KinectItem::MotionLights,
        KinectItem::MotionAutoOff,
        KinectItem::MotionSpeech,
        KinectItem::OcarinaMode,
        KinectItem::SecurityMode,
        KinectItem::ClearReminders,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            KinectItem::VoiceMute => "Voice Responses (TTS Mute)",
            KinectItem::MicMute => "Microphone Array (Mic Mute)",
            KinectItem::SelectVoice => "Assistant Voice Profile",
            KinectItem::SpeakText => "Speak Custom Phrase (TTS)",
            KinectItem::TestVoice => "Test Speaker Voice Output",
            KinectItem::ContinuousListen => "Continuous Listen (60s Mode)",
            KinectItem::MotorAngle => "Camera Motor Tilt Angle",
            KinectItem::MotorUp => "Tilt Camera Up (+20°)",
            KinectItem::MotorLevel => "Tilt Camera Level (0°)",
            KinectItem::MotorDown => "Tilt Camera Down (-20°)",
            KinectItem::CameraStream => "Live Camera Video Stream",
            KinectItem::CameraMode => "Camera View Mode",
            KinectItem::MotionLights => "Motion Auto-On Lights",
            KinectItem::MotionAutoOff => "Empty Room Auto-Off",
            KinectItem::MotionSpeech => "Motion Greeting Speech",
            KinectItem::OcarinaMode => "Zelda Ocarina Detection",
            KinectItem::SecurityMode => "Security Alarm Mode",
            KinectItem::ClearReminders => "Clear Voice Reminders",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            KinectItem::VoiceMute
            | KinectItem::MicMute
            | KinectItem::SelectVoice
            | KinectItem::SpeakText
            | KinectItem::TestVoice
            | KinectItem::ContinuousListen => "[VOICE]",
            KinectItem::MotorAngle
            | KinectItem::MotorUp
            | KinectItem::MotorLevel
            | KinectItem::MotorDown => "[MOTOR]",
            KinectItem::CameraStream | KinectItem::CameraMode => "[CAM]  ",
            KinectItem::MotionLights
            | KinectItem::MotionAutoOff
            | KinectItem::MotionSpeech => "[MOTN] ",
            KinectItem::OcarinaMode | KinectItem::SecurityMode | KinectItem::ClearReminders => {
                "[MODE] "
            }
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            KinectItem::VoiceMute => {
                "Mutes or unmutes the assistant's spoken TTS responses while keeping voice command recognition active."
            }
            KinectItem::MicMute => {
                "Mutes or unmutes the Xbox 360 Kinect 4-channel microphone array hardware input."
            }
            KinectItem::SelectVoice => {
                "Switch active assistant voice (Rachel, GLaDOS, HAL 9000, Cave Johnson, Daniel, Adam, George, Bella, Charlie, Karen, Espeak)."
            }
            KinectItem::SpeakText => {
                "Open a prompt to type arbitrary text for the Kinect assistant to synthesize and speak aloud."
            }
            KinectItem::TestVoice => {
                "Synthesizes and plays a test vocal greeting through the workstation speakers."
            }
            KinectItem::ContinuousListen => {
                "Enables 60-second continuous listening mode so you can speak commands without repeating wake words."
            }
            KinectItem::MotorAngle => {
                "Adjust the physical Kinect motorized base tilt angle between -30° and +30° using Left/Right or -/+ keys."
            }
            KinectItem::MotorUp => {
                "Tilts the physical Kinect motorized camera base upward to +20°."
            }
            KinectItem::MotorLevel => {
                "Centers the physical Kinect motorized camera base to level horizontal (0°)."
            }
            KinectItem::MotorDown => {
                "Tilts the physical Kinect motorized camera base downward to -20°."
            }
            KinectItem::CameraStream => {
                "Switches Kinect between 24/7 3D Infrared Depth motion sensing (0 lux) and live optical webcam streaming."
            }
            KinectItem::CameraMode => {
                "Cycles the active camera feed mode between Normal RGB Color, Thermal Depth Heatmap, and Depth Grayscale."
            }
            KinectItem::MotionLights => {
                "Automatically turns on room lights when 3D infrared depth sensing detects human movement."
            }
            KinectItem::MotionAutoOff => {
                "Automatically turns off room lights after the room remains empty for the timeout duration."
            }
            KinectItem::MotionSpeech => {
                "Speaks an assistant personality greeting when motion is triggered."
            }
            KinectItem::OcarinaMode => {
                "Listens for musical Ocarina melodies (Zelda songs) to trigger smart home macros."
            }
            KinectItem::SecurityMode => {
                "Sends a security alert notification to Beck's phone when room motion is detected."
            }
            KinectItem::ClearReminders => {
                "Clears all pending time, motion, and phone arrival voice reminders."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    Help,
    Assist {
        input: String,
        response: Option<String>,
        loading: bool,
    },
    VoicePicker {
        selected_idx: usize,
        announce: bool,
    },
    SpeakPrompt {
        input: String,
    },
    EntityDetail {
        entity_id: String,
    },
    ColorPicker {
        entity_id: String,
        selected_idx: usize,
    },
    ServiceCall {
        entity_id: String,
        domain: String,
        service: String,
        data_json: String,
        active_field: usize,
    },
}

pub struct App {
    pub client: Arc<HaClient>,
    pub status: ConnectionStatus,
    pub current_tab: Tab,
    pub entities: HashMap<String, Entity>,
    pub kinect: KinectStatus,
    pub events: Vec<LogEvent>,
    pub toasts: Vec<Toast>,
    pub active_modal: ActiveModal,

    // Selection indices for lists
    pub overview_selected: usize,
    pub lights_selected: usize,
    pub kinect_selected: usize,
    pub switches_selected: usize,
    pub buttons_selected: usize,
    pub media_selected: usize,
    pub sensors_selected: usize,
    pub entities_selected: usize,
    pub events_scroll: usize,

    // Universal list search & filter
    pub search_query: String,
    pub is_searching: bool,
    pub domain_filter: Option<String>,

    pub should_quit: bool,
    pub last_refresh: DateTime<Utc>,
}

impl App {
    pub fn new(client: Arc<HaClient>) -> Self {
        Self {
            client,
            status: ConnectionStatus::Connecting,
            current_tab: Tab::Overview,
            entities: HashMap::new(),
            kinect: KinectStatus::default(),
            events: Vec::new(),
            toasts: Vec::new(),
            active_modal: ActiveModal::None,

            overview_selected: 0,
            lights_selected: 0,
            kinect_selected: 0,
            switches_selected: 0,
            buttons_selected: 0,
            media_selected: 0,
            sensors_selected: 0,
            entities_selected: 0,
            events_scroll: 0,

            search_query: String::new(),
            is_searching: false,
            domain_filter: None,

            should_quit: false,
            last_refresh: Utc::now(),
        }
    }

    pub fn add_toast(&mut self, message: impl Into<String>, is_error: bool) {
        self.toasts.push(Toast {
            message: message.into(),
            is_error,
            expires_at: Utc::now() + Duration::seconds(4),
        });
    }

    pub fn clean_expired_toasts(&mut self) {
        let now = Utc::now();
        self.toasts.retain(|t| t.expires_at > now);
    }

    pub fn current_ha_tilt_angle(&self) -> f64 {
        self.entities
            .get("input_number.kinect_tilt_angle")
            .and_then(|e| e.numeric_value())
            .unwrap_or(self.kinect.tilt_angle)
    }

    pub fn handle_client_event(&mut self, event: ClientEvent) {
        match event {
            ClientEvent::WsConnected => {
                self.status = ConnectionStatus::Connected;
                self.add_toast("Connected to Home Assistant WebSocket", false);
            }
            ClientEvent::WsDisconnected(reason) => {
                self.status = ConnectionStatus::Reconnecting;
                self.add_toast(format!("Notice: {}", reason), true);
            }
            ClientEvent::FullStates(entities) => {
                self.set_entities(entities);
                self.status = ConnectionStatus::Connected;
                self.last_refresh = Utc::now();
                if let Some(t) = self
                    .entities
                    .get("input_number.kinect_tilt_angle")
                    .and_then(|e| e.numeric_value())
                {
                    self.kinect.tilt_angle = t;
                }
            }
            ClientEvent::KinectUpdated(mut k_status) => {
                // Preserve tilt angle from HA if available
                if let Some(t) = self
                    .entities
                    .get("input_number.kinect_tilt_angle")
                    .and_then(|e| e.numeric_value())
                {
                    k_status.tilt_angle = t;
                }
                self.kinect = k_status;
            }
            ClientEvent::StateChanged {
                entity_id,
                new_entity,
                old_state,
            } => {
                if entity_id == "light.living_room_lights" {
                    return;
                }
                let friendly_name = new_entity.friendly_name();
                let new_state = new_entity.state.clone();

                if entity_id == "input_number.kinect_tilt_angle" {
                    if let Some(val) = new_entity.numeric_value() {
                        self.kinect.tilt_angle = val;
                    }
                }

                // Add to log
                self.events.insert(
                    0,
                    LogEvent {
                        timestamp: Utc::now(),
                        entity_id: entity_id.clone(),
                        friendly_name,
                        old_state,
                        new_state,
                    },
                );
                if self.events.len() > 200 {
                    self.events.pop();
                }

                // Update store
                self.entities.insert(entity_id, new_entity);
            }
        }
    }

    pub fn set_entities(&mut self, entities: Vec<Entity>) {
        self.entities.clear();
        for entity in entities {
            if entity.entity_id != "light.living_room_lights" {
                self.entities.insert(entity.entity_id.clone(), entity);
            }
        }
    }

    fn matches_search(&self, entity: &Entity) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        let q = self.search_query.to_lowercase();
        entity.entity_id.to_lowercase().contains(&q)
            || entity.friendly_name().to_lowercase().contains(&q)
            || entity.state.to_lowercase().contains(&q)
    }

    // Filter helpers (all respect self.search_query!)
    pub fn get_lights(&self) -> Vec<&Entity> {
        let mut list: Vec<&Entity> = self
            .entities
            .values()
            .filter(|e| {
                e.domain() == "light"
                    && e.entity_id != "light.living_room_lights"
                    && self.matches_search(e)
            })
            .collect();
        list.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
        list
    }

    pub fn get_kinect_items(&self) -> Vec<KinectItem> {
        if self.search_query.is_empty() {
            return KinectItem::ALL.to_vec();
        }
        let q = self.search_query.to_lowercase();
        KinectItem::ALL
            .into_iter()
            .filter(|item| {
                item.label().to_lowercase().contains(&q)
                    || item.category().to_lowercase().contains(&q)
                    || item.description().to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn get_switches_and_automations(&self) -> Vec<&Entity> {
        let mut list: Vec<&Entity> = self
            .entities
            .values()
            .filter(|e| {
                matches!(e.domain(), "switch" | "automation" | "input_boolean")
                    && self.matches_search(e)
            })
            .collect();
        list.sort_by(|a, b| {
            a.domain()
                .cmp(b.domain())
                .then_with(|| a.entity_id.cmp(&b.entity_id))
        });
        list
    }

    pub fn get_buttons_and_scripts(&self) -> Vec<&Entity> {
        let mut list: Vec<&Entity> = self
            .entities
            .values()
            .filter(|e| matches!(e.domain(), "button" | "script") && self.matches_search(e))
            .collect();
        list.sort_by(|a, b| {
            a.domain()
                .cmp(b.domain())
                .then_with(|| a.entity_id.cmp(&b.entity_id))
        });
        list
    }

    pub fn get_media_players(&self) -> Vec<&Entity> {
        let mut list: Vec<&Entity> = self
            .entities
            .values()
            .filter(|e| e.domain() == "media_player" && self.matches_search(e))
            .collect();
        list.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
        list
    }

    pub fn get_sensors(&self) -> Vec<&Entity> {
        let mut list: Vec<&Entity> = self
            .entities
            .values()
            .filter(|e| {
                matches!(e.domain(), "sensor" | "binary_sensor" | "weather" | "sun")
                    && self.matches_search(e)
            })
            .collect();
        list.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
        list
    }

    pub fn get_filtered_entities(&self) -> Vec<&Entity> {
        let mut list: Vec<&Entity> = self
            .entities
            .values()
            .filter(|e| {
                if let Some(df) = &self.domain_filter {
                    if e.domain() != df {
                        return false;
                    }
                }
                self.matches_search(e)
            })
            .collect();
        list.sort_by(|a, b| a.entity_id.cmp(&b.entity_id));
        list
    }

    pub fn get_filtered_events(&self) -> Vec<&LogEvent> {
        if self.search_query.is_empty() {
            return self.events.iter().collect();
        }
        let q = self.search_query.to_lowercase();
        self.events
            .iter()
            .filter(|ev| {
                ev.entity_id.to_lowercase().contains(&q)
                    || ev.friendly_name.to_lowercase().contains(&q)
                    || ev.new_state.to_lowercase().contains(&q)
                    || ev.old_state.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn clamp_selections(&mut self) {
        let l_len = self.get_lights().len();
        if l_len > 0 && self.lights_selected >= l_len {
            self.lights_selected = l_len - 1;
        }
        if l_len > 0 && self.overview_selected >= l_len {
            self.overview_selected = l_len - 1;
        }
        let k_len = self.get_kinect_items().len();
        if k_len > 0 && self.kinect_selected >= k_len {
            self.kinect_selected = k_len - 1;
        }
        let s_len = self.get_switches_and_automations().len();
        if s_len > 0 && self.switches_selected >= s_len {
            self.switches_selected = s_len - 1;
        }
        let b_len = self.get_buttons_and_scripts().len();
        if b_len > 0 && self.buttons_selected >= b_len {
            self.buttons_selected = b_len - 1;
        }
        let m_len = self.get_media_players().len();
        if m_len > 0 && self.media_selected >= m_len {
            self.media_selected = m_len - 1;
        }
        let sen_len = self.get_sensors().len();
        if sen_len > 0 && self.sensors_selected >= sen_len {
            self.sensors_selected = sen_len - 1;
        }
        let e_len = self.get_filtered_entities().len();
        if e_len > 0 && self.entities_selected >= e_len {
            self.entities_selected = e_len - 1;
        }
    }

    // Navigation methods
    pub fn select_next(&mut self) {
        match self.current_tab {
            Tab::Overview => {
                let len = self.get_lights().len();
                if len > 0 {
                    self.overview_selected = (self.overview_selected + 1) % len;
                }
            }
            Tab::Lights => {
                let len = self.get_lights().len();
                if len > 0 {
                    self.lights_selected = (self.lights_selected + 1) % len;
                }
            }
            Tab::Kinect => {
                let len = self.get_kinect_items().len();
                if len > 0 {
                    self.kinect_selected = (self.kinect_selected + 1) % len;
                }
            }
            Tab::Switches => {
                let len = self.get_switches_and_automations().len();
                if len > 0 {
                    self.switches_selected = (self.switches_selected + 1) % len;
                }
            }
            Tab::Buttons => {
                let len = self.get_buttons_and_scripts().len();
                if len > 0 {
                    self.buttons_selected = (self.buttons_selected + 1) % len;
                }
            }
            Tab::Media => {
                let len = self.get_media_players().len();
                if len > 0 {
                    self.media_selected = (self.media_selected + 1) % len;
                }
            }
            Tab::Sensors => {
                let len = self.get_sensors().len();
                if len > 0 {
                    self.sensors_selected = (self.sensors_selected + 1) % len;
                }
            }
            Tab::Entities => {
                let len = self.get_filtered_entities().len();
                if len > 0 {
                    self.entities_selected = (self.entities_selected + 1) % len;
                }
            }
            Tab::Events => {
                let len = self.get_filtered_events().len();
                if self.events_scroll < len.saturating_sub(1) {
                    self.events_scroll += 1;
                }
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.current_tab {
            Tab::Overview => {
                let len = self.get_lights().len();
                if len > 0 {
                    if self.overview_selected == 0 {
                        self.overview_selected = len - 1;
                    } else {
                        self.overview_selected -= 1;
                    }
                }
            }
            Tab::Lights => {
                let len = self.get_lights().len();
                if len > 0 {
                    if self.lights_selected == 0 {
                        self.lights_selected = len - 1;
                    } else {
                        self.lights_selected -= 1;
                    }
                }
            }
            Tab::Kinect => {
                let len = self.get_kinect_items().len();
                if len > 0 {
                    if self.kinect_selected == 0 {
                        self.kinect_selected = len - 1;
                    } else {
                        self.kinect_selected -= 1;
                    }
                }
            }
            Tab::Switches => {
                let len = self.get_switches_and_automations().len();
                if len > 0 {
                    if self.switches_selected == 0 {
                        self.switches_selected = len - 1;
                    } else {
                        self.switches_selected -= 1;
                    }
                }
            }
            Tab::Buttons => {
                let len = self.get_buttons_and_scripts().len();
                if len > 0 {
                    if self.buttons_selected == 0 {
                        self.buttons_selected = len - 1;
                    } else {
                        self.buttons_selected -= 1;
                    }
                }
            }
            Tab::Media => {
                let len = self.get_media_players().len();
                if len > 0 {
                    if self.media_selected == 0 {
                        self.media_selected = len - 1;
                    } else {
                        self.media_selected -= 1;
                    }
                }
            }
            Tab::Sensors => {
                let len = self.get_sensors().len();
                if len > 0 {
                    if self.sensors_selected == 0 {
                        self.sensors_selected = len - 1;
                    } else {
                        self.sensors_selected -= 1;
                    }
                }
            }
            Tab::Entities => {
                let len = self.get_filtered_entities().len();
                if len > 0 {
                    if self.entities_selected == 0 {
                        self.entities_selected = len - 1;
                    } else {
                        self.entities_selected -= 1;
                    }
                }
            }
            Tab::Events => {
                if self.events_scroll > 0 {
                    self.events_scroll -= 1;
                }
            }
        }
    }

    pub fn get_selected_kinect_item(&self) -> Option<KinectItem> {
        let items = self.get_kinect_items();
        items.get(self.kinect_selected).copied()
    }

    pub fn get_selected_entity_id(&self) -> Option<String> {
        match self.current_tab {
            Tab::Overview => {
                let lights = self.get_lights();
                lights.get(self.overview_selected).map(|e| e.entity_id.clone())
            }
            Tab::Lights => {
                let lights = self.get_lights();
                lights.get(self.lights_selected).map(|e| e.entity_id.clone())
            }
            Tab::Kinect => None,
            Tab::Switches => {
                let list = self.get_switches_and_automations();
                list.get(self.switches_selected).map(|e| e.entity_id.clone())
            }
            Tab::Buttons => {
                let list = self.get_buttons_and_scripts();
                list.get(self.buttons_selected).map(|e| e.entity_id.clone())
            }
            Tab::Media => {
                let list = self.get_media_players();
                list.get(self.media_selected).map(|e| e.entity_id.clone())
            }
            Tab::Sensors => {
                let list = self.get_sensors();
                list.get(self.sensors_selected).map(|e| e.entity_id.clone())
            }
            Tab::Entities => {
                let list = self.get_filtered_entities();
                list.get(self.entities_selected).map(|e| e.entity_id.clone())
            }
            Tab::Events => None,
        }
    }

    pub fn active_voice_index(&self) -> usize {
        VOICE_PROFILES
            .iter()
            .position(|(k, _, _)| *k == self.kinect.active_voice)
            .unwrap_or(0)
    }
}
