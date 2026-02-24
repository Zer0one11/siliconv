use was_bindgen::prelude::*;
use crate::replay::ReplaySerializable;
use std::io::Cursor;

#[wasm_bindgen]
pub struct ReplayInfo {
    pub actions_count: usize,
    pub format_name: String,
}

#[wasm_bindgen]
pub fn get_replay_info(file_data: &[u8], extension: &str) -> Result<String, JsValue> {
    let mut cursor = Cursor::new(file_data);
    
    // Пытаемся прочитать реплей, используя твою логику из siliconv_formats
    // ВАЖНО: Мы вызовем DynamicReplay из форматов
    match siliconv_formats::DynamicReplay::read(&mut cursor, extension) {
        Ok(replay) => {
            Ok(format!(
                "Успех! Формат: {:?}, Действий: {}", 
                replay.0.format, 
                replay.0.actions.len()
            ))
        },
        Err(e) => Err(JsValue::from_str(&format!("Ошибка парсинга: {}", e))),
    }
}
