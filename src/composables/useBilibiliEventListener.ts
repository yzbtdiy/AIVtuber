import { onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { BilibiliLiveMessage } from '../types/bilibili.types'

export function useBilibiliEventListener() {
  let messageUnlisten: UnlistenFn | null = null

  const startListening = async (
    onMessage: (message: BilibiliLiveMessage) => void
  ) => {
    try {
      messageUnlisten = await listen('bilibili-message', (event) => {
        const message = event.payload as BilibiliLiveMessage
        console.log('收到bilibili消息:', message)
        onMessage(message)
      })
      console.log('开始监听bilibili消息')
    } catch (error) {
      console.error('启动消息监听失败:', error)
    }
  }

  const stopListening = () => {
    if (messageUnlisten) {
      messageUnlisten()
      messageUnlisten = null
      console.log('停止监听bilibili消息')
    }
  }

  // 生命周期钩子
  onUnmounted(() => {
    stopListening()
  })

  return {
    startListening,
    stopListening
  }
}
