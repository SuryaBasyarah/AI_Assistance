use crate::{database::Database, error::AppResult};
use reqwest::Client;
use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
    pub http: Client,
    pub voices_root: PathBuf,
    pub sbv2_base_url: String,
    pub rvc_base_url: String,
}

impl AppState {
    pub fn new(data_dir: PathBuf, project_dir: PathBuf) -> AppResult<Self> {
        let db = Database::open(&data_dir.join("haruya.sqlite3"))?;
        let voices_root = std::env::var("HARUYA_VOICES_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| project_dir.join("voices"));
        std::fs::create_dir_all(&voices_root)?;
        Ok(Self {
            db,
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(90))
                .build()?,
            voices_root,
            sbv2_base_url: std::env::var("HARUYA_SBV2_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:5000".into()),
            rvc_base_url: std::env::var("HARUYA_RVC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:5200".into()),
        })
    }
}
