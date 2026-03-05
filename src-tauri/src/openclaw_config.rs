//! Read/write ~/.openclaw/openclaw.json and expose agents.defaults, models.providers, subagents.
//! Uses Value for round-trip safety; presents a typed view for the UI.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const OPENCLAW_CONFIG_FILENAME: &str = "openclaw.json";

/// Path to openclaw.json (e.g. ~/.openclaw/openclaw.json).
pub fn openclaw_config_path() -> PathBuf {
    crate::get_base_dir().join(OPENCLAW_CONFIG_FILENAME)
}

/// View of the fields the UI needs: providers, primary model, models list, maxConcurrent, subagents.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenClawConfigView {
    /// Provider names from models.providers (e.g. ollama, lmstudio, nvidia-nim, anthropic).
    pub provider_names: Vec<String>,
    /// agents.defaults.model.primary
    pub primary_model: Option<String>,
    /// agents.defaults.model.fallbacks
    pub fallbacks: Vec<String>,
    /// Model ids from agents.defaults.models (keys) — "paths" to providers for dropdown.
    pub models: Vec<String>,
    /// agents.defaults.maxConcurrent
    pub max_concurrent: Option<u32>,
    /// agents.defaults.subagents
    pub subagents: SubagentsView,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubagentsView {
    pub max_concurrent: Option<u32>,
    pub max_spawn_depth: Option<u32>,
    pub max_children_per_agent: Option<u32>,
}

impl Default for SubagentsView {
    fn default() -> Self {
        Self {
            max_concurrent: Some(8),
            max_spawn_depth: Some(1),
            max_children_per_agent: Some(5),
        }
    }
}

/// Returns the raw `models.providers` object from openclaw.json for syncing to agent models.json.
pub fn get_openclaw_providers_raw() -> Result<serde_json::Value, String> {
    let path = openclaw_config_path();
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let root: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let providers = root
        .get("models")
        .and_then(|m| m.get("providers"))
        .cloned()
        .unwrap_or(serde_json::json!({}));
    Ok(providers)
}

pub fn get_raw_openclaw_config() -> serde_json::Value {
    let path = openclaw_config_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return serde_json::json!({ "agents": { "defaults": {} }, "models": { "providers": {} } }),
    };
    serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({ "agents": { "defaults": {} }, "models": { "providers": {} } }))
}

pub fn save_raw_openclaw_config(config: serde_json::Value) -> Result<(), String> {
    let path = openclaw_config_path();
    let dir = path.parent().ok_or("invalid path")?;
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

/// Reads openclaw.json and returns a view with required fields. Missing file or invalid JSON returns defaults.
#[must_use]
pub fn get_openclaw_config() -> OpenClawConfigView {
    let path = openclaw_config_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return default_view(),
    };
    parse_config_view(&content).unwrap_or_else(|_| default_view())
}

fn default_view() -> OpenClawConfigView {
    OpenClawConfigView {
        provider_names: vec![],
        primary_model: None,
        fallbacks: vec![],
        models: vec![],
        max_concurrent: None,
        subagents: SubagentsView::default(),
    }
}

