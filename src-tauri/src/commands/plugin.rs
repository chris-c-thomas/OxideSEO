//! Plugin management Tauri IPC commands.

use std::sync::Arc;

use tauri::State;
use tokio::sync::Mutex;

use crate::plugin::manager::{PluginDetail, PluginInfo, PluginManager};

/// Managed state type alias for the plugin manager.
type PluginManagerState = Arc<Mutex<PluginManager>>;

// ---------------------------------------------------------------------------
// Plugin commands
// ---------------------------------------------------------------------------

/// List all discovered plugins.
#[tauri::command]
pub async fn list_plugins(pm: State<'_, PluginManagerState>) -> Result<Vec<PluginInfo>, String> {
    let guard = pm.lock().await;
    Ok(guard.list_plugins())
}

/// Enable a plugin by name.
#[tauri::command]
pub async fn enable_plugin(name: String, pm: State<'_, PluginManagerState>) -> Result<(), String> {
    let mut guard = pm.lock().await;
    guard.enable(&name).map_err(|e| format!("{e:#}"))
}

/// Disable a plugin by name.
#[tauri::command]
pub async fn disable_plugin(name: String, pm: State<'_, PluginManagerState>) -> Result<(), String> {
    let mut guard = pm.lock().await;
    guard.disable(&name).map_err(|e| format!("{e:#}"))
}

/// Get detailed info about a specific plugin.
#[tauri::command]
pub async fn get_plugin_detail(
    name: String,
    pm: State<'_, PluginManagerState>,
) -> Result<PluginDetail, String> {
    let guard = pm.lock().await;
    guard
        .get_plugin_detail(&name)
        .ok_or_else(|| format!("Plugin not found: {name}"))
}

/// Re-scan the plugins directory for new or updated plugins.
#[tauri::command]
pub async fn reload_plugins(pm: State<'_, PluginManagerState>) -> Result<Vec<PluginInfo>, String> {
    let mut guard = pm.lock().await;
    guard.discover().map_err(|e| format!("{e:#}"))?;
    Ok(guard.list_plugins())
}

/// Install a plugin from a directory chosen via file dialog.
#[tauri::command]
pub async fn install_plugin_from_file(
    pm: State<'_, PluginManagerState>,
    app: tauri::AppHandle,
) -> Result<Option<PluginInfo>, String> {
    use tauri_plugin_dialog::DialogExt;

    let picked = app
        .dialog()
        .file()
        .set_title("Select Plugin Directory")
        .blocking_pick_folder();

    let folder_path = match picked {
        Some(fp) => fp.into_path().map_err(|e| format!("{e:#}"))?,
        None => return Ok(None),
    };

    let mut guard = pm.lock().await;
    let name = guard
        .install_from_path(&folder_path)
        .map_err(|e| format!("{e:#}"))?;

    Ok(guard.list_plugins().into_iter().find(|p| p.name == name))
}

/// Uninstall a plugin by name.
#[tauri::command]
pub async fn uninstall_plugin(
    name: String,
    pm: State<'_, PluginManagerState>,
) -> Result<(), String> {
    let mut guard = pm.lock().await;
    guard.uninstall(&name).map_err(|e| format!("{e:#}"))
}
