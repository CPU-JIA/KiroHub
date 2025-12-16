import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'

export default function AuthCallback() {
  const [status, setStatus] = useState('loading')
  const [message, setMessage] = useState('')
  const { t } = useI18n()

  useEffect(() => {
    setMessage(t('common.processing'))
    const handleCallback = async () => {
      try {
        // 从 URL 获取 code 和 state
        const url = new URL(window.location.href)
        const code = url.searchParams.get('code')
        const state = url.searchParams.get('state')

        if (!code || !state) {
          setStatus('error')
          setMessage(t('common.missingParams'))
          return
        }

        setStatus('processing')
        setMessage(t('common.processingToken'))

        // 调用 Rust 处理回调
        await invoke('handle_kiro_social_callback', { code, state })

        setStatus('success')
        setMessage(t('common.loginSuccessMsg'))

        // 3秒后关闭窗口
        setTimeout(() => {
          window.close()
        }, 3000)

      } catch (error) {
        console.error('Callback error:', error)
        setStatus('error')
        setMessage(error.message || t('common.loginFailedRetry'))
      }
    }

    handleCallback()
  }, [t])

  const getStatusIcon = () => {
    switch (status) {
      case 'loading':
      case 'processing':
        return (
          <div className="w-16 h-16 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
        )
      case 'success':
        return (
          <div className="w-16 h-16 bg-green-500 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg className="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
          </div>
        )
      case 'error':
        return (
          <div className="w-16 h-16 bg-red-500 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg className="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </div>
        )
      default:
        return null
    }
  }

  const getStatusColor = () => {
    switch (status) {
      case 'success':
        return 'text-green-600'
      case 'error':
        return 'text-red-600'
      default:
        return 'text-blue-600'
    }
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center p-4">
      <div className="bg-white rounded-2xl shadow-xl p-8 max-w-md w-full">
        {getStatusIcon()}
        
        <h1 className={`text-2xl font-bold text-center mb-4 ${getStatusColor()}`}>
          {status === 'success' && t('common.loginSuccess')}
          {status === 'error' && t('common.loginFailed')}
          {(status === 'loading' || status === 'processing') && t('common.processing')}
        </h1>
        
        <p className="text-gray-600 text-center mb-6 leading-relaxed">
          {message}
        </p>

        {status === 'success' && (
          <div className="text-center">
            <p className="text-sm text-gray-500 mb-4">
              {t('common.autoCloseHint')}
            </p>
            <button
              onClick={() => window.close()}
              className="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
            >
              {t('common.closeWindow')}
            </button>
          </div>
        )}

        {status === 'error' && (
          <div className="text-center">
            <button
              onClick={() => window.close()}
              className="px-6 py-2 bg-gray-500 text-white rounded-lg hover:bg-gray-600 transition-colors"
            >
              {t('common.closeWindow')}
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
