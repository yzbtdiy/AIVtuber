import * as THREE from 'three'
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js'
import { VRMLoaderPlugin, VRMUtils, VRM } from '@pixiv/three-vrm'
import { VRMAnimationLoaderPlugin, createVRMAnimationClip, VRMLookAtQuaternionProxy } from '@pixiv/three-vrm-animation'
import { AudioAnalyzer } from './audio-analyzer'
import { EmotionAnalyzer } from './emotion-analyzer'

export class VTuberManager {
  private renderer!: THREE.WebGLRenderer
  private scene!: THREE.Scene
  private camera!: THREE.PerspectiveCamera
  private clock!: THREE.Clock
  private vrm: VRM | null = null
  private mixer: THREE.AnimationMixer | null = null
  private idleClip: THREE.AnimationClip | null = null
  private audioAnalyzer: AudioAnalyzer | null = null
  private lipSyncBlendShapes: string[] = ['a', 'i', 'u', 'e', 'o']
  private lipSyncMapping: { [key: string]: string[] } = {
    'a': ['a', 'aa', 'A', 'mouth_a'],
    'i': ['i', 'ii', 'I', 'mouth_i'], 
    'u': ['u', 'uu', 'U', 'mouth_u'],
    'e': ['e', 'ee', 'E', 'mouth_e'],
    'o': ['o', 'oo', 'O', 'mouth_o']
  }
  private currentEmotion = 'neutral'
  private isLipSyncActive = false
  private isRealTimeLipSync = false
  private animationId: number | null = null

  async init(canvas: HTMLCanvasElement) {
    // 初始化渲染器
    this.renderer = new THREE.WebGLRenderer({ 
      canvas,
      alpha: true,
      antialias: true 
    })
    this.renderer.setSize(canvas.clientWidth, canvas.clientHeight)
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
    this.renderer.outputColorSpace = THREE.SRGBColorSpace
    this.renderer.shadowMap.enabled = false

    // 初始化场景
    this.scene = new THREE.Scene()

    // 初始化相机 - 调整位置以显示上半身并放大
    this.camera = new THREE.PerspectiveCamera(
      30,
      canvas.clientWidth / canvas.clientHeight,
      0.1,
      100
    )
    // 相机位置：适中距离以显示上半身
    this.camera.position.set(0.15, 1, 1.2)
    // 查看点：聚焦在胸部位置
    this.camera.lookAt(0, 0.9, 0)

    // 设置光照
    this.setupLighting()

    // 初始化时钟
    this.clock = new THREE.Clock()

    // 开始渲染循环
    this.startRenderLoop()

    // 处理窗口大小变化
    this.handleResize(canvas)
  }

  private setupLighting() {
    // 主光源
    const mainLight = new THREE.DirectionalLight(0xffffff, 3)
    mainLight.position.set(1, 1, 1)
    mainLight.castShadow = false
    this.scene.add(mainLight)

    // 环境光
    const ambientLight = new THREE.AmbientLight(0x404040, 1)
    this.scene.add(ambientLight)

    // 补光
    const fillLight = new THREE.DirectionalLight(0x8888ff, 0.5)
    fillLight.position.set(-1, 0.5, -1)
    this.scene.add(fillLight)
  }

  async loadModel(modelPath: string) {
    const loader = new GLTFLoader()
    
    // 注册VRM插件
    loader.register((parser) => new VRMLoaderPlugin(parser))
    loader.register((parser) => new VRMAnimationLoaderPlugin(parser))

    try {
      const gltf = await loader.loadAsync(modelPath)
      const vrm = gltf.userData.vrm as VRM

      if (!vrm) {
        throw new Error('Failed to load VRM from model')
      }

      // 性能优化
      VRMUtils.removeUnnecessaryVertices(gltf.scene)
      VRMUtils.combineSkeletons(gltf.scene)
      VRMUtils.combineMorphs(vrm)

      // 禁用视锥体剔除和阴影
      vrm.scene.traverse((obj) => {
        obj.frustumCulled = false
        if (obj instanceof THREE.Mesh) {
          obj.castShadow = false
          obj.receiveShadow = false
        }
      })

      // 调整VRM0.0的旋转
      VRMUtils.rotateVRM0(vrm)

      // 调整模型位置和缩放 - 向右平移并轻微向下偏移以突出上半身
      vrm.scene.position.set(0.15, -0.3, 0)  // 向右平移0.3单位，轻微向下偏移，显示上半身为主
      vrm.scene.rotation.set(0, 0, 0)

      this.vrm = vrm
      this.scene.add(vrm.scene)

      // 创建 VRMLookAtQuaternionProxy 来支持视线动画
      if (vrm.lookAt) {
        const lookAtQuatProxy = new VRMLookAtQuaternionProxy(vrm.lookAt)
        lookAtQuatProxy.name = 'lookAtQuaternionProxy'
        vrm.scene.add(lookAtQuatProxy)
      }

      // 初始化动画混合器
      this.mixer = new THREE.AnimationMixer(vrm.scene)

      console.log('VRM model loaded successfully:', vrm)
    } catch (error) {
      console.error('Error loading VRM model:', error)
      throw error
    }
  }

