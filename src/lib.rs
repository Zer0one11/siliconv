//! # Siliconv
//!
//! Siliconv is a multi-format converter and replay editor designed for Geometry Dash bot replays.

pub use siliconv_core::*;
use wasm_bindgen::prelude::*;
use std::io::Cursor;

#[wasm_bindgen]
pub struct ReplayResult {
    info: String,
    error: Option<String>,
}

#[wasm_bindgen]
impl ReplayResult {
    #[wasm_bindgen(getter)]
    pub fn info(&self) -> String { self.info.clone() }
    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> { self.error.clone() }
}

#[wasm_bindgen]
pub fn process_replay_data(file_bytes: &[u8], extension: &str) -> ReplayResult {
    let mut cursor = Cursor::new(file_bytes);
    match siliconv_formats::DynamicReplay::read(&mut cursor, extension) {
        Ok(replay) => {
            let info = format!(
                "Успешно! Формат: {:?}, Действий: {}", 
                replay.0.format, 
                replay.0.actions.len()
            );
            ReplayResult { info, error: None }
        },
        Err(e) => ReplayResult { 
            info: String::new(), 
            error: Some(format!("Ошибка: {}", e)) 
        },
    }
}

