#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows

mod agents;
mod detection;
mod llmfit;
mod models_available;
mod openclaw_config;
mod system;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Serialize, Deserialize)]
struct Config {
    gateway: GatewayConfig,
    models: Vec<String>,
    api_keys: ApiKeys,
}

#[derive(Serialize, Deserialize)]
struct GatewayConfig {
    enabled: bool,
    port: u16,
    timeout: u32,
}

#[derive(Serialize, Deserialize)]
struct ApiKeys {
    helius: Option<String>,
    jupiter: Option<String>,
    firecrawl: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gateway: GatewayConfig {
                enabled: true,
                port: 8080,
                timeout: 30000,
            },
            models: vec![],
            api_keys: ApiKeys {
                helius: None,
                jupiter: None,
                firecrawl: None,
            },
        }
    }
}

static BASE_DIR: RwLock<Option<PathBuf>> = RwLock::new(None);

pub fn get_base_dir() -> PathBuf {
    if let Ok(guard) = BASE_DIR.read() {
        if let Some(path) = &*guard {
            return path.clone();
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openclaw")
}

fn get_config_path() -> PathBuf {
    get_base_dir().join("config.json")
}

#[tauri::command]
fn get_current_config_dir() -> String {
    get_base_dir().to_string_lossy().to_string()
}

#[tauri::command]
fn set_config_dir(path: String) -> Result<(), String> {
    if let Ok(mut guard) = BASE_DIR.write() {
        *guard = Some(PathBuf::from(path));
        Ok(())
    } else {
        Err("Failed to acquire lock".into())
    }
}

#[tauri::command]
fn get_status() -> Config {
    let config_path = get_config_path();
    
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    } else {
        Config::default()
    }
}

#[tauri::command]
fn save_config(config: Config) -> Result<(), String> {
    let config_path = get_config_path();
    
    match fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn start_gateway() -> Result<String, String> {
    use std::process::Command;
    match Command::new("openclaw").arg("gateway").arg("start").spawn() {
        Ok(_) => Ok("Gateway start initiated".to_string()),
        Err(e) => Err(format!("Failed to start gateway: {}", e)),
    }
}

#[tauri::command]
fn stop_gateway() -> Result<String, String> {
    use std::process::Command;
    match Command::new("openclaw").arg("gateway").arg("stop").spawn() {
        Ok(_) => Ok("Gateway stop initiated".to_string()),
        Err(e) => Err(format!("Failed to stop gateway: {}", e)),
    }
}

#[tauri::command]
fn add_model(model_name: String) -> Result<Vec<String>, String> {
    let config_path = get_config_path();
    
    if !config_path.exists() {
        return Err("Config file not found".to_string());
    }

    let content = fs::read_to_string(&config_path).unwrap();
    let mut config: Config = serde_json::from_str(&content).unwrap_or_default();
    
    config.models.push(model_name);
    
    match fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()) {
        Ok(_) => Ok(config.models),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn save_api_key(service: String, key: String) -> Result<(), String> {
    let config_path = get_config_path();
    
    if !config_path.exists() {
        return Err("Config file not found".to_string());
    }

    let content = fs::read_to_string(&config_path).unwrap();
    let mut config: Config = serde_json::from_str(&content).unwrap_or_default();
    
    match service.as_str() {
        "helius" => config.api_keys.helius = Some(key),
        "jupiter" => config.api_keys.jupiter = Some(key),
        "firecrawl" => config.api_keys.firecrawl = Some(key),
        _ => return Err("Unknown service".to_string()),
    }
    
    match fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// --- Local LLM detection (delegate to detection module) ---

#[tauri::command]
fn detect_local_llms() -> detection::LocalLLMDetection {
    detection::detect_local_llms()
}

#[tauri::command]
fn get_system_info() -> system::SystemInfo {
    system::get_system_info()
}

#[tauri::command]
fn get_ollama_models() -> Vec<String> {
    models_available::get_ollama_models()
}

#[tauri::command]
fn get_lm_studio_models() -> Vec<String> {
    models_available::get_lm_studio_models()
}

#[tauri::command]
fn get_llmfit_system() -> Option<llmfit::LlmfitSystemJson> {
    llmfit::get_llmfit_system()
}

#[tauri::command]
fn get_llmfit_recommendations(limit: u8) -> Vec<llmfit::LlmfitRecommendation> {
    llmfit::get_llmfit_recommendations(limit)
}

#[tauri::command]
fn get_openclaw_config() -> openclaw_config::OpenClawConfigView {
    openclaw_config::get_openclaw_config()
}

#[tauri::command]
fn update_openclaw_config(updates: openclaw_config::OpenClawConfigUpdates) -> Result<(), String> {
    openclaw_config::update_openclaw_config(updates)
}

#[tauri::command]
fn list_agents() -> Vec<String> {
    agents::list_agent_names()
}

#[tauri::command]
fn get_agent_models(agent_name: String) -> Option<agents::AgentModelsView> {
    agents::get_agent_models(&agent_name)
}

#[tauri::command]
fn get_agent_provider_sync_status(agent_name: String) -> agents::ProviderSyncStatus {
    agents::get_provider_sync_status(&agent_name)
}

#[tauri::command]
fn update_agent_providers_from_openclaw(agent_name: String) -> Result<(), String> {
    agents::update_agent_providers_from_openclaw(&agent_name)
}

#[tauri::command]
fn check_gateway_status() -> Result<bool, String> {
    use std::process::Command;
    match Command::new("openclaw")
        .arg("gateway")
        .arg("discover")
        .arg("--json")
        .arg("--timeout")
        .arg("500")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(count) = json.get("count").and_then(|c| c.as_u64()) {
                        return Ok(count > 0);
                    }
                }
            }
            Ok(false)
        }
        Err(e) => Err(format!("Failed to check gateway status: {}", e)),
    }
}

#[tauri::command]
fn get_raw_openclaw_config() -> serde_json::Value {
    openclaw_config::get_raw_openclaw_config()
}

#[tauri::command]
fn save_raw_openclaw_config(config: serde_json::Value) -> Result<(), String> {
    openclaw_config::save_raw_openclaw_config(config)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_current_config_dir,
            set_config_dir,
            get_status,
            save_config,
            start_gateway,
            stop_gateway,
            check_gateway_status,
            add_model,
            save_api_key,
            detect_local_llms,
            get_system_info,
            get_ollama_models,
            get_lm_studio_models,
            get_llmfit_system,
            get_llmfit_recommendations,
            get_openclaw_config,
            update_openclaw_config,
            list_agents,
            get_agent_models,
            get_agent_provider_sync_status,
            update_agent_providers_from_openclaw,
            get_raw_openclaw_config,
            save_raw_openclaw_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