  async loadIdleAnimation(animationPath: string) {
    if (!this.vrm || !this.mixer) {
      console.warn('VRM model must be loaded before loading animations')
      return
    }

    const loader = new GLTFLoader()
    loader.register((parser) => new VRMAnimationLoaderPlugin(parser))

    try {
      const gltf = await loader.loadAsync(animationPath)
      const vrmAnimations = gltf.userData.vrmAnimations

      if (vrmAnimations && vrmAnimations.length > 0) {
        const animationClip = createVRMAnimationClip(vrmAnimations[0], this.vrm)
        this.idleClip = animationClip
        
        // 播放闲置动画
        this.playIdleAnimation()
        
        console.log('Idle animation loaded and playing')
      }
    } catch (error) {
      console.error('Error loading idle animation:', error)
    }
  }

  /**
   * 播放闲置动画
   */
  playIdleAnimation() {
    if (!this.idleClip || !this.mixer) return

    const action = this.mixer.clipAction(this.idleClip)
    action.reset()
    action.setLoop(THREE.LoopRepeat, Infinity)
    action.play()
    
    console.log('Idle animation started')
  }

  /**
   * 停止闲置动画
   */
  stopIdleAnimation() {
    if (!this.idleClip || !this.mixer) return

    const action = this.mixer.clipAction(this.idleClip)
    action.stop()
    
    console.log('Idle animation stopped')
  }

  /**
   * 暂停/恢复闲置动画
   */
  toggleIdleAnimation() {
    if (!this.idleClip || !this.mixer) return

    const action = this.mixer.clipAction(this.idleClip)
    if (action.isRunning()) {
      action.paused = !action.paused
      console.log('Idle animation', action.paused ? 'paused' : 'resumed')
    } else {
      this.playIdleAnimation()
    }
  }

  setEmotion(emotion: string) {
    if (!this.vrm?.expressionManager) return

    this.currentEmotion = emotion

    // 重置所有表情
    const expressionManager = this.vrm.expressionManager
    const presetExpressions = ['happy', 'angry', 'sad', 'relaxed', 'surprised']
    presetExpressions.forEach(exp => {
      expressionManager.setValue(exp, 0)
    })

    // 设置目标表情
    switch (emotion) {
      case 'happy':
      case '高兴':
        expressionManager.setValue('happy', 1.0)
        break
      case 'sad':
      case '难过':
        expressionManager.setValue('sad', 1.0)
        break
      case 'surprised':
      case '惊讶':
        expressionManager.setValue('surprised', 1.0)
        break
      case 'neutral':
      case '平静':
        expressionManager.setValue('relaxed', 0.3)
        break
      default:
        console.warn('Unknown emotion:', emotion)
    }

    console.log('Emotion set to:', emotion)
  }

  async testLipSync() {
    if (!this.vrm?.expressionManager) return

    this.isLipSyncActive = true
    this.lipSyncAnimation()
  }

