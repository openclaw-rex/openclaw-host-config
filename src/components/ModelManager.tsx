import { invoke } from '@tauri-apps/api/core'
import React, { useState, useEffect } from 'react'

export default function ModelManager() {
  const [config, setConfig] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)

  const [expandedProviders, setExpandedProviders] = useState<Record<string, boolean>>({})
  const [expandedModels, setExpandedModels] = useState<Record<string, boolean>>({})
  const [revealedKeys, setRevealedKeys] = useState<Record<string, boolean>>({})

  const fetchConfig = async () => {
    try {
      setLoading(true)
      const data = await invoke<any>('get_raw_openclaw_config')
      setConfig(data)
    } catch (error) {
      console.error('Failed to fetch config:', error)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchConfig()
  }, [])

  const saveConfig = async (newConfig: any) => {
    try {
      setSaving(true)
      await invoke('save_raw_openclaw_config', { config: newConfig })
      setConfig(newConfig)
    } catch (error) {
      console.error('Failed to save config:', error)
      alert(`Failed to save config: ${error}`)
    } finally {
      setSaving(false)
    }
  }

  if (loading || !config) {
    return (
      <div className="status-card">
        <h2>Model Configuration</h2>
        <p>Loading openclaw config...</p>
      </div>
    )
  }

  // Ensure necessary paths exist
  const cfg = { ...config }
  if (!cfg.agents) cfg.agents = {}
  if (!cfg.agents.defaults) cfg.agents.defaults = {}
  if (!cfg.agents.defaults.model) cfg.agents.defaults.model = {}
  if (!cfg.models) cfg.models = {}
  if (!cfg.models.providers) cfg.models.providers = {}

  const providers = cfg.models.providers
  const primaryModel = cfg.agents.defaults.model.primary || ''

  // Gather all available models
  const availableModels: string[] = []
  Object.keys(providers).forEach((providerName) => {
    const p = providers[providerName]
    if (p && Array.isArray(p.models)) {
      p.models.forEach((m: any) => {
        if (m.id) {
          // Typically "provider/id"
          // Unless id already starts with provider
          let modelPath = `${providerName}/${m.id}`
          if (m.id.startsWith(providerName + '/')) modelPath = m.id
          availableModels.push(modelPath)
        }
      })
    }
  })

  // Dropdown for primary model
  const handlePrimaryModelChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    cfg.agents.defaults.model.primary = e.target.value
    saveConfig(cfg)
  }

  const handleProviderChange = (oldName: string, field: string, value: string) => {
    providers[oldName][field] = value;
    saveConfig(cfg)
  }

  const handleProviderRename = (oldName: string, newName: string) => {
    if (!newName || newName === oldName || providers[newName]) return;
    providers[newName] = providers[oldName]
    delete providers[oldName]
    saveConfig(cfg)
  }

  const addProvider = () => {
    let name = "new_provider"
    let idx = 1
    while (providers[`${name}_${idx}`]) idx++
    const newName = `${name}_${idx}`

    providers[newName] = {
      api: "",
      apiKey: "",
      baseUrl: "",
      models: []
    }
    setExpandedProviders({ ...expandedProviders, [newName]: true })
    saveConfig(cfg)
  }

  const removeProvider = (name: string) => {
    if (confirm(`Are you sure you want to delete ${name}?`)) {
      delete providers[name]
      saveConfig(cfg)
    }
  }

  const toggleProvider = (name: string) => {
    setExpandedProviders(prev => ({ ...prev, [name]: !prev[name] }))
  }

  const toggleModel = (provider: string, mIndex: number) => {
    const key = `${provider}-${mIndex}`
    setExpandedModels(prev => ({ ...prev, [key]: !prev[key] }))
  }

  const toggleKey = (provider: string) => {
    setRevealedKeys(prev => ({ ...prev, [provider]: !prev[provider] }))
  }

  const addModel = (provider: string) => {
    if (!providers[provider].models) providers[provider].models = []
    providers[provider].models.push({
      id: "new-model",
      name: "New Model",
      contextWindow: 4096,
      input: ["text"]
    })
    const mIndex = providers[provider].models.length - 1
    setExpandedModels({ ...expandedModels, [`${provider}-${mIndex}`]: true })
    saveConfig(cfg)
  }

  const removeModel = (provider: string, mIndex: number) => {
    if (confirm(`Remove this model?`)) {
      providers[provider].models.splice(mIndex, 1)
      saveConfig(cfg)
    }
  }

  const updateModelField = (provider: string, mIndex: number, field: string, val: any) => {
    providers[provider].models[mIndex][field] = val
    saveConfig(cfg)
  }

  return (
    <div className="status-card" style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2>Model Configuration</h2>
        <button className="btn" onClick={fetchConfig} disabled={loading}>{loading ? 'Refreshing...' : 'Refresh JSON'}</button>
      </div>

      <div style={{ background: 'rgba(0,0,0,0.1)', padding: '1rem', borderRadius: '8px', border: '1px solid #334155' }}>
        <label style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 'bold' }}>Primary Model (agents.defaults.model.primary)</label>
        <select
          value={primaryModel}
          onChange={handlePrimaryModelChange}
          style={{ width: '100%', padding: '0.5rem', borderRadius: '4px', background: '#1e293b', color: '#f8fafc', border: '1px solid #475569' }}
        >
          <option value="">-- Select Primary Model --</option>
          {availableModels.map(m => (
            <option key={m} value={m}>{m}</option>
          ))}
          {primaryModel && !availableModels.includes(primaryModel) && (
            <option value={primaryModel}>{primaryModel} (Not in models list)</option>
          )}
        </select>
        {saving && <span style={{ marginLeft: '1rem', fontSize: '0.8rem', color: '#cbd5e1' }}>Saving...</span>}
      </div>

      <div>
        <h3>Providers</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', marginTop: '1rem' }}>
          {Object.keys(providers).map((providerName) => {
            const provider = providers[providerName]
            const isExpanded = expandedProviders[providerName]

            return (
              <div key={providerName} style={{ border: '1px solid #334155', borderRadius: '8px', overflow: 'hidden' }}>
                <div style={{ display: 'flex', alignItems: 'center', background: '#1e293b', padding: '0.75rem 1rem', cursor: 'pointer' }} onClick={() => toggleProvider(providerName)}>
                  <span style={{ marginRight: '1rem', display: 'flex', alignItems: 'center', justifyContent: 'center', width: '28px', height: '28px', background: '#334155', borderRadius: '6px', fontSize: '0.9rem', color: '#f8fafc', flexShrink: 0 }}>{isExpanded ? '▼' : '▶'}</span>
                  <input
                    type="text"
                    value={providerName}
                    onClick={e => e.stopPropagation()}
                    onChange={e => handleProviderRename(providerName, e.target.value)}
                    style={{ flex: 1, background: 'transparent', border: 'none', color: '#f8fafc', fontSize: '1.1rem', fontWeight: 'bold', outline: 'none' }}
                  />
                  <button onClick={(e) => { e.stopPropagation(); removeProvider(providerName) }} style={{ background: 'transparent', border: 'none', color: '#ef4444', cursor: 'pointer', fontSize: '1.2rem' }} title="Remove Provider">
                    🗑️
                  </button>
                </div>

                {isExpanded && (
                  <div style={{ padding: '1rem', background: 'rgba(0,0,0,0.2)' }}>
                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '0.5rem', marginBottom: '1rem' }}>
                      <label>API</label>
                      <input type="text" value={provider.api || ''} onChange={e => handleProviderChange(providerName, 'api', e.target.value)} style={{ padding: '0.3rem', borderRadius: '4px', background: '#0f172a', border: '1px solid #334155', color: 'white' }} />

                      <label>API Key</label>
                      <div style={{ display: 'flex', gap: '0.5rem' }}>
                        <input type={revealedKeys[providerName] ? "text" : "password"} value={provider.apiKey || ''} onChange={e => handleProviderChange(providerName, 'apiKey', e.target.value)} style={{ flex: 1, padding: '0.3rem', borderRadius: '4px', background: '#0f172a', border: '1px solid #334155', color: 'white' }} />
                        <button onClick={() => toggleKey(providerName)} style={{ background: '#334155', border: 'none', color: '#f8fafc', borderRadius: '4px', padding: '0 0.5rem', cursor: 'pointer' }}>
                          {revealedKeys[providerName] ? "Hide" : "Show"}
                        </button>
                      </div>

                      <label>Base URL</label>
                      <input type="text" value={provider.baseUrl || ''} onChange={e => handleProviderChange(providerName, 'baseUrl', e.target.value)} style={{ padding: '0.3rem', borderRadius: '4px', background: '#0f172a', border: '1px solid #334155', color: 'white' }} />
                    </div>

                    <div style={{ marginTop: '1.5rem' }}>
                      <h4 style={{ marginBottom: '0.5rem' }}>Models ({provider.models?.length || 0})</h4>
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        {(provider.models || []).map((model: any, mIndex: number) => {
                          const mKey = `${providerName}-${mIndex}`
                          const mExpanded = expandedModels[mKey]

                          return (
                            <div key={mIndex} style={{ border: '1px dotted #475569', borderRadius: '4px', padding: '0.5rem' }}>
                              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                <div style={{ display: 'flex', alignItems: 'center', flex: 1, cursor: 'pointer' }} onClick={() => toggleModel(providerName, mIndex)}>
                                  <span style={{ marginRight: '0.75rem', display: 'flex', alignItems: 'center', justifyContent: 'center', width: '24px', height: '24px', background: '#334155', borderRadius: '4px', fontSize: '0.8rem', color: '#f8fafc', flexShrink: 0 }}>{mExpanded ? '▼' : '▶'}</span>
                                  <strong>{model.name || model.id || 'Unnamed Model'}</strong>
                                </div>
                                <button onClick={() => removeModel(providerName, mIndex)} style={{ background: 'transparent', border: 'none', color: '#ef4444', cursor: 'pointer' }}>🗑️</button>
                              </div>

                              {mExpanded && (
                                <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '0.5rem', marginTop: '0.5rem', paddingLeft: '1.5rem', alignItems: 'center' }}>
                                  <label>ID</label>
                                  <input type="text" value={model.id || ''} onChange={e => updateModelField(providerName, mIndex, 'id', e.target.value)} style={{ padding: '0.3rem', background: '#0f172a', color: 'white', border: '1px solid #334155', borderRadius: '4px' }} />

                                  <label>Name</label>
                                  <input type="text" value={model.name || ''} onChange={e => updateModelField(providerName, mIndex, 'name', e.target.value)} style={{ padding: '0.3rem', background: '#0f172a', color: 'white', border: '1px solid #334155', borderRadius: '4px' }} />

                                  <label>Context Window</label>
                                  <input type="number" value={model.contextWindow || 0} onChange={e => updateModelField(providerName, mIndex, 'contextWindow', parseInt(e.target.value, 10) || 0)} style={{ padding: '0.3rem', background: '#0f172a', color: 'white', border: '1px solid #334155', borderRadius: '4px' }} />

                                  <label>Max Tokens</label>
                                  <input type="number" value={model.maxTokens || 0} onChange={e => updateModelField(providerName, mIndex, 'maxTokens', parseInt(e.target.value, 10) || 0)} style={{ padding: '0.3rem', background: '#0f172a', color: 'white', border: '1px solid #334155', borderRadius: '4px' }} placeholder="e.g. 8192" />

                                  <label>Reasoning Supported</label>
                                  <input type="checkbox" checked={!!model.reasoning} onChange={e => updateModelField(providerName, mIndex, 'reasoning', e.target.checked)} style={{ justifySelf: 'flex-start', transform: 'scale(1.2)' }} />

                                  <label>Input Types</label>
                                  <input type="text" value={(model.input || []).join(', ')} onChange={e => updateModelField(providerName, mIndex, 'input', e.target.value.split(',').map((s: string) => s.trim()).filter(Boolean))} style={{ padding: '0.3rem', background: '#0f172a', color: 'white', border: '1px solid #334155', borderRadius: '4px' }} placeholder="text, image" />
                                </div>
                              )}
                            </div>
                          )
                        })}
                        <button className="btn" style={{ alignSelf: 'flex-start', fontSize: '0.8rem', marginTop: '0.5rem' }} onClick={() => addModel(providerName)}>
                          + Add Model
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )
          })}
        </div>
        <button className="btn" style={{ marginTop: '1rem', width: '100%', background: '#3b82f6' }} onClick={addProvider}>
          + Add New Provider
        </button>
      </div>

    </div>
  )
}