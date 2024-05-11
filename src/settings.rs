use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub settings_version: String,
    pub program_name: String,
    pub program_ver: String,
    pub program_devs: Vec<String>,
    pub program_web: String,
    pub max_embed_ratio: f32,
    pub min_capacity: u64,
    pub prog_code: String,
    pub byte_chunk: u32,
    pub secret_folder: String,
    pub thumb_folder: String,
}
