<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import type { DanmakuMessage } from '../types/bilibililive.ts';

// Props 接收数据
interface Props {
  danmus?: DanmakuMessage[];
  connectionStats?: {
    danmuCount: number;
    giftCount: number;
  };
  isConnected?: boolean;
  danmuConfig?: {
    enabled: boolean;
    speed: number;
    fontSize: number;
    opacity: number;
    maxRows: number;
    minInterval: number;
  };
}

const props = withDefaults(defineProps<Props>(), {
  danmus: () => [],
  connectionStats: () => ({ danmuCount: 0, giftCount: 0 }),
  isConnected: false,
  danmuConfig: () => ({
    enabled: true,
    speed: 50,
    fontSize: 16,
    opacity: 0.9,
    maxRows: 10,
    minInterval: 500
  })
});

// Events 发送事件
const emit = defineEmits<{
  clearMessages: [];
  updateActiveCount: [count: number];
}>();

// 弹幕容器引用
const danmuContainerRef = ref<HTMLDivElement>();

// 活跃弹幕列表
interface ActiveDanmu {
  id: string;
  message: DanmakuMessage;
  element?: HTMLElement;
  row: number;
  startTime: number;
  isActive: boolean;
}

const activeDanmus = ref<ActiveDanmu[]>([]);
const rowUsage = ref<Array<{ lastEndTime: number }>>([]);

// 初始化行使用情况
const initializeRows = () => {
  rowUsage.value = Array.from({ length: props.danmuConfig?.maxRows || 10 }, () => ({
    lastEndTime: 0
  }));
};

// 获取可用行
const getAvailableRow = (): number => {
  const now = Date.now();
  const minInterval = props.danmuConfig?.minInterval || 1000; // 增加到1秒间隔
  
  for (let i = 0; i < rowUsage.value.length; i++) {
    const timeSinceLastUse = now - rowUsage.value[i].lastEndTime;
    if (timeSinceLastUse >= minInterval) {
      console.log(`行 ${i} 可用，上次使用时间: ${rowUsage.value[i].lastEndTime}, 当前时间: ${now}, 间隔: ${timeSinceLastUse}ms`);
      return i;
    }
  }
  
  console.log('所有行都被占用，寻找最早结束的行');
  // 找到最早结束的行
  let earliestRowIndex = 0;
  let earliestTime = rowUsage.value[0].lastEndTime;
  
  for (let i = 1; i < rowUsage.value.length; i++) {
    if (rowUsage.value[i].lastEndTime < earliestTime) {
      earliestTime = rowUsage.value[i].lastEndTime;
      earliestRowIndex = i;
    }
  }
  
  console.log(`使用最早结束的行: ${earliestRowIndex}`);
  return earliestRowIndex;
};

// 创建弹幕元素
const createDanmuElement = (danmu: DanmakuMessage, row: number): HTMLElement => {
  const element = document.createElement('div');
  element.className = 'floating-danmu';
  
  // 调整起始位置，避免被状态栏遮挡（状态栏高度60px）
  const topPosition = 80 + row * 40; // 从80px开始，给状态栏留出空间
  
  element.style.cssText = `
    position: fixed;
    top: ${topPosition}px;
    right: -50px;
    z-index: 9999;
    white-space: nowrap;
    pointer-events: none;
    font-size: ${props.danmuConfig?.fontSize || 16}px;
    opacity: ${props.danmuConfig?.opacity || 0.9};
    color: #ffffff;
    text-shadow: 2px 2px 4px rgba(0,0,0,0.8);
    font-weight: bold;
    padding: 6px 12px;
    background: rgba(0,0,0,0.7);
    border-radius: 16px;
    border: 2px solid rgba(255,255,255,0.3);
    font-family: Arial, sans-serif;
  `;
  
  // 创建弹幕内容
  const content = `${danmu.uname}: ${danmu.msg}`;
  element.textContent = content;
  
  console.log('创建弹幕元素:', {
    content,
    top: topPosition,
    row,
    fontSize: props.danmuConfig?.fontSize || 16,
    opacity: props.danmuConfig?.opacity || 0.9
  });
  
  return element;
};

