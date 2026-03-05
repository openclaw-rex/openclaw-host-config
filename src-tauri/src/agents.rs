//! List ~/.openclaw/agents (main, dev, ...), read agent/agent/models.json, sync with openclaw.json providers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::openclaw_config;

const AGENTS_DIR_NAME: &str = "agents";
const AGENT_SUBDIR: &str = "agent";
const MODELS_JSON: &str = "models.json";

fn openclaw_root() -> PathBuf {
    crate::get_base_dir()
}

/// Path to ~/.openclaw/agents.
#[must_use]
pub fn agents_dir() -> PathBuf {
    openclaw_root().join(AGENTS_DIR_NAME)
}

/// Path to an agent's models.json: ~/.openclaw/agents/<name>/agent/models.json.
#[must_use]
pub fn agent_models_path(agent_name: &str) -> PathBuf {
    agents_dir().join(agent_name).join(AGENT_SUBDIR).join(MODELS_JSON)
}

/// List agent names (subdirs of ~/.openclaw/agents that contain agent/models.json).
#[must_use]
pub fn list_agent_names() -> Vec<String> {
    let dir = match fs::read_dir(agents_dir()) {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let mut names: Vec<String> = dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| agent_models_path(e.file_name().to_string_lossy().as_ref()).exists())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

/// One provider entry in an agent's models.json (baseUrl, apiKey, api, models).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentProviderView {
    pub base_url: Option<String>,
    pub api_key_set: bool,
    pub api: Option<String>,
    pub models_count: usize,
}

/// Full view of an agent's models.json for the UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentModelsView {
    pub agent_name: String,
    pub providers: HashMap<String, AgentProviderView>,
    pub provider_names: Vec<String>,
}

/// Read agent's models.json and return a view. Returns None if file missing or invalid.
#[must_use]
pub fn get_agent_models(agent_name: &str) -> Option<AgentModelsView> {
    let path = agent_models_path(agent_name);
    let content = fs::read_to_string(&path).ok()?;
    let root: serde_json::Value = serde_json::from_str(&content).ok()?;
    let prov_obj = root.get("providers").and_then(|v| v.as_object())?;
    let mut providers = HashMap::new();
    let mut provider_names = Vec::new();
    for (name, val) in prov_obj {
        let obj = val.as_object()?;
        let base_url = obj.get("baseUrl").and_then(|v| v.as_str()).map(String::from);
        let api_key_set = obj
            .get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .or_else(|| obj.get("apiKey").and_then(|v| v.as_bool()))
            .unwrap_or(false);
        let api = obj.get("api").and_then(|v| v.as_str()).map(String::from);
        let models_count = obj
            .get("models")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        provider_names.push(name.clone());
        providers.insert(
            name.clone(),
            AgentProviderView {
                base_url,
                api_key_set,
                api,
                models_count,
            },
        );
    }
    provider_names.sort();
    Some(AgentModelsView {
        agent_name: agent_name.to_string(),
        providers,
        provider_names,
    })
}

/// Sync status: agent's models.json providers vs openclaw.json models.providers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderSyncStatus {
    pub in_sync: bool,
    pub openclaw_provider_names: Vec<String>,
    pub agent_provider_names: Vec<String>,
    pub missing_in_agent: Vec<String>,
    pub extra_in_agent: Vec<String>,
}

/// Compare openclaw.json models.providers with an agent's models.json providers.
#[must_use]
pub fn get_provider_sync_status(agent_name: &str) -> ProviderSyncStatus {
    let openclaw_names: Vec<String> = openclaw_config::get_openclaw_config()
        .provider_names
        .into_iter()
        .collect();
    let agent = get_agent_models(agent_name);
    let agent_names = agent
        .map(|a| a.provider_names)
        .unwrap_or_default();
    let openclaw_set: std::collections::HashSet<_> = openclaw_names.iter().cloned().collect();
    let agent_set: std::collections::HashSet<_> = agent_names.iter().cloned().collect();
    let missing_in_agent: Vec<String> = openclaw_set.difference(&agent_set).cloned().collect();
    let extra_in_agent: Vec<String> = agent_set.difference(&openclaw_set).cloned().collect();
    let in_sync = missing_in_agent.is_empty() && extra_in_agent.is_empty();
    ProviderSyncStatus {
        in_sync,
        openclaw_provider_names: openclaw_names,
        agent_provider_names: agent_names,
        missing_in_agent,
        extra_in_agent,
    }
}

/// Overwrite an agent's models.json providers with openclaw.json's models.providers.
/// Preserves existing provider keys (e.g. apiKey) when the provider exists in both; otherwise uses openclaw's value.
pub fn update_agent_providers_from_openclaw(agent_name: &str) -> Result<(), String> {
    let openclaw_providers = openclaw_config::get_openclaw_providers_raw()?;
    let openclaw_obj = openclaw_providers.as_object().ok_or("openclaw providers not an object")?;

    let path = agent_models_path(agent_name);
    let mut root: serde_json::Value = if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({ "providers": {} })
    };

    let root_obj = root.as_object_mut().ok_or("models.json root not an object")?;
    let agents_providers = root_obj
        .entry("providers")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("models.json providers not an object")?;

    for (name, openclaw_val) in openclaw_obj {
        let mut merged = openclaw_val.clone();
        if let (Some(merged_obj), Some(existing)) = (
            merged.as_object_mut(),
            agents_providers.get(name).and_then(|v| v.as_object()),
        ) {
            if let Some(api_key) = existing.get("apiKey") {
                merged_obj.insert("apiKey".to_string(), api_key.clone());
            }
        }
        agents_providers.insert(name.clone(), merged);
    }

    // Remove providers that exist in the agent but not in openclaw.json so sync status becomes in_sync.
    agents_providers.retain(|k, _| openclaw_obj.contains_key(k));

    let parent = path.parent().ok_or("invalid path")?;
    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agents_dir_path() {
        let p = agents_dir();
        assert!(p.to_string_lossy().contains("openclaw"));
        assert!(p.to_string_lossy().ends_with("agents"));
    }

    #[test]
    fn test_agent_models_path() {
        let p = agent_models_path("main");
        assert!(p.to_string_lossy().contains("main"));
        assert!(p.to_string_lossy().ends_with("models.json"));
    }

    #[test]
    fn test_list_agent_names_no_panic() {
        let _ = list_agent_names();
    }
}
