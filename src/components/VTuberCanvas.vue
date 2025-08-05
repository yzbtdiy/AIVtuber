<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { useVTuberManager } from '../composables/useVTuberManager'

const canvasRef = ref<HTMLCanvasElement>()

// 使用 VTuber Manager composable
const {
  isLoading,
  isInitialized,
  initVTuber,
  loadModel,
  loadIdleAnimation,
  setEmotion,
  testLipSync,
  startRealTimeLipSync,
  stopLipSync,
  playGesture,
  resetPose,
  processText,
  playAudioFile,
  playAudio,
  dispose
} = useVTuberManager()

// 公开方法供父组件调用
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
  isLoading,
  isInitialized
})

// 组件生命周期
onMounted(async () => {
  await nextTick()

  if (canvasRef.value) {
    try {
      await initVTuber(canvasRef.value)
      await loadModel('/vroid.vrm')
      await loadIdleAnimation('/idle.vrma')
      console.log('VTuber Canvas 初始化完成')
    } catch (error) {
      console.error('VTuber Canvas 初始化失败:', error)
    }
  }
})

onUnmounted(() => {
  dispose()
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