fn parse_config_view(content: &str) -> Result<OpenClawConfigView, ()> {
    let root: serde_json::Value = serde_json::from_str(content).map_err(|_| ())?;
    let obj = root.as_object().ok_or(())?;

    let provider_names = obj
        .get("models")
        .and_then(|m| m.get("providers"))
        .and_then(|p| p.as_object())
        .map(|o| o.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    let (primary_model, fallbacks, models, max_concurrent, subagents) = obj
        .get("agents")
        .and_then(|a| a.get("defaults"))
        .map(|d| {
            let primary = d
                .get("model")
                .and_then(|m| m.get("primary"))
                .and_then(|v| v.as_str())
                .map(String::from);
            let fallbacks = d
                .get("model")
                .and_then(|m| m.get("fallbacks"))
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let models = d
                .get("models")
                .and_then(|m| m.as_object())
                .map(|o| o.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            let max_concurrent = d.get("maxConcurrent").and_then(|v| v.as_u64()).map(|n| n as u32);
            let subagents = d
                .get("subagents")
                .map(parse_subagents_view)
                .unwrap_or_else(SubagentsView::default);
            (primary, fallbacks, models, max_concurrent, subagents)
        })
        .unwrap_or((
            None,
            vec![],
            vec![],
            None,
            SubagentsView::default(),
        ));

    Ok(OpenClawConfigView {
        provider_names,
        primary_model,
        fallbacks,
        models,
        max_concurrent,
        subagents,
    })
}

fn parse_subagents_view(v: &serde_json::Value) -> SubagentsView {
    let empty_map = serde_json::Map::new();
    let o = v.as_object().unwrap_or(&empty_map);
    SubagentsView {
        max_concurrent: o.get("maxConcurrent").and_then(|v| v.as_u64()).map(|n| n as u32),
        max_spawn_depth: o.get("maxSpawnDepth").and_then(|v| v.as_u64()).map(|n| n as u32),
        max_children_per_agent: o
            .get("maxChildrenPerAgent")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32),
    }
}

/// Updates a subset of openclaw.json. Merges into existing file or creates with minimal structure.
pub fn update_openclaw_config(updates: OpenClawConfigUpdates) -> Result<(), String> {
    let path = openclaw_config_path();
    let mut root: serde_json::Value = if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({ "agents": { "defaults": {} }, "models": {} })
    };

    ensure_agents_defaults(&mut root);
    ensure_subagents(&mut root);

    if let Some(v) = updates.primary_model {
        set_nested(&mut root, &["agents", "defaults", "model", "primary"], serde_json::json!(v));
    }
    if let Some(v) = updates.fallbacks {
        set_nested(
            &mut root,
            &["agents", "defaults", "model", "fallbacks"],
            serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect()),
        );
    }
    if let Some(v) = updates.max_concurrent {
        set_nested(&mut root, &["agents", "defaults", "maxConcurrent"], serde_json::json!(v));
    }
    if let Some(v) = updates.subagents_max_concurrent {
        set_nested(
            &mut root,
            &["agents", "defaults", "subagents", "maxConcurrent"],
            serde_json::json!(v),
        );
    }
    if let Some(v) = updates.subagents_max_spawn_depth {
        set_nested(
            &mut root,
            &["agents", "defaults", "subagents", "maxSpawnDepth"],
            serde_json::json!(v),
        );
    }
    if let Some(v) = updates.subagents_max_children_per_agent {
        set_nested(
            &mut root,
            &["agents", "defaults", "subagents", "maxChildrenPerAgent"],
            serde_json::json!(v),
        );
    }

    let dir = path.parent().ok_or("invalid path")?;
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenClawConfigUpdates {
    pub primary_model: Option<String>,
    pub fallbacks: Option<Vec<String>>,
    pub max_concurrent: Option<u32>,
    pub subagents_max_concurrent: Option<u32>,
    pub subagents_max_spawn_depth: Option<u32>,
    pub subagents_max_children_per_agent: Option<u32>,
}

fn ensure_agents_defaults(root: &mut serde_json::Value) {
    let obj = root.as_object_mut().expect("root object");
    if !obj.contains_key("agents") {
        obj.insert(
            "agents".into(),
            serde_json::json!({ "defaults": { "model": {} } }),
        );
        return;
    }
    let agents = obj.get_mut("agents").and_then(|v| v.as_object_mut()).expect("agents");
    if !agents.contains_key("defaults") {
        agents.insert("defaults".into(), serde_json::json!({ "model": {} }));
        return;
    }
    let defaults = agents.get_mut("defaults").and_then(|v| v.as_object_mut()).expect("defaults");
    if !defaults.contains_key("model") {
        defaults.insert("model".into(), serde_json::json!({}));
    }
}

