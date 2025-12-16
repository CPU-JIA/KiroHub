import { Users, Plus } from 'lucide-react'
import { useTheme } from '../../contexts/ThemeContext'
import { useI18n } from '../../i18n.jsx'
import { useMemo, useRef } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import AccountCard from './AccountCard'

// 虚拟列表阈值：账号数超过此值时启用虚拟列表
const VIRTUAL_LIST_THRESHOLD = 50
const CARD_HEIGHT = 295  // 卡片高度 + gap
function AccountTable({
  accounts,
  filteredAccounts,
  selectedIds,
  onSelectAll,
  onSelectOne,
  copiedId,
  onCopy,
  onSwitch,
  onRefresh,
  onEdit,
  onEditLabel,
  onDelete,
  onAdd,
  refreshingId,
  switchingId,
  localToken,
}) {
  const { theme, colors } = useTheme()
  const { t } = useI18n()
  const isDark = theme === 'dark'
  const parentRef = useRef(null)

  // 是否使用虚拟列表（账号数 > 阈值）
  const useVirtualList = accounts.length > VIRTUAL_LIST_THRESHOLD

  // 虚拟滚动实例
  const rowVirtualizer = useVirtualizer({
    count: accounts.length + 1,  // +1 for add button
    getScrollElement: () => parentRef.current,
    estimateSize: () => CARD_HEIGHT,
    enabled: useVirtualList,
  })

  return (
    <div className="flex-1 overflow-auto p-6" ref={parentRef}>
      {/* 全选控制栏 */}
      {accounts.length > 0 && (
        <div className={`flex items-center gap-3 mb-4 px-1`}>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={selectedIds.length === filteredAccounts.length && filteredAccounts.length > 0}
              onChange={(e) => onSelectAll(e.target.checked)}
              className="w-4 h-4 rounded transition-transform hover:scale-110"
            />
            <span className={`text-sm ${colors.textMuted}`}>
              {selectedIds.length > 0 ? `${t('common.selected')} ${selectedIds.length}` : t('common.selectAll')}
            </span>
          </label>
        </div>
      )}

      {/* 账号列表 */}
      {accounts.length === 0 ? (
        <div className={`flex flex-col items-center justify-center py-20 ${colors.textMuted}`}>
          <div className={`w-20 h-20 rounded-full ${isDark ? 'bg-white/5' : 'bg-gray-100'} flex items-center justify-center animate-float mb-4`}>
            <Users size={40} strokeWidth={1} className="opacity-50" />
          </div>
          <p className="font-medium mb-1">{t('common.noAccounts')}</p>
          <p className="text-sm opacity-75">{t('common.addAccountHint')}</p>
        </div>
      ) : useVirtualList ? (
        /* 虚拟列表模式（50+ 账号） */
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4" style={{ height: `${rowVirtualizer.getTotalSize()}px`, position: 'relative' }}>
          {rowVirtualizer.getVirtualItems().map((virtualItem) => {
            const index = virtualItem.index
            const isAddButton = index === accounts.length
            const account = isAddButton ? null : accounts[index]

            return (
              <div
                key={virtualItem.key}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualItem.size}px`,
                  transform: `translateY(${virtualItem.start}px)`,
                }}
              >
                {isAddButton ? (
                  <button
                    onClick={onAdd}
                    className={`rounded-2xl border-2 border-dashed transition-all duration-200 min-h-[280px] flex flex-col items-center justify-center gap-3 ${
                      isDark
                        ? 'border-gray-700 hover:border-gray-500 hover:bg-white/5'
                        : 'border-gray-300 hover:border-gray-400 hover:bg-gray-50'
                    }`}
                  >
                    <div className={`w-12 h-12 rounded-full flex items-center justify-center ${
                      isDark ? 'bg-white/10' : 'bg-gray-100'
                    }`}>
                      <Plus size={24} className={colors.textMuted} />
                    </div>
                    <span className={`text-sm font-medium ${colors.textMuted}`}>{t('common.addAccount')}</span>
                  </button>
                ) : (
                  <AccountCard
                    account={account}
                    isSelected={selectedIds.includes(account.id)}
                    onSelect={(checked) => onSelectOne(account.id, checked)}
                    copiedId={copiedId}
                    onCopy={onCopy}
                    onSwitch={onSwitch}
                    onRefresh={onRefresh}
                    onEdit={onEdit}
                    onEditLabel={onEditLabel}
                    onDelete={onDelete}
                    refreshingId={refreshingId}
                    switchingId={switchingId}
                    isCurrentAccount={localToken?.refreshToken && account.refreshToken === localToken.refreshToken}
                  />
                )}
              </div>
            )
          })}
        </div>
      ) : (
        /* 普通模式（< 50 账号） */
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {accounts.map((account) => (
            <AccountCard
              key={account.id}
              account={account}
              isSelected={selectedIds.includes(account.id)}
              onSelect={(checked) => onSelectOne(account.id, checked)}
              copiedId={copiedId}
              onCopy={onCopy}
              onSwitch={onSwitch}
              onRefresh={onRefresh}
              onEdit={onEdit}
              onEditLabel={onEditLabel}
              onDelete={onDelete}
              refreshingId={refreshingId}
              switchingId={switchingId}
              isCurrentAccount={localToken?.refreshToken && account.refreshToken === localToken.refreshToken}
            />
          ))}
          {/* 添加账号卡片 */}
          <button
            onClick={onAdd}
            className={`rounded-2xl border-2 border-dashed transition-all duration-200 min-h-[280px] flex flex-col items-center justify-center gap-3 ${
              isDark 
                ? 'border-gray-700 hover:border-gray-500 hover:bg-white/5' 
                : 'border-gray-300 hover:border-gray-400 hover:bg-gray-50'
            }`}
          >
            <div className={`w-12 h-12 rounded-full flex items-center justify-center ${
              isDark ? 'bg-white/10' : 'bg-gray-100'
            }`}>
              <Plus size={24} className={colors.textMuted} />
            </div>
            <span className={`text-sm font-medium ${colors.textMuted}`}>{t('common.addAccount')}</span>
          </button>
        </div>
      )}
    </div>
  )
}

export default AccountTable
