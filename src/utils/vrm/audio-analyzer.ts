import type { AudioAnalysisConfig } from './types'

/**
 * 音频分析器
 * 负责处理音频输入和分析，生成口型同步数据
 */
export class AudioAnalyzer {
  private audioContext: AudioContext | null = null
  private analyser: AnalyserNode | null = null
  private dataArray: Uint8Array | null = null
  private isAnalyzing = false
  private animationFrame: number | null = null
  private currentSource: AudioBufferSourceNode | null = null

  // 默认配置
  private readonly defaultConfig: AudioAnalysisConfig = {
    fftSize: 256,
    smoothingTimeConstant: 0.8,
    minDecibels: -90,
    maxDecibels: -10
  }

  /**
   * 初始化音频分析器
   */
  async init(config: Partial<AudioAnalysisConfig> = {}): Promise<boolean> {
    try {
      this.audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
      this.analyser = this.audioContext.createAnalyser()
      
      const finalConfig = { ...this.defaultConfig, ...config }
      this.analyser.fftSize = finalConfig.fftSize
      this.analyser.smoothingTimeConstant = finalConfig.smoothingTimeConstant
      this.analyser.minDecibels = finalConfig.minDecibels
      this.analyser.maxDecibels = finalConfig.maxDecibels

      const bufferLength = this.analyser.frequencyBinCount
      this.dataArray = new Uint8Array(bufferLength)

      console.log('AudioAnalyzer initialized with config:', finalConfig)
      return true
    } catch (error) {
      console.error('Failed to initialize AudioAnalyzer:', error)
      return false
    }
  }

