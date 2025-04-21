<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'; // 导入 onUnmounted
import { GetStatus } from "../../bindings/github.com/yzbtdiy/AIVtuber/services/bilibililiveservice";

// 使用 ref 存储状态，并提供初始值
const isStart = ref(false);
const connected = ref(false);
let intervalId: number | null = null; // 用于存储定时器 ID

// 提取状态更新逻辑到一个函数
async function updateStatus() {
    try {
        // 使用数组解构接收 GetStatus 返回的两个布尔值
        const [startStatus, connectedStatus] = await GetStatus();
        isStart.value = startStatus;
        connected.value = connectedStatus;
        // 更新日志以反映接收到的值
        console.log(`直播状态已更新: isStart=${isStart.value}, connected=${connected.value}`);
    } catch (error) {
        console.error("更新直播状态失败:", error);
        // 可以选择在出错时重置状态
        isStart.value = false;
        connected.value = false;
    }
}

// 在组件挂载后获取初始状态并设置定时器
onMounted(async () => {
    await updateStatus(); // 立即获取一次初始状态

    // 设置定时器，每 20 秒更新一次状态
    intervalId = window.setInterval(updateStatus, 20000); // setInterval 返回 number 类型
});

// 在组件卸载前清除定时器
onUnmounted(() => {
    if (intervalId !== null) {
        clearInterval(intervalId);
        console.log("直播状态更新定时器已清除。");
    }
});

// 计算属性，根据状态决定样式和文本
const statusClass = computed(() => {
    return isStart.value && connected.value ? 'status-connected' : 'status-disconnected';
});

const statusText = computed(() => {
    return isStart.value && connected.value ? '直播接入' : '直播断开';
});

</script>

<template>
    <!-- 临时调试信息 -->
    <!-- <div>Debug: isStart={{ isStart }}, connected={{ connected }}, class={{ statusClass }}</div> -->
    <div class="live-status-indicator" :class="statusClass">
        {{ statusText }}
    </div>
</template>

<style scoped>
.live-status-indicator {
    position: fixed; /* 固定定位 */
    bottom: 20px;    /* 距离底部 20px */
    left: 20px;     /* 距离左侧 20px */
    padding: 4px 10px; /* 内边距 */
    border-radius: 20px; /* 圆角 */
    color: white;      /* 文字颜色 */
    font-size: 15px;   /* 字体大小 */
    text-align: center; /* 文字居中 */
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2); /* 添加阴影增加可见性 */
    transition: background-color 0.3s ease; /* 背景色过渡效果 */
    z-index: 1000; /* 确保在顶层 */
}

.status-connected {
    background-color: #4CAF50; /* 绿色背景 */
}

.status-disconnected {
    background-color: #F44336; /* 红色背景 */
}
</style>
