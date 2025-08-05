import { ref } from 'vue'
import type { 
  BilibiliLiveMessage, 
  FormattedMessage,
  DanmakuMessage,
  GiftMessage,
  SuperChatMessage,
  SuperChatDelMessage,
  GuardMessage,
  LikeMessage,
  RoomEnterMessage,
  LiveStartMessage,
  LiveEndMessage,
  InteractionEndMessage
} from '../types/bilibili.types'
import { LivePlatformCmd } from '../types/bilibili.types'
import { chatAndSpeakIntegrated, base64ToArrayBuffer } from '../api/chat'

export function useMessageHandler() {
  // 消息状态
  const messages = ref<FormattedMessage[]>([])
  const danmus = ref<DanmakuMessage[]>([])

  // 类型安全的消息格式化函数
  const formatMessage = async (message: BilibiliLiveMessage): Promise<FormattedMessage> => {
    const baseMessage = {
      id: Date.now() + Math.random(),
      timestamp: new Date().toLocaleString(),
      raw: message
    }

    switch (message.cmd) {
      case LivePlatformCmd.DM: {
        const data = message.data as DanmakuMessage
        return {
          ...baseMessage,
          type: 'danmu' as const,
          content: data.msg,
          user: {
            name: data.uname,
            avatar: data.uface,
            open_id: data.open_id
          }
        }
      }

      case LivePlatformCmd.SEND_GIFT: {
        const data = message.data as GiftMessage
        return {
          ...baseMessage,
          type: 'gift' as const,
          content: `送出了 ${data.gift_num} 个 ${data.gift_name}`,
          user: {
            name: data.uname,
            avatar: data.uface,
            open_id: data.open_id
          }
        }
      }

      case LivePlatformCmd.SUPER_CHAT: {
        const data = message.data as SuperChatMessage
        return {
          ...baseMessage,
          type: 'superchat' as const,
          content: `${data.message} (${data.rmb}元)`,
          user: {
            name: data.uname,
            avatar: data.uface,
            open_id: data.open_id
          }
        }
      }

      case LivePlatformCmd.SUPER_CHAT_DEL: {
        const data = message.data as SuperChatDelMessage
        return {
          ...baseMessage,
          type: 'system' as const,
          content: `删除了 ${data.message_ids.length} 条醒目留言`
        }
      }

      case LivePlatformCmd.GUARD: {
        const data = message.data as GuardMessage
        const guardLevelText = data.guard_level === 1 ? '总督' : data.guard_level === 2 ? '提督' : '舰长'
        return {
          ...baseMessage,
          type: 'guard' as const,
          content: `购买了 ${guardLevelText}`,
          user: {
            name: data.user_info.uname,
            avatar: data.user_info.uface,
            open_id: data.user_info.open_id
          }
        }
      }

      case LivePlatformCmd.LIKE: {
        const data = message.data as LikeMessage
        return {
          ...baseMessage,
          type: 'like' as const,
          content: `${data.like_text} ${data.like_count} 次`,
          user: {
            name: data.uname,
            avatar: data.uface,
            open_id: data.open_id
          }
        }
      }

      case LivePlatformCmd.LIVE_ROOM_ENTER: {
        const data = message.data as RoomEnterMessage
        return {
          ...baseMessage,
          type: 'enter' as const,
          content: '进入了直播间',
          user: {
            name: data.uname,
            avatar: data.uface,
            open_id: data.open_id
          }
        }
      }

      case LivePlatformCmd.LIVE_START: {
        const data = message.data as LiveStartMessage
        return {
          ...baseMessage,
          type: 'system' as const,
          content: `直播间 ${data.room_id} 开始直播，标题：${data.title}`
        }
      }

      case LivePlatformCmd.LIVE_END: {
        const data = message.data as LiveEndMessage
        return {
          ...baseMessage,
          type: 'system' as const,
          content: `直播间 ${data.room_id} 结束直播`
        }
      }

      case LivePlatformCmd.INTERACTION_END: {
        const data = message.data as InteractionEndMessage
        return {
          ...baseMessage,
          type: 'system' as const,
          content: `连接 ${data.game_id} 已结束`
        }
      }

      default:
        return {
          ...baseMessage,
          type: 'system' as const,
          content: `未知消息类型: ${message.cmd}`
        }
    }
  }

  // 处理弹幕消息并生成AI回复
  const processDanmuMessage = async (
    danmuData: DanmakuMessage,
    playAudio: (audioData: ArrayBuffer) => void
  ) => {
    try {
      const chatResp = await chatAndSpeakIntegrated(danmuData.msg)
      if (chatResp.audio_data) {
        playAudio(base64ToArrayBuffer(chatResp.audio_data))
      }
    } catch (error) {
      console.error('处理弹幕AI回复失败:', error)
    }
  }

  // 添加消息的主要处理函数
  const addMessage = async (
    message: BilibiliLiveMessage,
    options: {
      playAudio?: (audioData: ArrayBuffer) => void
      danmuRef?: { addSingleDanmu?: (danmu: DanmakuMessage) => void }
      updateStats?: (messageType: string) => void
    } = {}
  ) => {
    console.log('addMessage called with:', message)

    const formattedMessage = await formatMessage(message)

    // 更新统计
    options.updateStats?.(message.cmd)

    // 添加到消息列表
    messages.value.unshift(formattedMessage)

    // 特殊处理弹幕消息
    if (message.cmd === LivePlatformCmd.DM) {
      const danmuData = message.data as DanmakuMessage
      console.log('Processing danmu message:', danmuData)

      // 处理AI回复
      if (options.playAudio) {
        await processDanmuMessage(danmuData, options.playAudio)
      }

      // 显示弹幕
      if (options.danmuRef?.addSingleDanmu) {
        console.log('直接调用Danmu组件方法显示弹幕:', danmuData)
        options.danmuRef.addSingleDanmu(danmuData)
      } else {
        console.log('Danmu组件引用不可用，无法显示弹幕')
      }

      // 添加到弹幕列表
      danmus.value.unshift(danmuData)

      // 限制弹幕列表长度
      if (danmus.value.length > 50) {
        danmus.value.splice(50)
      }
    }

    // 保留所有类型消息，但限制总数
    if (messages.value.length > 200) {
      messages.value = messages.value.slice(0, 200)
    }
  }

  // 清空消息
  const clearMessages = () => {
    messages.value = []
    danmus.value = []
  }

  return {
    // 状态
    messages,
    danmus,
    
    // 方法
    formatMessage,
    processDanmuMessage,
    addMessage,
    clearMessages
  }
}
