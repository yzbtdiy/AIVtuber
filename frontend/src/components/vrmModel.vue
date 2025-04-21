<script setup lang="ts">
import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { VRMLoaderPlugin, VRMUtils } from '@pixiv/three-vrm';
import { createVRMAnimationClip, VRMAnimationLoaderPlugin, VRMLookAtQuaternionProxy } from '@pixiv/three-vrm-animation';
import { onMounted, onUnmounted, ref } from 'vue';
import { Events } from "@wailsio/runtime";


// 用于存储当前嘴部张开度的 Ref，由音频处理更新
const mouthOpenness = ref(0.0);
// 用于平滑处理的目标嘴部张开度 Ref
const targetMouthOpenness = ref(0.0);
// 用于存储加载后的 VRM 实例的 Ref
const vrmInstance = ref<any>(null);
// AudioContext 的 Ref
const audioContext = ref<AudioContext | null>(null);
// 用于播放期间音频分析的 Refs
const analyserNode = ref<AnalyserNode | null>(null);
const analyserDataArray = ref<Uint8Array | null>(null);
const isPlayingAudio = ref(false); // 标记播放状态的标志

// 处理从后端接收的 base64 WAV 数据的函数
async function processAudioChunk(base64Wav: string) {
    if (!audioContext.value) {
        console.error("AudioContext 未初始化");
        return;
    }
    // 确保 context 正在运行
    if (audioContext.value.state === 'suspended') {
        await audioContext.value.resume();
    }
    if (audioContext.value.state !== 'running') {
        console.error("AudioContext 未运行。");
        return;
    }

    try {
        // 1. 将 Base64 解码为 ArrayBuffer
        const binaryString = window.atob(base64Wav);
        const len = binaryString.length;
        const bytes = new Uint8Array(len);
        for (let i = 0; i < len; i++) {
            bytes[i] = binaryString.charCodeAt(i);
        }
        const audioData = bytes.buffer;

        // 2. 使用 Web Audio API 解码 WAV 数据
        const audioBuffer = await audioContext.value.decodeAudioData(audioData);

        // 3. 如果尚未完成，则设置 Analyser
        if (!analyserNode.value) {
            analyserNode.value = audioContext.value.createAnalyser();
            analyserNode.value.fftSize = 256; // 根据需要调整
            const bufferLength = analyserNode.value.frequencyBinCount;
            analyserDataArray.value = new Uint8Array(bufferLength);
            console.log("AnalyserNode 已创建。");
        }

        // 4. 创建并连接用于播放和分析的源节点
        const source = audioContext.value.createBufferSource();
        source.buffer = audioBuffer;
        // 连接 source -> analyser -> destination
        source.connect(analyserNode.value);
        analyserNode.value.connect(audioContext.value.destination); // 将 analyser 连接到输出

        // 5. 处理播放开始和结束
        isPlayingAudio.value = true;
        source.onended = () => {
            console.log("音频播放结束。");
            isPlayingAudio.value = false;
            mouthOpenness.value = 0.0; // 音频结束时重置嘴部
            source.disconnect(); // 播放后断开源
            // 如果不立即重用，可以选择断开 analyser
            // analyserNode.value?.disconnect();
        };

        // 立即开始播放
        source.start(0);
        console.log("音频播放开始，分析已激活。");

    } catch (error) {
        console.error("处理或播放音频块时出错:", error); // 更新了错误消息
        mouthOpenness.value = 0.0;
        isPlayingAudio.value = false; // 确保在出错时重置标志
    }
}

// 事件处理函数 - 改为异步
async function handleChatAnswer(answer: any) {
    try {
        // 尝试从事件数据中提取音频 base64 字符串
        const audioBase64 = answer?.data?.[0]?.choices?.[0]?.Message?.audio?.data;
        // 提取文本内容（如果需要）
        const contentString = answer?.data?.[0]?.choices?.[0]?.Message?.content;

        console.log("收到 CHAT:ANSWER 事件，内容:", contentString); // 记录收到的文本

        if (audioBase64 && typeof audioBase64 === 'string') {
            console.log("正在处理来自事件的音频数据...");
            // 调用 processAudioChunk 处理音频，并等待其完成
            await processAudioChunk(audioBase64);
        } else {
            console.warn("在 CHAT:ANSWER 事件中未找到有效的音频数据。");
        }
    } catch (error) {
        console.error("处理 CHAT:ANSWER 事件时发生错误:", error);
    }
}

