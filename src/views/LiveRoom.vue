<script setup lang="ts">
import { ref, onMounted } from 'vue'
import Danmu from '../components/Danmu.vue'
import StatusBar from '../components/StatusBar.vue'
import VTuberCanvas from '../components/VTuberCanvas.vue'

// Composables
import { useBilibiliConnection } from '../composables/useBilibiliConnection'
import { useMessageHandler } from '../composables/useMessageHandler'
import { useDanmuConfig } from '../composables/useDanmuConfig'
import { useBilibiliEventListener } from '../composables/useBilibiliEventListener'

// 组件引用
const vtuberCanvasRef = ref<InstanceType<typeof VTuberCanvas>>()
const danmuRef = ref<InstanceType<typeof Danmu>>()

// 使用composables
const {
  isConnected,
  connectionStatus,
  connectionStats,
  connectBilibili,
  disconnectBilibili,
  checkConnectionStatus,
  loadConfig,
  updateStats,
  resetStats
} = useBilibiliConnection()

const {
  danmus,
  addMessage,
  clearMessages
} = useMessageHandler()

const {
  danmuConfig,
  activeDanmusCount,
  toggleDanmu,
  updateActiveCount
} = useDanmuConfig()

const { startListening } = useBilibiliEventListener()

// 工具函数
const playAudio = (audioData: ArrayBuffer) => {
  vtuberCanvasRef.value?.playAudio(audioData)
}

// 事件处理函数
const handleToggleDanmu = () => {
  toggleDanmu()
}

const handleClearAllDanmus = () => {
  clearMessages()
  resetStats()
  console.log('清空所有弹幕')
}

const handleUpdateActiveCount = (count: number) => {
  updateActiveCount(count)
}

// 初始化
onMounted(async () => {
  try {
    // 检查连接状态
    await checkConnectionStatus()
    
    // 尝试自动加载配置文件
    await loadConfig()
    
    // 开始监听bilibili消息
    await startListening((message) => {
      addMessage(message, {
        playAudio,
        danmuRef: danmuRef.value,
        updateStats
      })
    })
    
    console.log('LiveRoom 初始化完成')
  } catch (error) {
    console.error('初始化直播间失败:', error)
  }
})
</script>

<template>
  <!-- 状态栏组件 -->
  <VTuberCanvas ref="vtuberCanvasRef" />
  <StatusBar :is-connected="isConnected" :connection-status="connectionStatus" :connection-stats="connectionStats"
    :danmu-config="danmuConfig" :active-danmus-count="activeDanmusCount" @connect-bilibili="connectBilibili"
    @disconnect-bilibili="disconnectBilibili" @toggle-danmu="handleToggleDanmu"
    @clear-all-danmus="handleClearAllDanmus" />

  <!-- 弹幕组件 -->
  <Danmu ref="danmuRef" :danmus="danmus" :connection-stats="connectionStats" :is-connected="isConnected"
    :danmu-config="danmuConfig" @clear-messages="clearMessages" @update-active-count="handleUpdateActiveCount" />
</template>

<style scoped>
/* LiveRoom 专用样式，主要样式已迁移到 StatusBar */
</style>