import { ref } from 'vue'
import { VTuberManager } from '../utils/vrm/vtuber-manager'

export function useVTuberManager() {
  const isLoading = ref(true)
  const isInitialized = ref(false)
  let vtuberManager: VTuberManager | null = null

  const initVTuber = async (canvas: HTMLCanvasElement) => {
    try {
      isLoading.value = true
      vtuberManager = new VTuberManager()
      await vtuberManager.init(canvas)
      isInitialized.value = true
      console.log('VTuber Manager 初始化完成')
    } catch (error) {
      console.error('VTuber Manager 初始化失败:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  const loadModel = async (modelPath: string) => {
    if (!vtuberManager) {
      throw new Error('VTuber Manager 未初始化')
    }
    try {
      await vtuberManager.loadModel(modelPath)
      console.log('VRM 模型加载完成')
    } catch (error) {
      console.error('VRM 模型加载失败:', error)
      throw error
    }
  }

  const loadIdleAnimation = async (animationPath: string) => {
    if (!vtuberManager) {
      throw new Error('VTuber Manager 未初始化')
    }
    try {
      await vtuberManager.loadIdleAnimation(animationPath)
      console.log('闲置动画加载完成')
    } catch (error) {
      console.error('闲置动画加载失败:', error)
      throw error
    }
  }

  // VTuber 控制方法
  const setEmotion = (emotion: string) => {
    vtuberManager?.setEmotion(emotion)
  }

  const testLipSync = () => {
    vtuberManager?.testLipSync()
  }

  const startRealTimeLipSync = async () => {
    try {
      await vtuberManager?.startRealTimeLipSync()
    } catch (error) {
      console.error('启动实时口型同步失败:', error)
    }
  }

  const stopLipSync = () => {
    vtuberManager?.stopLipSync()
  }

  const playGesture = (gesture: string) => {
    vtuberManager?.playGesture(gesture)
  }

  const resetPose = () => {
    vtuberManager?.resetPose()
  }

  const processText = (text: string) => {
    return vtuberManager?.processText(text)
  }

  const playAudioFile = async (audioUrl: string) => {
    try {
      return await vtuberManager?.playAudioFile(audioUrl)
    } catch (error) {
      console.error('播放音频文件失败:', error)
      return false
    }
  }

  const playAudio = async (audioData: ArrayBuffer, mimeType?: string) => {
    try {
      return await vtuberManager?.playAudio(audioData, mimeType)
    } catch (error) {
      console.error('播放音频数据失败:', error)
      return false
    }
  }

  // 清理资源
  const dispose = () => {
    if (vtuberManager) {
      vtuberManager.dispose()
      vtuberManager = null
      isInitialized.value = false
    }
  }

  return {
    // 状态
    isLoading,
    isInitialized,
    
    // 初始化方法
    initVTuber,
    loadModel,
    loadIdleAnimation,
    
    // 控制方法
    setEmotion,
    testLipSync,
    startRealTimeLipSync,
    stopLipSync,
    playGesture,
    resetPose,
    processText,
    playAudioFile,
    playAudio,
    
    // 清理方法
    dispose
  }
}
