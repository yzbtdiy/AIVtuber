/**
 * VRM相关类型定义
 */

// 口型同步权重
export interface LipSyncWeights {
  a: number
  i: number
  u: number
  e: number
  o: number
}

// 情感分析结果
export interface EmotionResult {
  dominantEmotion: string
  confidence: number
  emotions: Record<string, number>
}

// VTuber配置
export interface VTuberConfig {
  modelPath: string
  animationPath?: string
  lipSyncSensitivity: number
  emotionIntensity: number
}

// 音频分析配置
export interface AudioAnalysisConfig {
  fftSize: number
  smoothingTimeConstant: number
  minDecibels: number
  maxDecibels: number
}

// 表情映射
export interface ExpressionMapping {
  happy: string[]
  sad: string[]
  angry: string[]
  surprised: string[]
  neutral: string[]
}

// 手势类型
export type GestureType = 'nod' | 'shake' | 'wave' | 'point' | 'clap'

// 情感类型
export type EmotionType = 'happy' | 'sad' | 'angry' | 'surprised' | 'neutral' | 'excited' | 'calm'

// 口型形状
export type MouthShape = 'a' | 'i' | 'u' | 'e' | 'o' | 'silence'
