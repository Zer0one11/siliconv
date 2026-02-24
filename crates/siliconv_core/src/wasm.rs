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
    
    // Пытаемся прочитать реплей, используя твою логику из siliconv_formats
    match siliconv_formats::DynamicReplay::read(&mut cursor, extension) {
        Ok(replay) => {
            let info = format!(
                "Успешно! Формат: {:?}, Действий: {}, Версия: {}.{}", 
                replay.0.format, 
                replay.0.actions.len(),
                replay.0.game_version.major,
                replay.0.game_version.minor
            );
            ReplayResult { info, error: None }
        },
        Err(e) => ReplayResult { 
            info: String::new(), 
            error: Some(format!("Ошибка: {}", e)) 
        },
    }
}
