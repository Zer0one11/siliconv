#[wasm_bindgen]
pub fn convert_replay(file_bytes: &[u8], input_ext: &str) -> Option<Vec<u8>> {
    let mut cursor = Cursor::new(file_bytes);
    
    // 1. Читаем входящий файл
    let replay = siliconv_formats::DynamicReplay::read(&mut cursor, input_ext).ok()?;
    
    // 2. Готовим буфер для записи (например, всегда в Slc3 для теста)
    let mut output = Vec::new();
    let mut writer = Cursor::new(&mut output);
    
    // 3. Сериализуем обратно через SilicateReplay
    let serializable = siliconv_formats::silicate::SilicateReplay::new(replay.0);
    serializable.write(&mut writer).ok()?;
    
    Some(output)
}