fn ensure_subagents(root: &mut serde_json::Value) {
    let defaults = root
        .get_mut("agents")
        .and_then(|v| v.as_object_mut())
        .and_then(|o| o.get_mut("defaults"))
        .and_then(|v| v.as_object_mut());
    if let Some(d) = defaults {
        if !d.contains_key("subagents") {
            d.insert(
                "subagents".into(),
                serde_json::json!({ "maxConcurrent": 8, "maxSpawnDepth": 1, "maxChildrenPerAgent": 5 }),
            );
        }
    }
}

fn set_nested(root: &mut serde_json::Value, path: &[&str], value: serde_json::Value) {
    let mut current = root;
    for (i, key) in path.iter().enumerate() {
        let is_last = i == path.len() - 1;
        if is_last {
            if let Some(obj) = current.as_object_mut() {
                obj.insert((*key).to_string(), value);
            }
            return;
        }
        let obj = current.as_object_mut().expect("object");
        if !obj.contains_key(*key) {
            obj.insert((*key).to_string(), serde_json::json!({}));
        }
        current = obj.get_mut(*key).expect("just inserted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openclaw_config_path() {
        let p = openclaw_config_path();
        assert!(p.to_string_lossy().contains("openclaw"));
        assert!(p.to_string_lossy().ends_with("openclaw.json"));
    }

    #[test]
    fn test_parse_config_view_empty() {
        let view = parse_config_view("{}").unwrap();
        assert!(view.provider_names.is_empty());
        assert!(view.primary_model.is_none());
        assert!(view.models.is_empty());
    }

    #[test]
    fn test_parse_config_view_providers_and_primary() {
        let json = r#"{
            "models": {
                "providers": {
                    "ollama": {},
                    "anthropic": {},
                    "nvidia-nim": {}
                }
            },
            "agents": {
                "defaults": {
                    "model": {
                        "primary": "anthropic/claude-sonnet-4-5",
                        "fallbacks": ["openai/gpt-5-mini"]
                    },
                    "models": {
                        "anthropic/claude-sonnet-4-5": { "alias": "sonnet" },
                        "anthropic/claude-haiku-4-5": { "alias": "haiku" }
                    },
                    "maxConcurrent": 4,
                    "subagents": {
                        "maxConcurrent": 8,
                        "maxSpawnDepth": 2,
                        "maxChildrenPerAgent": 5
                    }
                }
            }
        }"#;
        let view = parse_config_view(json).unwrap();
        assert_eq!(view.provider_names.len(), 3);
        assert!(view.provider_names.contains(&"ollama".to_string()));
        assert!(view.provider_names.contains(&"anthropic".to_string()));
        assert!(view.provider_names.contains(&"nvidia-nim".to_string()));
        assert_eq!(view.primary_model.as_deref(), Some("anthropic/claude-sonnet-4-5"));
        assert_eq!(view.fallbacks, vec!["openai/gpt-5-mini"]);
        assert_eq!(view.models.len(), 2);
        assert!(view.models.contains(&"anthropic/claude-sonnet-4-5".to_string()));
        assert_eq!(view.max_concurrent, Some(4));
        assert_eq!(view.subagents.max_concurrent, Some(8));
        assert_eq!(view.subagents.max_spawn_depth, Some(2));
        assert_eq!(view.subagents.max_children_per_agent, Some(5));
    }

    #[test]
    fn test_parse_config_view_invalid_returns_err() {
        assert!(parse_config_view("not json").is_err());
        assert!(parse_config_view("[]").is_err());
    }

    #[test]
    fn test_get_openclaw_config_no_panic() {
        let view = get_openclaw_config();
        assert!(view.provider_names.len() <= 100);
        assert!(view.models.len() <= 500);
        assert!(view.subagents.max_spawn_depth.map(|d| (1..=5).contains(&d)).unwrap_or(true));
        assert!(view
            .subagents
            .max_children_per_agent
            .map(|c| (1..=20).contains(&c))
            .unwrap_or(true));
    }
}
