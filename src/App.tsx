import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import GatewayStatus from './components/GatewayStatus'
import ModelManager from './components/ModelManager'
import ApiKeyManager from './components/ApiKeyManager'
import LocalLLMs from './components/LocalLLMs'
import OpenClawConfig from './components/OpenClawConfig'
import Agents from './components/Agents'

function App() {
  const [activeTab, setActiveTab] = useState<'gateway' | 'models' | 'api-keys' | 'local-llms' | 'openclaw' | 'agents'>('gateway')
  const [configPath, setConfigPath] = useState<string>('')

  useEffect(() => {
    invoke<string>('get_current_config_dir').then(setConfigPath).catch(console.error)
  }, [])

  const handleOpenDir = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: configPath || undefined,
      });
      if (selected && typeof selected === 'string') {
        await invoke('set_config_dir', { path: selected });
        setConfigPath(selected);
      }
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <div className="app">
      <header className="header">
        <h1>🦕 OpenClaw Config</h1>
        <p>Local Host Configuration Tool</p>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginTop: '1rem', background: 'rgba(0,0,0,0.2)', padding: '0.5rem', borderRadius: '8px' }}>
          <span style={{ fontSize: '0.9rem', color: '#94a3b8' }}>Config Dir:</span>
          <input 
            type="text" 
            readOnly 
            value={configPath} 
            title={configPath}
            style={{ flex: 1, padding: '0.4rem', borderRadius: '4px', border: '1px solid #334155', background: '#1e293b', color: '#f8fafc', fontSize: '0.9rem', outline: 'none' }} 
          />
          <button className="btn" onClick={handleOpenDir}>Open</button>
        </div>
      </header>

      <nav className="tabs">
        <button
          className={activeTab === 'gateway' ? 'active' : ''}
          onClick={() => setActiveTab('gateway')}
        >
          Gateway
        </button>
        <button
          className={activeTab === 'models' ? 'active' : ''}
          onClick={() => setActiveTab('models')}
        >
          Models
        </button>
        <button
          className={activeTab === 'api-keys' ? 'active' : ''}
          onClick={() => setActiveTab('api-keys')}
        >
          API Keys
        </button>
        <button 
          className={activeTab === 'local-llms' ? 'active' : ''}
          onClick={() => setActiveTab('local-llms')}
        >
          Local LLMs
        </button>
        <button 
          className={activeTab === 'openclaw' ? 'active' : ''}
          onClick={() => setActiveTab('openclaw')}
        >
          OpenClaw
        </button>
        <button 
          className={activeTab === 'agents' ? 'active' : ''}
          onClick={() => setActiveTab('agents')}
        >
          Agents
        </button>
      </nav>

      <main className="content" key={configPath}>
        {activeTab === 'gateway' && <GatewayStatus />}
        {activeTab === 'models' && <ModelManager />}
        {activeTab === 'api-keys' && <ApiKeyManager />}
        {activeTab === 'local-llms' && <LocalLLMs />}
        {activeTab === 'openclaw' && <OpenClawConfig />}
        {activeTab === 'agents' && <Agents />}
      </main>

      <footer className="footer">
        <p>Built with Tauri + React</p>
      </footer>
    </div>
  )
}

export default App