  async startRealTimeLipSync() {
    if (!this.audioAnalyzer) {
      this.audioAnalyzer = new AudioAnalyzer()
      await this.audioAnalyzer.init()
    }

    this.isRealTimeLipSync = true
    
    const success = await this.audioAnalyzer.startMicrophoneAnalysis((volume: number, frequencies: number[]) => {
      if (!this.isRealTimeLipSync || !this.vrm?.expressionManager) return

      const weights = AudioAnalyzer.calculateLipSyncWeights(volume, frequencies)
      const expressionManager = this.vrm.expressionManager

      // 应用口型权重 - 使用映射表支持多种命名，调整权重减小张嘴幅度
      Object.entries(weights).forEach(([baseShape, weight]) => {
        const possibleNames = this.lipSyncMapping[baseShape] || [baseShape]
        for (const name of possibleNames) {
          if (expressionManager.expressionMap[name]) {
            // 将权重乘以0.6来减小嘴巴张开幅度
            expressionManager.setValue(name, (weight as number) * 0.6)
            break
          }
        }
      })
    })

    if (!success) {
      console.warn('Failed to start real-time lip sync')
      this.isRealTimeLipSync = false
    }
  }

  async playAudioFile(audioUrl: string) {
    if (!this.audioAnalyzer) {
      this.audioAnalyzer = new AudioAnalyzer()
      await this.audioAnalyzer.init()
    }

    // 只停止当前音频播放，但不停止分析器
    if (this.audioAnalyzer) {
      this.audioAnalyzer.stopCurrentAudio()
    }
    
    // 停止其他类型的口型同步
    this.isLipSyncActive = false
    this.isRealTimeLipSync = false
    
    console.log('Starting audio file playback and lip sync:', audioUrl)
    
    // 检查VRM模型和表情管理器是否可用
    if (!this.vrm?.expressionManager) {
      console.error('VRM model or expression manager not available')
      return false
    }
    
    // 打印可用的表情
    const availableExpressions = this.vrm.expressionManager.expressionMap
    console.log('Available expressions:', Object.keys(availableExpressions))
    
    const success = await this.audioAnalyzer.loadAndAnalyzeAudioFile(audioUrl, (volume: number, frequencies: number[]) => {
      if (!this.vrm?.expressionManager) return

      const weights = AudioAnalyzer.calculateLipSyncWeights(volume, frequencies)
      const expressionManager = this.vrm.expressionManager

      // 应用口型权重 - 使用映射表支持多种命名，调整权重减小张嘴幅度
      Object.entries(weights).forEach(([baseShape, weight]) => {
        if ((weight as number) > 0) {
          const possibleNames = this.lipSyncMapping[baseShape] || [baseShape]
          for (const name of possibleNames) {
            if (expressionManager.expressionMap[name]) {
              // 将权重乘以0.6来减小嘴巴张开幅度
              const adjustedWeight = (weight as number) * 0.6
              expressionManager.setValue(name, adjustedWeight)
              console.log(`Setting ${name} to ${adjustedWeight.toFixed(3)}`)
              break
            }
          }
        }
      })
    })

    if (!success) {
      console.warn('Failed to play audio file')
    }
    
    return success
  }

  /**
   * 播放二进制音频数据并同步口型
   * @param audioData 音频的二进制数据 (ArrayBuffer)
   * @param mimeType 音频的MIME类型，例如 'audio/wav', 'audio/mp3' 等（此参数仅用于日志记录）
   * @returns 是否成功开始播放
   */
  async playAudio(audioData: ArrayBuffer, mimeType: string = 'audio/wav') {
    if (!this.audioAnalyzer) {
      this.audioAnalyzer = new AudioAnalyzer()
      await this.audioAnalyzer.init()
    }

    // 停止其他类型的口型同步
    this.isLipSyncActive = false
    this.isRealTimeLipSync = false
    
    console.log('Starting audio binary data playback and lip sync, type:', mimeType)
    
    // 检查VRM模型和表情管理器是否可用
    if (!this.vrm?.expressionManager) {
      console.error('VRM model or expression manager not available')
      return false
    }
    
    // 打印可用的表情
    const availableExpressions = this.vrm.expressionManager.expressionMap
    console.log('Available expressions:', Object.keys(availableExpressions))
    
    const success = await this.audioAnalyzer.loadAndAnalyzeAudioBuffer(audioData, (volume: number, frequencies: number[]) => {
      if (!this.vrm?.expressionManager) return

      const weights = AudioAnalyzer.calculateLipSyncWeights(volume, frequencies)
      const expressionManager = this.vrm.expressionManager

      // 应用口型权重 - 使用映射表支持多种命名，调整权重减小张嘴幅度
      Object.entries(weights).forEach(([baseShape, weight]) => {
        if (weight > 0) {
          const possibleNames = this.lipSyncMapping[baseShape] || [baseShape]
          for (const name of possibleNames) {
            if (expressionManager.expressionMap[name]) {
              // 将权重乘以0.6来减小嘴巴张开幅度
              expressionManager.setValue(name, (weight as number) * 0.6)
              break
            }
          }
        }
      })
    })

    if (!success) {
      console.warn('Failed to play audio binary data')
    }
    
    return success
  }

