import { ref } from 'vue'

export interface DanmuConfig {
  enabled: boolean
  speed: number
  fontSize: number
  opacity: number
  maxRows: number
  minInterval: number
}

export function useDanmuConfig() {
  // 弹幕配置
  const danmuConfig = ref<DanmuConfig>({
    enabled: true,
    speed: 50,
    fontSize: 16,
    opacity: 0.9,
    maxRows: 10,
    minInterval: 500,
  })

  // 活跃弹幕数量
  const activeDanmusCount = ref(0)

  // 弹幕配置相关方法
  const toggleDanmu = () => {
    danmuConfig.value.enabled = !danmuConfig.value.enabled
    console.log('切换弹幕状态:', danmuConfig.value.enabled)
  }

  const updateDanmuConfig = (config: Partial<DanmuConfig>) => {
    danmuConfig.value = { ...danmuConfig.value, ...config }
    console.log('更新弹幕配置:', danmuConfig.value)
  }

  const updateActiveCount = (count: number) => {
    activeDanmusCount.value = count
    console.log('活跃弹幕数量:', count)
  }

  // 重置弹幕配置到默认值
  const resetDanmuConfig = () => {
    danmuConfig.value = {
      enabled: true,
      speed: 50,
      fontSize: 16,
      opacity: 0.9,
      maxRows: 10,
      minInterval: 500,
    }
  }

  return {
    // 状态
    danmuConfig,
    activeDanmusCount,
    
    // 方法
    toggleDanmu,
    updateDanmuConfig,
    updateActiveCount,
    resetDanmuConfig
  }
}
