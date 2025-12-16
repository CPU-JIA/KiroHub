import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import Sidebar from './components/Sidebar'
import Home from './components/Home'
import AccountManager from './components/AccountManager/index'
import Settings from './components/Settings'
import KiroConfig from './components/KiroConfig/index'
import About from './components/About'
import Login from './components/Login'
import WebOAuthLogin from './components/WebOAuthLogin'
import AuthCallback from './components/AuthCallback'
import UpdateChecker from './components/UpdateChecker'

import { useTheme } from './contexts/ThemeContext'
import { useAppSettings } from './contexts/AppSettingsContext'
import { isAccountBanned, DEFAULTS } from './utils/constants'

function App() {
  const [user, setUser] = useState(null)
  const [loading, setLoading] = useState(true)
  const [activeMenu, setActiveMenu] = useState('home')
  const { colors } = useTheme()
  const { settings: appSettings } = useAppSettings()
  const refreshTimerRef = useRef(null)

  // 通用的 token 刷新逻辑（合并了启动和定时刷新）
  const refreshExpiredTokens = async (logPrefix = 'AutoRefresh') => {
    try {
      // 使用缓存的设置
      if (!appSettings?.autoRefresh) return

      const accounts = await invoke('get_accounts')
      if (!accounts || accounts.length === 0) return

      const now = new Date()
      const refreshThreshold = DEFAULTS.REFRESH_THRESHOLD_MS // 提前 5 分钟

      const expiredAccounts = accounts.filter(acc => {
        // 跳过已封禁账号
        if (isAccountBanned(acc.status)) return false
        if (!acc.expiresAt) return false
        const expiresAt = new Date(acc.expiresAt.replace(/\//g, '-'))
        return (expiresAt.getTime() - now.getTime()) < refreshThreshold
      })

      if (expiredAccounts.length === 0) {
        console.log(`[${logPrefix}] 没有需要刷新的 token`)
        return
      }

      console.log(`[${logPrefix}] 刷新 ${expiredAccounts.length} 个过期 token...`)

      // 并发刷新
      await Promise.allSettled(
        expiredAccounts.map(async (account) => {
          try {
            await invoke('refresh_account_token', { id: account.id })
            console.log(`[${logPrefix}] ${account.email} token 刷新成功`)
          } catch (e) {
            console.warn(`[${logPrefix}] ${account.email} token 刷新失败:`, e)
          }
        })
      )

      console.log(`[${logPrefix}] token 刷新完成`)
    } catch (e) {
      console.error(`[${logPrefix}] 刷新失败:`, e)
    }
  }

  // 启动自动刷新定时器
  const startAutoRefreshTimer = async () => {
    if (refreshTimerRef.current) {
      clearInterval(refreshTimerRef.current)
    }

    // 等待设置加载完成
    if (!appSettings) return

    // 启动时只刷新 token（快速启动）
    refreshExpiredTokens('Startup')

    // 使用缓存的刷新间隔
    const intervalMs = (appSettings.autoRefreshInterval || 50) * 60 * 1000

    console.log(`[AutoRefresh] 定时器间隔: ${appSettings.autoRefreshInterval || 50} 分钟`)
    refreshTimerRef.current = setInterval(() => refreshExpiredTokens(), intervalMs)
  }

  // 当设置变化时重启定时器
  useEffect(() => {
    if (appSettings) {
      startAutoRefreshTimer()
    }
    return () => {
      if (refreshTimerRef.current) {
        clearInterval(refreshTimerRef.current)
      }
    }
  }, [appSettings])

  useEffect(() => {
    checkAuth()

    // 检查是否是回调页面
    const url = new URL(window.location.href)
    if (url.pathname === '/callback' && (url.searchParams.has('code') || url.searchParams.has('state'))) {
      setActiveMenu('callback')
      return
    }

    // 监听登录成功事件
    const unlisten = listen('login-success', (event) => {
      console.log('Login success in App:', event.payload)
      checkAuth()
      setActiveMenu('token')
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [])

  const checkAuth = async () => {
    try {
      const currentUser = await invoke('get_current_user')
      setUser(currentUser)
    } catch (e) {
      console.error('Auth check failed:', e)
    } finally {
      setLoading(false)
    }
  }

  const handleLogin = (loggedInUser) => {
    if (loggedInUser) {
      setUser(loggedInUser)
    }
    checkAuth()
  }

  const handleLogout = async () => {
    await invoke('logout')
    setUser(null)
  }

  const renderContent = () => {
    switch (activeMenu) {
      case 'home': return <Home onNavigate={setActiveMenu} />
      case 'token': return <AccountManager />
      case 'kiro-config': return <KiroConfig />
      case 'login': return <Login onLogin={(user) => { handleLogin(user); setActiveMenu('token'); }} />
      case 'web-oauth': return <WebOAuthLogin onLogin={(user) => { handleLogin(user); setActiveMenu('token'); }} />
      case 'callback': return <AuthCallback />
      case 'settings': return <Settings />
      case 'about': return <About />
      default: return <Home />
    }
  }

  if (loading) {
    return (
      <div className="h-screen bg-[#0d0d0d] flex items-center justify-center">
        <div className="text-white">加载中...</div>
      </div>
    )
  }

  return (
    <div className={`flex h-screen ${colors.main}`}>
      <Sidebar 
        activeMenu={activeMenu} 
        onMenuChange={setActiveMenu}
        user={user}
        onLogout={handleLogout}
      />
      <main className="flex-1 overflow-hidden">
        {renderContent()}
      </main>
      
      <UpdateChecker />
    </div>
  )
}

export default App