  /**
   * 处理文本输入，分析情感并执行相应的表情和动作
   * @param text 输入的文本
   */
  processText(text: string) {
    if (!this.vrm) return

    console.log('Processing text:', text)

    // 分析文本情感
    const emotionResult = EmotionAnalyzer.analyzeEmotion(text)
    console.log('Emotion analysis result:', emotionResult)

    // 设置表情
    const vrmExpression = EmotionAnalyzer.getVRMExpression(emotionResult.dominantEmotion)
    this.setEmotion(vrmExpression)

    // 建议动作
    const suggestedGesture = EmotionAnalyzer.suggestGesture(text, emotionResult)
    if (suggestedGesture) {
      // 延迟执行动作，让表情先显示
      setTimeout(() => {
        this.playGesture(suggestedGesture)
      }, 500)
    }

    // 保存当前情感状态
    this.currentEmotion = emotionResult.dominantEmotion

    return emotionResult
  }

  /**
   * 停止口型同步
   */
  stopLipSync() {
    this.isLipSyncActive = false
    this.isRealTimeLipSync = false
    
    if (this.audioAnalyzer) {
      this.audioAnalyzer.stopCurrentAudio()
      this.audioAnalyzer.stopAnalysis()
    }

    if (this.vrm?.expressionManager) {
      // 重置口型 - 使用映射表支持多种命名
      this.lipSyncBlendShapes.forEach(baseShape => {
        const possibleNames = this.lipSyncMapping[baseShape] || [baseShape]
        for (const name of possibleNames) {
          if (this.vrm!.expressionManager!.expressionMap[name]) {
            this.vrm!.expressionManager!.setValue(name, 0)
            break
          }
        }
      })
    }
  }

  /**
   * 获取当前情感状态
   */
  getCurrentEmotion(): string {
    return this.currentEmotion
  }

  private lipSyncAnimation() {
    if (!this.isLipSyncActive || !this.vrm?.expressionManager) return

    const time = this.clock.elapsedTime
    const expressionManager = this.vrm.expressionManager

    // 获取所有可用的表情名称（仅第一次）
    if (!this.lipSyncBlendShapes.length) {
      console.log('Available expressions:', Object.keys(expressionManager.expressionMap))
    }

    // 重置所有口型 - 使用映射表支持多种命名
    this.lipSyncBlendShapes.forEach(baseShape => {
      const possibleNames = this.lipSyncMapping[baseShape] || [baseShape]
      for (const name of possibleNames) {
        if (expressionManager.expressionMap[name]) {
          expressionManager.setValue(name, 0)
          break
        }
      }
    })

    // 模拟说话的口型变化
    const frequency = 5 // 说话频率
    const amplitude = 0.8 // 强度
    const phase = Math.sin(time * frequency) * amplitude

    let targetShape = 'a'
    let weight = 0

    if (phase > 0.6) {
      targetShape = 'a'
      weight = phase
    } else if (phase > 0.2) {
      targetShape = 'i'
      weight = phase
    } else if (phase > -0.2) {
      targetShape = 'u'
      weight = Math.abs(phase)
    } else if (phase > -0.6) {
      targetShape = 'e'
      weight = Math.abs(phase)
    } else {
      targetShape = 'o'
      weight = Math.abs(phase)
    }

    // 应用目标口型 - 使用映射表，调整权重减小张嘴幅度
    const possibleNames = this.lipSyncMapping[targetShape] || [targetShape]
    for (const name of possibleNames) {
      if (expressionManager.expressionMap[name]) {
        // 将权重乘以0.6来减小嘴巴张开幅度
        const adjustedWeight = weight * 0.6
        expressionManager.setValue(name, adjustedWeight)
        console.log(`Setting ${name} to ${adjustedWeight.toFixed(3)}`)
        break
      }
    }

    // 继续动画
    requestAnimationFrame(() => this.lipSyncAnimation())
  }

