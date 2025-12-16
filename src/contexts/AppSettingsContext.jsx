import { createContext, useContext, useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const AppSettingsContext = createContext(null)

export function AppSettingsProvider({ children }) {
  const [settings, setSettings] = useState(null)
  const [loading, setLoading] = useState(true)

  // 加载设置
  const loadSettings = async () => {
    try {
      const appSettings = await invoke('get_app_settings')
      setSettings(appSettings)
    } catch (err) {
      console.error('Failed to load app settings:', err)
      // 使用默认设置
      setSettings({
        lockModel: true,
        autoRefresh: true,
        autoRefreshInterval: 50,
        autoChangeMachineId: false,
        bindMachineIdToAccount: false,
        browserPath: ''
      })
    } finally {
      setLoading(false)
    }
  }

  // 更新设置（同时更新缓存和后端）
  const updateSettings = async (updates) => {
    try {
      await invoke('save_app_settings', { updates })
      setSettings(prev => ({ ...prev, ...updates }))
      return true
    } catch (err) {
      console.error('Failed to update settings:', err)
      return false
    }
  }

  useEffect(() => {
    loadSettings()

    // 监听设置变更事件（如果其他窗口修改了设置）
    const unlisten = listen('app-settings-changed', (event) => {
      setSettings(event.payload)
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [])

  const value = {
    settings,
    loading,
    updateSettings,
    reload: loadSettings
  }

  return (
    <AppSettingsContext.Provider value={value}>
      {children}
    </AppSettingsContext.Provider>
  )
}

export function useAppSettings() {
  const context = useContext(AppSettingsContext)
  if (context === null) {
    throw new Error('useAppSettings must be used within AppSettingsProvider')
  }
  return context
}
