<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { VTuberManager } from '../utils/vrm/VTuberManager'

const canvasRef = ref<HTMLCanvasElement>()
const isLoading = ref(true)
let vtuberManager: VTuberManager | null = null

// 公开方法供父组件调用
const setEmotion = (emotion: string) => {
  vtuberManager?.setEmotion(emotion)
}

const testLipSync = () => {
  vtuberManager?.testLipSync()
}

const startRealTimeLipSync = () => {
  vtuberManager?.startRealTimeLipSync()
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
  vtuberManager?.processText(text)
}

const playAudioFile = (audioUrl: string) => {
  vtuberManager?.playAudioFile(audioUrl)
}

const playAudio = (audioData: ArrayBuffer) => {
  vtuberManager?.playAudio(audioData)
}

defineExpose({
  setEmotion,
  testLipSync,
  startRealTimeLipSync,
  stopLipSync,
  playGesture,
  resetPose,
  processText,
  playAudioFile,
  playAudio,
  playIdleAnimation: () => vtuberManager?.playIdleAnimation(),
  stopIdleAnimation: () => vtuberManager?.stopIdleAnimation(),
  toggleIdleAnimation: () => vtuberManager?.toggleIdleAnimation()
})

onMounted(async () => {
  await nextTick()

  if (canvasRef.value) {
    try {
      vtuberManager = new VTuberManager()
      await vtuberManager.init(canvasRef.value)
      await vtuberManager.loadModel('/vroid.vrm')
      await vtuberManager.loadIdleAnimation('/idle.vrma')
      isLoading.value = false
    } catch (error) {
      console.error('Failed to initialize VTuber:', error)
      isLoading.value = false
    }
  }
})

onUnmounted(() => {
  vtuberManager?.dispose()
})
</script>

<template>
  <div class="vtuber-container">
    <canvas ref="canvasRef" class="vtuber-canvas"></canvas>
    <div class="loading" v-if="isLoading">
      <div class="spinner"></div>
      <p>正在加载虚拟主播模型...</p>
    </div>
  </div>
</template>

<style scoped>
.vtuber-container {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.vtuber-canvas {
  width: 100%;
  height: 100%;
  display: block;
  background: transparent;
}

.loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  color: white;
  font-size: 18px;
  z-index: 100;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(255, 255, 255, 0.3);
  border-top: 4px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}
</style>