  async startMicrophoneAnalysis(onVolumeUpdate: (volume: number, frequencies: number[]) => void) {
    if (!this.audioContext || !this.analyser || !this.dataArray) {
      console.error('AudioAnalyzer not initialized')
      return false
    }

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      const source = this.audioContext.createMediaStreamSource(stream)
      source.connect(this.analyser)

      this.isAnalyzing = true
      this.analyze(onVolumeUpdate)

      console.log('Microphone analysis started')
      return true
    } catch (error) {
      console.error('Failed to access microphone:', error)
      return false
    }
  }

  async loadAndAnalyzeAudioFile(audioUrl: string, onVolumeUpdate: (volume: number, frequencies: number[]) => void) {
    if (!this.audioContext || !this.analyser) {
      console.error('AudioAnalyzer not initialized')
      return false
    }

    try {
      // 停止当前播放的音频
      this.stopCurrentAudio()

      console.log('Loading audio file:', audioUrl)

      // 获取音频文件
      const response = await fetch(audioUrl)
      if (!response.ok) {
        throw new Error(`Failed to fetch audio file: ${response.statusText}`)
      }

      const arrayBuffer = await response.arrayBuffer()
      return await this.loadAndAnalyzeAudioBuffer(arrayBuffer, onVolumeUpdate)
    } catch (error) {
      console.error('Failed to load and analyze audio file:', error)
      return false
    }
  }

  /**
   * 加载并分析音频 ArrayBuffer 数据
   * @param arrayBuffer 音频的二进制数据
   * @param onVolumeUpdate 音频分析回调函数
   * @returns 是否成功开始播放和分析
   */
  async loadAndAnalyzeAudioBuffer(arrayBuffer: ArrayBuffer, onVolumeUpdate: (volume: number, frequencies: number[]) => void) {
    if (!this.audioContext || !this.analyser) {
      console.error('AudioAnalyzer not initialized')
      return false
    }

    try {
      // 停止当前播放的音频
      this.stopCurrentAudio()

      console.log(`Processing audio buffer, size: ${arrayBuffer.byteLength} bytes`)

      const audioBuffer = await this.audioContext.decodeAudioData(arrayBuffer)

      console.log(`Audio buffer decoded successfully. Duration: ${audioBuffer.duration.toFixed(2)}s`)

      // 创建音频源并连接到分析器
      this.currentSource = this.audioContext.createBufferSource()
      this.currentSource.buffer = audioBuffer

      // 确保分析器设置正确
      this.analyser.fftSize = 512  // 增加FFT大小以获得更好的频率分辨率
      this.analyser.smoothingTimeConstant = 0.3  // 添加平滑处理

      this.currentSource.connect(this.analyser)

      // 也连接到音频输出，这样可以听到声音
      this.currentSource.connect(this.audioContext.destination)

      // 开始分析
      this.isAnalyzing = true
      console.log('Starting audio analysis...')
      this.analyze(onVolumeUpdate)

      // 播放音频
      this.currentSource.start()

      // 当音频播放完成时停止分析
      this.currentSource.onended = () => {
        console.log('Audio playback finished')
        this.stopAnalysis()
        this.currentSource = null

        // 重置口型到默认状态
        onVolumeUpdate(0, [0, 0, 0, 0, 0])
      }

      console.log('Audio analysis and playback started')
      return true
    } catch (error) {
      console.error('Failed to load and analyze audio buffer:', error)
      return false
    }
  }

  stopCurrentAudio() {
    if (this.currentSource) {
      try {
        this.currentSource.stop()
        this.currentSource.disconnect()
      } catch (error) {
        // 忽略已经停止的音频源的错误
      }
      this.currentSource = null
    }
    // 注意：这里不调用 stopAnalysis()，让分析继续进行
  }

  analyzeAudioFile(audioBuffer: AudioBuffer, onVolumeUpdate: (volume: number, frequencies: number[]) => void) {
    if (!this.audioContext || !this.analyser) {
      console.error('AudioAnalyzer not initialized')
      return
    }

    const source = this.audioContext.createBufferSource()
    source.buffer = audioBuffer
    source.connect(this.analyser)
    source.start()

    this.isAnalyzing = true
    this.analyze(onVolumeUpdate)
  }

  private analyze(onVolumeUpdate: (volume: number, frequencies: number[]) => void) {
    if (!this.analyser || !this.dataArray || !this.isAnalyzing) return

    this.analyser.getByteFrequencyData(this.dataArray as Uint8Array<ArrayBuffer>)

    // 计算总音量
    let sum = 0
    for (let i = 0; i < this.dataArray.length; i++) {
      sum += this.dataArray[i]
    }
    const volume = sum / this.dataArray.length / 255

    // 提取关键频率范围
    const frequencies = [
      this.getFrequencyRange(0, 10),   // 低频
      this.getFrequencyRange(10, 30),  // 中低频
      this.getFrequencyRange(30, 60),  // 中频
      this.getFrequencyRange(60, 100), // 中高频
      this.getFrequencyRange(100, Math.min(128, this.dataArray.length - 1)) // 高频
    ]

    // 添加调试日志（每秒只记录一次）
    if (Math.random() < 0.016) { // 大约每秒一次（假设60fps）
      console.log('Audio analysis:', {
        volume: volume.toFixed(3),
        frequencies: frequencies.map(f => f.toFixed(3)),
        isAnalyzing: this.isAnalyzing
      })
    }

    onVolumeUpdate(volume, frequencies)

    if (this.isAnalyzing) {
      this.animationFrame = requestAnimationFrame(() => this.analyze(onVolumeUpdate))
    }
  }

  private getFrequencyRange(start: number, end: number): number {
    if (!this.dataArray) return 0

    let sum = 0
    let count = 0
    for (let i = start; i < Math.min(end, this.dataArray.length); i++) {
      sum += this.dataArray[i]
      count++
    }
    return count > 0 ? (sum / count) / 255 : 0
  }

  stopAnalysis() {
    this.isAnalyzing = false
    if (this.animationFrame !== null) {
      cancelAnimationFrame(this.animationFrame)
      this.animationFrame = null
    }
    console.log('Audio analysis stopped')
  }

  dispose() {
    this.stopCurrentAudio()
    if (this.audioContext) {
      this.audioContext.close()
      this.audioContext = null
    }
  }

  // 根据音频特征计算口型权重
  static calculateLipSyncWeights(volume: number, frequencies: number[]): { [key: string]: number } {
    const weights: { [key: string]: number } = {
      a: 0,
      i: 0,
      u: 0,
      e: 0,
      o: 0
    }

    // 降低静音阈值，使其对音频文件更敏感
    if (volume < 0.05) {
      // 安静时，嘴巴略微张开
      weights.a = 0.05
      return weights
    }

    // 基于频率特征映射到不同口型
    const [low, midLow, mid, midHigh, high] = frequencies

    // 增加权重乘数，使口型变化更明显
    const volumeMultiplier = Math.min(volume * 2.0, 1.0)

    // 'a' 音：低频为主，嘴巴大张
    if (low > 0.2 || (midLow > 0.15 && volume > 0.1)) {
      weights.a = Math.min(volumeMultiplier * 1.2, 1.0)
    }

    // 'i' 音：高频为主，嘴巴横向
    if (high > 0.2 || (midHigh > 0.2 && volume > 0.1)) {
      weights.i = Math.min(volumeMultiplier * 1.0, 1.0)
    }

    // 'u' 音：中频为主，嘴巴圆形
    if (mid > 0.2 && high < 0.2) {
      weights.u = Math.min(volumeMultiplier * 1.1, 1.0)
    }

    // 'e' 音：中高频，嘴巴中等张开
    if (midHigh > 0.15 || (mid > 0.2 && volume > 0.1)) {
      weights.e = Math.min(volumeMultiplier * 0.9, 1.0)
    }

    // 'o' 音：中低频为主，嘴巴圆形但比'u'更张开
    if (midLow > 0.2 || (low > 0.15 && volume > 0.1)) {
      weights.o = Math.min(volumeMultiplier * 1.0, 1.0)
    }

    // 确保至少有一个权重，避免完全静音
    const totalWeight = Object.values(weights).reduce((sum, w) => sum + w, 0)
    if (totalWeight < 0.1 && volume > 0.02) {
      weights.a = Math.min(volume * 1.5, 0.6)
    }

    // 添加调试日志
    if (Math.random() < 0.01) { // 偶尔记录
      console.log('LipSync weights:', {
        volume: volume.toFixed(3),
        frequencies: frequencies.map(f => f.toFixed(3)),
        weights: Object.fromEntries(Object.entries(weights).map(([k, v]) => [k, v.toFixed(3)]))
      })
    }

    return weights
  }
}
