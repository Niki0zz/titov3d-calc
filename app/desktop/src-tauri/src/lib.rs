// Tauri-обвязка TitoV3D Calc.
// Две команды: чтение и запись общего JSON в каталог данных приложения.

use std::fs;
use std::path::PathBuf;
use tauri::Manager;

fn data_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("3d_calc_data_v2.json"))
}

/// Путь к файлу данных (для отображения в настройках).
#[tauri::command]
fn data_file(app: tauri::AppHandle) -> Result<String, String> {
    Ok(data_path(&app)?.to_string_lossy().to_string())
}

/// Читает JSON. Возвращает None, если файла ещё нет.
#[tauri::command]
fn read_data(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = data_path(&app)?;
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(path).map(Some).map_err(|e| e.to_string())
}

/// Атомарно пишет JSON (через временный файл).
#[tauri::command]
fn write_data(app: tauri::AppHandle, text: String) -> Result<(), String> {
    let path = data_path(&app)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![data_file, read_data, write_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
