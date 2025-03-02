use crate::models::AppState;
use serde::Serialize;
use web_sys::Storage;

const STORAGE_KEY: &str = "pomodoro_app_state";

pub fn get_local_storage() -> Option<Storage> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
}

pub fn save_state<T: Serialize>(state: &T) -> Result<(), String> {
    if let Some(storage) = get_local_storage() {
        let json = serde_json::to_string(state).map_err(|e| e.to_string())?;
        storage
            .set_item(STORAGE_KEY, &json)
            .map_err(|e| e.as_string().unwrap_or_else(|| "Storage error".to_string()))?;
        Ok(())
    } else {
        Err("LocalStorage not available".to_string())
    }
}

pub fn load_state() -> Option<AppState> {
    get_local_storage()
        .and_then(|storage| storage.get_item(STORAGE_KEY).ok())
        .flatten()
        .and_then(|json| serde_json::from_str(&json).ok())
}
