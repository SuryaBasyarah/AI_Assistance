use serde::{Deserialize, Serialize};

fn default_pitch() -> i32 {
    0
}
fn default_f0() -> String {
    "rmvpe".into()
}
fn default_index_rate() -> f32 {
    0.75
}
fn default_protect() -> f32 {
    0.33
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub display_name: Option<String>,
    pub model: Option<String>,
    pub index: Option<String>,
    pub language_hint: Option<String>,
    #[serde(default = "default_pitch")]
    pub pitch: i32,
    #[serde(default = "default_f0")]
    pub f0_method: String,
    #[serde(default = "default_index_rate")]
    pub index_rate: f32,
    #[serde(default = "default_protect")]
    pub protect: f32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            display_name: None,
            model: None,
            index: None,
            language_hint: Some("ja".into()),
            pitch: 0,
            f0_method: default_f0(),
            index_rate: 0.75,
            protect: 0.33,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RvcProfile {
    pub id: String,
    pub display_name: String,
    pub model_file: Option<String>,
    pub index_file: Option<String>,
    pub valid: bool,
    pub loaded: bool,
    pub warning: Option<String>,
    pub error: Option<String>,
    pub language_hint: String,
    pub pitch: i32,
    pub f0_method: String,
    pub index_rate: f32,
    pub protect: f32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceList {
    pub voices: Vec<RvcProfile>,
    pub invalid: Vec<RvcProfile>,
    pub selected_profile: Option<String>,
    pub voices_root: String,
}
