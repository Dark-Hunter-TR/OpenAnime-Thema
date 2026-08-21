use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct Lock(AtomicBool);

impl Lock {
    pub fn set(&self, value: bool) {
        self.0.store(value, Ordering::Relaxed);
    }

    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[tauri::command]
pub fn easter_egg_open(app: AppHandle, state: tauri::State<'_, Lock>) -> Result<String, String> {
    let path = app
        .path()
        .resolve("easter-egg.b64", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("kaynak çözümlenemedi: {e}"))?;

    let data =
        std::fs::read_to_string(&path).map_err(|e| format!("{} okunamadı: {e}", path.display()))?;

    state.set(true);
    Ok(data)
}

#[tauri::command]
pub fn easter_egg_close(state: tauri::State<'_, Lock>) {
    state.set(false);
}
