import type { EmotionType } from './types'

// 情感分析结果接口
export interface EmotionResult {
  dominantEmotion: string
  confidence: number
  emotions: Record<string, number>
  intensity: number
  foundKeywords: string[]
  hasEmotionMarkers: boolean
}

/**
 * 情感分析器
 * 负责分析文本内容并识别情感，用于控制VTuber表情
 */
export class EmotionAnalyzer {
  // 情感关键词映射
  private static readonly emotionKeywords: Record<EmotionType, string[]> = {
    happy: ['高兴', '开心', '快乐', '愉快', '兴奋', '喜悦', '欢喜', '乐', '哈哈', '😄', '😊', '😃', '🎉'],
    sad: ['难过', '伤心', '悲伤', '沮丧', '失落', '郁闷', '痛苦', '哭', '😢', '😭', '😞', '☹️'],
    angry: ['生气', '愤怒', '恼火', '气愤', '暴怒', '怒', '烦躁', '😠', '😡', '🤬'],
    surprised: ['惊讶', '震惊', '惊喜', '意外', '吃惊', '诧异', '😲', '😮', '🤯', '😱'],
    neutral: ['平静', '冷静', '淡定', '平常', '正常', '一般', '还好', '🙂', '😐'],
    excited: ['兴奋', '激动', '热情', '狂欢', '激昂', '🤩', '😆', '🥳'],
    calm: ['放松', '舒缓', '宁静', '安详', '轻松', '�', '🧘']
  }

  // VRM表情映射
  private static readonly vrmExpressionMap: Record<EmotionType, string> = {
    happy: 'happy',
    sad: 'sad',
    angry: 'angry',
    surprised: 'surprised',
    neutral: 'neutral',
    excited: 'happy', // 兴奋映射到高兴
    calm: 'relaxed'   // 平静映射到放松
  }

  /**
   * 获取VRM表情名称
   */
  static getVRMExpression(emotion: string): string {
    return this.vrmExpressionMap[emotion as EmotionType] || 'neutral'
  }

  /**
   * 分析文本中的情感
   * @param text 要分析的文本
   * @returns 情感分析结果
   */
  static analyzeEmotion(text: string): EmotionResult {
    const emotions: Record<string, number> = {}
    const foundKeywords: string[] = []

    // 初始化情感分数
    Object.keys(this.emotionKeywords).forEach(emotion => {
      emotions[emotion] = 0
    })

    // 扫描文本中的情感关键词
    Object.entries(this.emotionKeywords).forEach(([emotion, keywords]) => {
      keywords.forEach(keyword => {
        const regex = new RegExp(keyword, 'gi')
        const matches = text.match(regex)
        if (matches) {
          emotions[emotion] += matches.length
          foundKeywords.push(...matches)
        }
      })
    })

    // 检测特殊标记，如 [高兴]、[难过] 等
    const emotionMarkers = text.match(/\[(.*?)\]/g)
    if (emotionMarkers) {
      emotionMarkers.forEach(marker => {
        const emotion = marker.replace(/\[|\]/g, '')
        Object.entries(this.emotionKeywords).forEach(([key, keywords]) => {
          if (keywords.includes(emotion)) {
            emotions[key] += 2 // 标记的权重更高
            foundKeywords.push(emotion)
          }
        })
      })
    }

    // 找出主导情感
    let dominantEmotion = 'neutral'
    let maxScore = 0
    
    Object.entries(emotions).forEach(([emotion, score]) => {
      if (score > maxScore) {
        maxScore = score
        dominantEmotion = emotion
      }
    })

    // 计算情感强度 (0-1)
    const totalKeywords = foundKeywords.length
    const intensity = Math.min(totalKeywords / 3, 1) // 最多3个关键词达到最大强度
    const confidence = maxScore > 0 ? maxScore / Math.max(1, totalKeywords) : 0

    return {
      dominantEmotion,
      confidence,
      intensity,
      emotions,
      foundKeywords,
      hasEmotionMarkers: emotionMarkers !== null
    }
  }

  /**
   * 根据文本内容生成相应的动作建议
   * @param text 文本内容
   * @param emotion 情感分析结果
   * @returns 动作建议
   */
  static suggestGesture(text: string, emotion: EmotionResult): string | null {
    // 检测特定动作关键词
    const gestureKeywords = {
      nod: ['点头', '嗯', '是的', '对', '同意', '好的'],
      shake: ['摇头', '不', '不是', '否定'],
      clap: ['鼓掌', '拍手', '棒', '厉害', '赞'],
      think: ['思考', '想想', '让我想想', '嗯...', '考虑']
    }

    for (const [gesture, keywords] of Object.entries(gestureKeywords)) {
      for (const keyword of keywords) {
        if (text.includes(keyword)) {
          return gesture
        }
      }
    }

    // 根据情感建议动作
    switch (emotion.dominantEmotion) {
      case 'happy':
        return 'nod'
      case 'excited':
        return 'nod'
      case 'surprised':
        return 'nod'
      case 'confused':
        return 'think'
      case 'neutral':
        return null
      default:
        return null
    }
  }

  /**
   * 创建带有情感标记的文本
   * @param text 原始文本
   * @param emotion 要添加的情感
   * @returns 带有情感标记的文本
   */
  static addEmotionMarker(text: string, emotion: string): string {
    const emotionNames: Record<string, string> = {
      happy: '高兴',
      sad: '难过',
      angry: '生气',
      surprised: '惊讶',
      neutral: '平静'
    }

    const emotionName = emotionNames[emotion] || emotion
    return `[${emotionName}]${text}`
  }
}
