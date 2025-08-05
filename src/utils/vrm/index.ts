/**
 * VRM工具模块
 * 
 * 提供VTuber相关的核心功能，包括：
 * - 音频分析和口型同步
 * - 情感分析和表情控制  
 * - VTuber模型管理
 */

export { AudioAnalyzer } from './audio-analyzer'
export { EmotionAnalyzer } from './emotion-analyzer'
export { VTuberManager } from './vtuber-manager'

// 导出常用类型
export type {
  LipSyncWeights,
  EmotionResult,
  VTuberConfig
} from './types'
