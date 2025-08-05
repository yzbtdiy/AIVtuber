import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { BilibiliConfig } from '../types/bilibili.types'

export function useBilibiliConnection() {
  // 连接状态
  const isConnected = ref(false)
  const connectionStatus = ref('未连接')
  const isProxyRunning = ref(false)
  const proxyStatus = ref('未运行')

  // bilibili配置
  const bilibiliConfig = ref<BilibiliConfig>({
    id_code: '',
    app_id: 0,
    access_key: '',
    access_secret: '',
    host: 'https://live-open.biliapi.com'
  })

  // 连接状态相关
  const connectionStats = ref({
    totalMessages: 0,
    danmuCount: 0,
    giftCount: 0,
    connectTime: null as Date | null
  })

  // 计算属性
  const isConnectionReady = computed(() => 
    bilibiliConfig.value.id_code && 
    bilibiliConfig.value.app_id && 
    bilibiliConfig.value.access_key && 
    bilibiliConfig.value.access_secret
  )

  // 连接相关方法
  const connectBilibili = async () => {
    await startProxyServer()
    try {
      connectionStatus.value = '连接中...'
      const result = await invoke('connect_bilibili', {
        config: bilibiliConfig.value
      })

      isConnected.value = true
      connectionStatus.value = '已连接'
      connectionStats.value.connectTime = new Date()
      console.log('连接成功:', result)
    } catch (error) {
      console.error('连接失败:', error)
      connectionStatus.value = '连接失败: ' + error
      isConnected.value = false
    }
  }

  const disconnectBilibili = async () => {
    try {
      await invoke('disconnect_bilibili')
      isConnected.value = false
      connectionStatus.value = '未连接'
      connectionStats.value.connectTime = null
      console.log('断开连接成功')
    } catch (error) {
      console.error('断开连接失败:', error)
    } finally {
      await stopProxyServer()
    }
  }

  const checkConnectionStatus = async () => {
    try {
      const status = await invoke('get_connection_status')
      isConnected.value = status as boolean
      connectionStatus.value = status ? '已连接' : '未连接'
    } catch (error) {
      console.error('检查连接状态失败:', error)
    }
  }

  // 代理服务相关方法
  const startProxyServer = async () => {
    try {
      proxyStatus.value = '启动中...'
      const result = await invoke('start_proxy_server', { port: 12345 })
      console.log('代理服务启动结果:', result)

      isProxyRunning.value = true
      proxyStatus.value = '运行中'
    } catch (error) {
      console.error('启动代理服务失败:', error)
      proxyStatus.value = '启动失败'
      isProxyRunning.value = false
      throw new Error('启动代理服务失败: ' + error)
    }
  }

  const stopProxyServer = async () => {
    try {
      const result = await invoke('stop_proxy_server')
      console.log('代理服务停止结果:', result)

      isProxyRunning.value = false
      proxyStatus.value = '未运行'
    } catch (error) {
      console.error('停止代理服务失败:', error)
      throw new Error('停止代理服务失败: ' + error)
    }
  }

  // 加载配置
  const loadConfig = async () => {
    try {
      const result = await invoke('load_config_from_file')
      if (result) {
        bilibiliConfig.value = result as BilibiliConfig
        console.log('自动加载配置文件成功:', result)
        return true
      } else {
        console.log('读取配置文件失败')
        return false
      }
    } catch (error) {
      console.log('加载配置失败:', error)
      return false
    }
  }

  // 更新统计信息
  const updateStats = (messageType: string) => {
    connectionStats.value.totalMessages++
    if (messageType === 'DM') {
      connectionStats.value.danmuCount++
    } else if (messageType === 'SEND_GIFT') {
      connectionStats.value.giftCount++
    }
  }

  const resetStats = () => {
    connectionStats.value = {
      totalMessages: 0,
      danmuCount: 0,
      giftCount: 0,
      connectTime: connectionStats.value.connectTime
    }
  }

  return {
    // 状态
    isConnected,
    connectionStatus,
    isProxyRunning,
    proxyStatus,
    bilibiliConfig,
    connectionStats,
    
    // 计算属性
    isConnectionReady,
    
    // 方法
    connectBilibili,
    disconnectBilibili,
    checkConnectionStatus,
    startProxyServer,
    stopProxyServer,
    loadConfig,
    updateStats,
    resetStats
  }
}