  playGesture(gesture: string) {
    if (!this.vrm?.humanoid) return

    const humanoid = this.vrm.humanoid

    switch (gesture) {
      case 'nod':
      case '点头':
        this.animateNod(humanoid)
        break
      default:
        console.warn('Unknown gesture:', gesture)
    }
  }

  private animateNod(humanoid: any) {
    try {
      const head = humanoid.getNormalizedBoneNode('head')
      const neck = humanoid.getNormalizedBoneNode('neck')
      
      if (head) {
        // 保存原始旋转
        const originalHeadRotation = {
          x: head.rotation.x,
          y: head.rotation.y,
          z: head.rotation.z
        }
        const originalNeckRotation = neck ? {
          x: neck.rotation.x,
          y: neck.rotation.y,
          z: neck.rotation.z
        } : null

        const startTime = this.clock.elapsedTime
        const duration = 1.5
        const nodCount = 2 // 点头次数

        const animate = () => {
          const elapsed = this.clock.elapsedTime - startTime
          const progress = Math.min(elapsed / duration, 1)
          
          if (progress < 1) {
            // 使用更自然的点头曲线
            const nodPhase = progress * Math.PI * 2 * nodCount
            const intensity = Math.sin(Math.PI * progress) // 开始和结束时较慢
            const nodAngle = Math.sin(nodPhase) * 0.4 * intensity
            
            // 主要点头动作在头部
            head.rotation.x = originalHeadRotation.x + nodAngle
            
            // 颈部也有轻微参与
            if (neck && originalNeckRotation) {
              neck.rotation.x = originalNeckRotation.x + nodAngle * 0.3
            }
            
            requestAnimationFrame(animate)
          } else {
            // 重置到原始位置
            head.rotation.x = originalHeadRotation.x
            head.rotation.y = originalHeadRotation.y
            head.rotation.z = originalHeadRotation.z
            
            if (neck && originalNeckRotation) {
              neck.rotation.x = originalNeckRotation.x
              neck.rotation.y = originalNeckRotation.y
              neck.rotation.z = originalNeckRotation.z
            }
          }
        }
        
        animate()
      }
    } catch (error) {
      console.warn('Nod animation failed:', error)
    }
  }

  resetPose() {
    if (!this.vrm?.humanoid) return

    const humanoid = this.vrm.humanoid
    
    // 使用正确的骨骼名称枚举
    const boneNames = [
      'head', 'neck',
      'leftUpperArm', 'leftLowerArm', 'leftHand',
      'rightUpperArm', 'rightLowerArm', 'rightHand',
      'spine', 'chest', 'upperChest'
    ] as const

    boneNames.forEach(boneName => {
      try {
        const bone = humanoid.getNormalizedBoneNode(boneName)
        if (bone) {
          bone.rotation.set(0, 0, 0)
        }
      } catch (error) {
        // 忽略不存在的骨骼
        console.warn(`Bone ${boneName} not found`)
      }
    })

    // 重置表情
    this.setEmotion('neutral')
    this.stopLipSync()

    // 重新播放闲置动画
    if (this.idleClip) {
      this.playIdleAnimation()
    }

    console.log('Pose reset to neutral')
  }

  private startRenderLoop() {
    const animate = () => {
      this.animationId = requestAnimationFrame(animate)

      const deltaTime = this.clock.getDelta()

      // 更新VRM
      if (this.vrm) {
        this.vrm.update(deltaTime)
      }

      // 更新动画混合器
      if (this.mixer) {
        this.mixer.update(deltaTime)
      }

      // 渲染场景
      this.renderer.render(this.scene, this.camera)
    }

    animate()
  }

  private handleResize(canvas: HTMLCanvasElement) {
    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect
        
        this.camera.aspect = width / height
        this.camera.updateProjectionMatrix()
        this.renderer.setSize(width, height)
      }
    })

    resizeObserver.observe(canvas)
  }

  dispose() {
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId)
    }

    this.stopLipSync()

    if (this.audioAnalyzer) {
      this.audioAnalyzer.dispose()
      this.audioAnalyzer = null
    }

    if (this.vrm) {
      VRMUtils.deepDispose(this.vrm.scene)
    }

    if (this.mixer) {
      this.mixer.stopAllAction()
    }

    this.renderer.dispose()
  }
}
