use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn hash_key(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn read_cache<T: DeserializeOwned>(app_data_dir: &Path, subdir: &str, key: &str) -> Option<T> {
    let path = app_data_dir.join(subdir).join(format!("{}.json", key));
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn write_cache<T: Serialize>(app_data_dir: &Path, subdir: &str, key: &str, value: &T) {
    let dir = app_data_dir.join(subdir);
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("[cache] failed to create dir {:?}: {}", dir, e);
        return;
    }
    let path = dir.join(format!("{}.json", key));
    match serde_json::to_string(value) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                eprintln!("[cache] failed to write {:?}: {}", path, e);
            }
        }
        Err(e) => {
            eprintln!("[cache] failed to serialize for key {}: {}", key, e);
        }
    }
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[tauri::command]
pub async fn get_cache_size(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let cache_dir = app_data_dir.join("cache");
    if !cache_dir.exists() {
        return Ok("0 B".to_string());
    }
    Ok(format_bytes(dir_size(&cache_dir)))
}

#[tauri::command]
pub async fn clear_cache(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let cache_dir = app_data_dir.join("cache");
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).map_err(|e| format!("Failed to clear cache: {}", e))?;
    }
    Ok("Cache cleared.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_key_is_deterministic() {
        let a = hash_key("hello world");
        let b = hash_key("hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_key_different_inputs_differ() {
        let a = hash_key("input_a");
        let b = hash_key("input_b");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_key_is_16_hex_chars() {
        let h = hash_key("test");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn format_bytes_small() {
        assert_eq!(format_bytes(512), "512 B");
    }

    #[test]
    fn format_bytes_kilobytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
    }

    #[test]
    fn format_bytes_megabytes() {
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn read_write_cache_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let key = "test_key";
        let value = serde_json::json!({"foo": "bar", "num": 42});
        write_cache(tmp.path(), "sub", key, &value);
        let read: Option<serde_json::Value> = read_cache(tmp.path(), "sub", key);
        assert_eq!(read.unwrap(), value);
    }

    #[test]
    fn read_cache_miss_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let read: Option<serde_json::Value> = read_cache(tmp.path(), "sub", "nonexistent");
        assert!(read.is_none());
    }
}