// 启动弹幕动画
const startDanmuAnimation = (activeDanmu: ActiveDanmu) => {
  if (!activeDanmu.element) return;
  
  const element = activeDanmu.element;
  const containerWidth = window.innerWidth;
  const elementWidth = element.offsetWidth || 200; // 默认宽度防止0
  
  // 根据配置动态计算动画时长
  const baseSpeed = props.danmuConfig?.speed || 50; // 基础速度 1-100
  const speedMultiplier = baseSpeed / 50; // 速度倍数，50为基准
  const baseDuration = 10000; // 基础10秒
  const duration = baseDuration / speedMultiplier; // 速度越快，时长越短
  
  console.log('启动弹幕动画:', {
    elementWidth,
    containerWidth,
    baseSpeed,
    speedMultiplier,
    duration,
    elementStyle: element.style.cssText
  });
  
  // 设置动画起始位置和动画
  element.style.right = '-50px';
  element.style.animation = `danmuMove ${duration}ms linear forwards`;
  
  // 动画结束后清理
  setTimeout(() => {
    if (activeDanmu.element && activeDanmu.element.parentNode) {
      activeDanmu.element.parentNode.removeChild(activeDanmu.element);
      console.log('弹幕元素已移除');
    }
    activeDanmu.isActive = false;
    
    // 清理已完成的弹幕
    const index = activeDanmus.value.findIndex(d => d.id === activeDanmu.id);
    if (index !== -1) {
      activeDanmus.value.splice(index, 1);
    }
    
    // 更新活跃弹幕数量
    emit('updateActiveCount', activeDanmus.value.length);
  }, duration + 100);
};

// 添加新弹幕
const addDanmu = (danmu: DanmakuMessage) => {
  console.log('addDanmu called:', danmu);
  console.log('danmuConfig.enabled:', props.danmuConfig?.enabled);
  
  if (!props.danmuConfig?.enabled) {
    console.log('弹幕已禁用，跳过');
    return;
  }
  
  const row = getAvailableRow();
  console.log('可用行:', row);
  if (row === -1) {
    console.log('没有可用行，跳过');
    return; // 没有可用行
  }
  
  const now = Date.now();
  const activeDanmu: ActiveDanmu = {
    id: `${danmu.open_id}-${danmu.timestamp}-${Math.random()}`,
    message: danmu,
    row,
    startTime: now,
    isActive: true
  };
  
  // 创建并添加弹幕元素
  const element = createDanmuElement(danmu, row);
  document.body.appendChild(element);
  activeDanmu.element = element;
  
  console.log('弹幕元素已创建并添加到DOM:', element);
  
  // 更新行使用情况 - 根据动画速度动态计算
  const baseSpeed = props.danmuConfig?.speed || 50;
  const speedMultiplier = baseSpeed / 50;
  const baseDuration = 10000; // 基础10秒
  const animationDuration = baseDuration / speedMultiplier;
  
  // 计算弹幕完全离开起始位置的时间（约1/4的动画时长）
  const clearTime = animationDuration * 0.25; 
  rowUsage.value[row].lastEndTime = now + clearTime;
  
  activeDanmus.value.push(activeDanmu);
  
  // 更新活跃弹幕数量
  emit('updateActiveCount', activeDanmus.value.length);
  
  console.log('弹幕已添加到活跃列表:', {
    activeDanmusCount: activeDanmus.value.length,
    animationDuration,
    clearTime,
    speed: baseSpeed
  });
  
  // 启动动画
  requestAnimationFrame(() => {
    startDanmuAnimation(activeDanmu);
  });
};