onMounted(async () => {
    // 在 onMounted 开始时注册事件监听器
    Events.On("CHAT:ANSWER", handleChatAnswer);
    console.log("CHAT:ANSWER 事件监听器已注册。");

    // 初始化 AudioContext
    try {
        audioContext.value = new AudioContext();
        // 不再尝试在 onMounted 中立即恢复，推迟到实际需要播放时
        console.log("AudioContext 已初始化，状态：", audioContext.value.state);
    } catch (e) {
        console.error("创建 AudioContext 失败", e);
        // AudioContext 创建失败不应阻止模型加载
    }

    const canvas = document.getElementById('vrmModel') as HTMLCanvasElement;
    if (!canvas) {
        console.error("未找到 ID 为 'vrmModel' 的 Canvas 元素。");
        return;
    }

    // 渲染器
    const renderer = new THREE.WebGLRenderer({ canvas: canvas, alpha: true });
    renderer.setSize(canvas.clientWidth, canvas.clientHeight);
    renderer.setPixelRatio(window.devicePixelRatio);

    // 相机
    const camera = new THREE.PerspectiveCamera(30.0, canvas.clientWidth / canvas.clientHeight, 0.1, 20.0);

    // 相机控制器
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.screenSpacePanning = true;

    // 场景
    const scene = new THREE.Scene();

    // 光照
    const light = new THREE.DirectionalLight(0xffffff, Math.PI);
    light.position.set(1.0, 1.0, 1.0).normalize();
    scene.add(light);

    // gltf 加载器
    const loader = new GLTFLoader();
    loader.crossOrigin = 'anonymous';

    loader.register((parser) => {
        return new VRMLoaderPlugin(parser);
    });

    loader.register((parser) => {
        return new VRMAnimationLoaderPlugin(parser);
    });

    // 加载 VRM 模型
    try {
        const gltfVrm = await loader.loadAsync('/vroid.vrm');
        const vrm = gltfVrm.userData.vrm;
        vrmInstance.value = vrm; // 存储 VRM 实例

        // VRM 工具处理
        VRMUtils.removeUnnecessaryVertices(vrm.scene);
        VRMUtils.combineSkeletons(vrm.scene);
        VRMUtils.combineMorphs(vrm);

        vrm.scene.traverse((obj: THREE.Object3D) => {
            obj.frustumCulled = false; // 关闭视锥体剔除
        });

        // LookAt 代理
        const lookAtQuatProxy = new VRMLookAtQuaternionProxy(vrm.lookAt);
        lookAtQuatProxy.name = 'lookAtQuaternionProxy';
        vrm.scene.add(lookAtQuatProxy);

        // 添加 VRM 到场景并应用水平和垂直平移
        console.log(vrm);
        const modelShiftX = -0.16; // 保持水平偏移
        const modelShiftY = 0.02; // 向上平移的值
        vrm.scene.position.set(modelShiftX, modelShiftY, 0); // 设置模型位置（带 Y 轴偏移）
        scene.add(vrm.scene);

        // 移动模型后计算包围盒并调整相机
        const boundingBox = new THREE.Box3().setFromObject(vrm.scene);
        const modelSize = new THREE.Vector3();
        boundingBox.getSize(modelSize);
        const modelCenter = new THREE.Vector3();
        boundingBox.getCenter(modelCenter); // modelCenter 现在反映了偏移后的位置

        // 根据新的中心点调整相机控制器的目标点（例如，头部水平）
        const targetYOffset = modelSize.y * 0.3;
        controls.target.set(modelCenter.x, modelCenter.y + targetYOffset, modelCenter.z);

        // 调整相机位置以框住模型
        const maxDim = Math.max(modelSize.x, modelSize.y, modelSize.z);
        const fov = camera.fov * (Math.PI / 180);
        const distanceToFit = (maxDim / 2) / Math.tan(fov / 2);
        const zoomFactor = 0.4; // 保持缩放因子
        const cameraZ = distanceToFit * zoomFactor;

        const cameraYOffset = modelSize.y * 0.01; // 保持 Y 轴偏移以获得水平视角
        const cameraXOffset = modelSize.x * 0.08; // 小的正 X 轴偏移以获得轻微向左旋转的视角（根据需要调整）
        camera.position.set(controls.target.x + cameraXOffset, controls.target.y + cameraYOffset, controls.target.z + cameraZ);

        controls.update(); // 更新控制器状态

        // 初始化时确保相机宽高比正确
        camera.aspect = canvas.clientWidth / canvas.clientHeight;
        camera.updateProjectionMatrix();

        // 加载 VRMA 动画
        const gltfVrma = await loader.loadAsync('/idle_loop.vrma');
        const vrmAnimation = gltfVrma.userData.vrmAnimations[0];

        // 创建动画剪辑
        const clip = createVRMAnimationClip(vrmAnimation, vrm);

        // 播放动画
        const mixer = new THREE.AnimationMixer(vrm.scene);
        mixer.clipAction(clip).play();

        // 动画循环
        const clock = new THREE.Clock();
        clock.start();

        function animate() {
            requestAnimationFrame(animate);
            const deltaTime = clock.getDelta();

            // --- 唇形同步更新（播放期间实时，带平滑处理） ---
            if (isPlayingAudio.value && analyserNode.value && analyserDataArray.value && vrmInstance.value?.expressionManager) {
                // 改为获取时域数据 (波形)
                analyserNode.value.getByteTimeDomainData(analyserDataArray.value);

                // 计算当前帧的 RMS 音量
                let sumSquares = 0.0;
                for (const amplitude of analyserDataArray.value) {
                    // 将 0-255 的值转换为 -1.0 到 1.0 的范围 (近似)
                    const normalizedAmplitude = (amplitude / 128.0) - 1.0;
                    sumSquares += normalizedAmplitude * normalizedAmplitude;
                }
                const rms = Math.sqrt(sumSquares / analyserDataArray.value.length);

                // 将 RMS 音量映射到目标 blend shape 值
                // RMS 值通常在 0 到 1 之间，但有效范围可能更小 (例如 0 到 0.5)
                // 需要根据实际音频调整阈值和缩放因子
                const rmsThreshold = 0.02; // 忽略非常低的 RMS 值
                const rmsScaleFactor = 8.0; // 放大 RMS 值以达到 0-1 范围 (需要调整)
                let calculatedOpenness = 0.0;
                if (rms > rmsThreshold) {
                    calculatedOpenness = Math.min(0.8, Math.max(0.0, (rms - rmsThreshold) * rmsScaleFactor));
                }
                targetMouthOpenness.value = calculatedOpenness;

            } else {
                // 未播放时目标为闭嘴
                targetMouthOpenness.value = 0.0;
            }

            // 将当前嘴部张开度平滑地插值到目标值
            // 可以尝试不同的平滑因子，例如 0.4 或 0.6
            const lerpFactor = 0.4; // 调整平滑因子
            mouthOpenness.value = THREE.MathUtils.lerp(mouthOpenness.value, targetMouthOpenness.value, lerpFactor);

            // 将平滑后的值应用于 VRM 模型
            if (vrmInstance.value?.expressionManager) {
                // 确保表情名称 'aa' 与您的 VRM 模型定义匹配
                vrmInstance.value.expressionManager.setValue('aa', mouthOpenness.value);
            }
            // --- 唇形同步更新结束 ---

            if (mixer) mixer.update(deltaTime); // 更新动画混合器
            if (vrmInstance.value) vrmInstance.value.update(deltaTime); // 更新 VRM 组件（应基于管理器值更新表情）

            renderer.render(scene, camera); // 渲染场景
        }

        animate();

        // 处理窗口大小调整（或 Canvas 大小调整，如果需要）
        const resizeObserver = new ResizeObserver(entries => {
            for (let entry of entries) {
                const { width, height } = entry.contentRect;
                camera.aspect = width / height;
                camera.updateProjectionMatrix();
                renderer.setSize(width, height);
            }
        });
        resizeObserver.observe(canvas);

    } catch (error) {
        console.error("加载 VRM 模型或设置场景/动画时出错:", error);
        return;
    }
}); // onMounted 结束

onUnmounted(() => {
    // 在 onUnmounted 时注销事件监听器
    Events.Off("CHAT:ANSWER");
    console.log("CHAT:ANSWER 事件监听器已注销。");

    // 清理 AudioContext 和 Analyser
    if (analyserNode.value) {
        analyserNode.value.disconnect();
    }
    if (audioContext.value && audioContext.value.state !== 'closed') {
        audioContext.value.close();
        console.log("AudioContext 已关闭。");
    }
    console.log("组件已卸载。");
});

</script>

<template>
    <div id="vrmContainer">
        <canvas id="vrmModel"></canvas>
    </div>
</template>

<style scoped>
#vrmContainer {
    position: absolute;
    bottom: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 30vw;
    height: 80vh;
    overflow: hidden;
}

#vrmModel {
    width: 100%;
    height: 100%;
    display: block;
    margin: 0;
    padding: 0;
}
</style>