// 监听弹幕数据变化
watch(() => props.danmus, (newDanmus, oldDanmus) => {
  console.log('弹幕数据变化触发:', {
    newLength: newDanmus?.length,
    oldLength: oldDanmus?.length,
    newDanmus: newDanmus?.slice(0, 3), // 只显示前3个避免日志过长
    oldDanmus: oldDanmus?.slice(0, 3)
  });
  
  if (!newDanmus || newDanmus.length === 0) {
    console.log('新弹幕数据为空，返回');
    return;
  }
  
  const oldLength = oldDanmus?.length || 0;
  const newLength = newDanmus.length;
  
  console.log('数组长度变化:', oldLength, '->', newLength);
  
  // 如果长度增加了，处理新增的弹幕
  if (newLength > oldLength) {
    // 处理所有新增的弹幕
    const newMessagesCount = newLength - oldLength;
    console.log(`检测到 ${newMessagesCount} 条新弹幕`);
    
    for (let i = 0; i < newMessagesCount; i++) {
      const newMessage = newDanmus[i]; // 新弹幕在数组开头
      console.log('处理新弹幕:', newMessage);
      addDanmu(newMessage);
    }
  } else if (newLength === oldLength && newLength > 0) {
    // 长度相同但内容可能变化，检查第一个元素是否是新的
    const latestMessage = newDanmus[0];
    const oldLatestMessage = oldDanmus?.[0];
    
    if (!oldLatestMessage || latestMessage.msg_id !== oldLatestMessage.msg_id) {
      console.log('检测到内容变化的新弹幕:', latestMessage);
      addDanmu(latestMessage);
    } else {
      console.log('弹幕内容无变化');
    }
  } else {
    console.log('数组长度没有增加或减少，不处理新弹幕');
  }
}, { deep: true });

// 监听弹幕配置变化，重新初始化行数
watch(() => props.danmuConfig?.maxRows, () => {
  console.log('弹幕最大行数变化:', props.danmuConfig?.maxRows);
  initializeRows();
});

// 监听弹幕开关状态，关闭时清空所有弹幕
watch(() => props.danmuConfig?.enabled, (newEnabled) => {
  console.log('弹幕开关状态变化:', newEnabled);
  if (!newEnabled) {
    clearAllDanmus();
  }
});

// 监听所有弹幕配置变化
watch(() => props.danmuConfig, (newConfig) => {
  console.log('弹幕配置变化:', newConfig);
}, { deep: true });

// 清空所有弹幕
const clearAllDanmus = () => {
  activeDanmus.value.forEach(danmu => {
    if (danmu.element && danmu.element.parentNode) {
      danmu.element.parentNode.removeChild(danmu.element);
    }
  });
  activeDanmus.value = [];
  emit('clearMessages');
  emit('updateActiveCount', 0);
};

// 组件实例方法，用于直接添加弹幕
const addSingleDanmu = (danmu: DanmakuMessage) => {
  console.log('直接添加单个弹幕:', danmu);
  addDanmu(danmu);
};

// 暴露方法给父组件
defineExpose({
  addSingleDanmu,
  clearAllDanmus
});

// 组件挂载
onMounted(() => {
  console.log('Danmu组件已挂载');
  console.log('初始弹幕配置:', props.danmuConfig);
  console.log('初始弹幕数据:', props.danmus);
  
  initializeRows();
  
  // 添加全局 CSS 动画
  const style = document.createElement('style');
  style.textContent = `
    @keyframes danmuMove {
      from {
        right: -50px;
      }
      to {
        right: 100vw;
      }
    }
    
    .floating-danmu {
      transition: none !important;
    }
  `;
  document.head.appendChild(style);
  console.log('弹幕动画样式已添加');
  
  // 添加一个立即测试弹幕
  setTimeout(() => {
    console.log('添加挂载测试弹幕');
    const testDanmu: DanmakuMessage = {
      msg: "组件挂载测试弹幕",
      uname: "系统测试",
      open_id: "test-mount",
      msg_id: "test-mount-" + Date.now(),
      timestamp: Date.now(),
      room_id: 0,
      uid: 0,
      uface: "",
      dm_type: 0,
      fans_medal_wearing_status: false,
      fans_medal_name: "",
      fans_medal_level: 0,
      guard_level: 0,
      is_admin: 0,
      glory_level: 0,
      reply_open_id: "",
      reply_uname: "",
      emoji_img_url: "",
      union_id: ""
    };
    addDanmu(testDanmu);
  }, 2000);
});

// 组件卸载
onUnmounted(() => {
  clearAllDanmus();
});
</script>

<template>
  <!-- 弹幕组件现在只负责弹幕滚动，不显示任何UI -->
  <div ref="danmuContainerRef" style="display: none;"></div>
</template>

<style scoped>
/* 全局弹幕样式 */
:global(.floating-danmu) {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  user-select: none;
  will-change: transform;
}

/* 确保弹幕在最上层 */
:global(body) {
  position: relative;
}
</style>
