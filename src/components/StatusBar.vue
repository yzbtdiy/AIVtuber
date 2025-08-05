<script setup lang="ts">
import { ref } from 'vue';

// Props 接收数据
interface Props {
  // 连接相关
  isConnected?: boolean;
  connectionStatus?: string;
  connectionStats?: {
    totalMessages: number;
    danmuCount: number;
    giftCount: number;
    connectTime: Date | null;
  };
  
  // 弹幕配置
  danmuConfig?: {
    enabled: boolean;
    speed: number;
    fontSize: number;
    opacity: number;
    maxRows: number;
    minInterval: number;
  };
  
  // 活跃弹幕数量
  activeDanmusCount?: number;
}

const props = withDefaults(defineProps<Props>(), {
  isConnected: false,
  connectionStatus: '未连接',
  connectionStats: () => ({ totalMessages: 0, danmuCount: 0, giftCount: 0, connectTime: null }),
  danmuConfig: () => ({
    enabled: true,
    speed: 50,
    fontSize: 16,
    opacity: 0.9,
    maxRows: 10,
    minInterval: 500
  }),
  activeDanmusCount: 0
});

// Events 发送事件
const emit = defineEmits<{
  // 连接控制
  connectBilibili: [];
  disconnectBilibili: [];
  
  // 弹幕控制
  toggleDanmu: [];
  clearAllDanmus: [];
  updateDanmuConfig: [config: {
    enabled: boolean;
    speed: number;
    fontSize: number;
    opacity: number;
    maxRows: number;
    minInterval: number;
  }];
}>();

// 本地弹幕配置状态
const localDanmuConfig = ref({ ...props.danmuConfig });

// 切换弹幕显示
const toggleDanmu = () => {
  localDanmuConfig.value.enabled = !localDanmuConfig.value.enabled;
  emit('updateDanmuConfig', { ...localDanmuConfig.value });
  emit('toggleDanmu');
};

// 更新弹幕配置
const updateConfig = () => {
  emit('updateDanmuConfig', { ...localDanmuConfig.value });
};

// 清空弹幕
const clearAllDanmus = () => {
  emit('clearAllDanmus');
};

// 连接控制
const connectBilibili = () => {
  emit('connectBilibili');
};

const disconnectBilibili = () => {
  emit('disconnectBilibili');
};


// 监听props变化，同步本地配置
import { watch } from 'vue';
watch(() => props.danmuConfig, (newConfig) => {
  localDanmuConfig.value = { ...newConfig };
}, { deep: true });
</script>

<template>
  <div class="status-bar">
    <!-- 连接状态 -->
    <div class="connection-status">
      <span class="status-indicator" :class="{ connected: isConnected }"></span>
      <span class="status-text">{{ connectionStatus }}</span>
    </div>

    <!-- 统计信息 -->
    <div class="stats">
      <span class="stat-item">弹幕: {{ connectionStats?.danmuCount || 0 }}</span>
      <span class="stat-item">礼物: {{ connectionStats?.giftCount || 0 }}</span>
      <span class="stat-item">活跃: {{ activeDanmusCount }}</span>
    </div>

    <!-- 连接控制按钮 -->
    <div class="connection-buttons">
      <button 
        v-show="!isConnected" 
        @click="connectBilibili" 
        class="btn btn-connect"
      >
        接入
      </button>
      <button 
        v-show="isConnected" 
        @click="disconnectBilibili" 
        class="btn btn-disconnect"
      >
        断开
      </button>
    </div>

    <!-- 弹幕控制 -->
    <div class="danmu-controls">
      <button 
        @click="toggleDanmu" 
        class="control-btn" 
        :class="{ active: localDanmuConfig.enabled }"
        title="切换弹幕显示"
      >
        {{ localDanmuConfig.enabled ? '弹幕开' : '弹幕关' }}
      </button>
      
      <button 
        @click="clearAllDanmus" 
        class="control-btn"
        title="清空弹幕"
      >
        🗑️
      </button>
    
    </div>

    <!-- 滑块控制 -->
    <div class="config-controls">
      <div class="config-control">
        <label>速度:</label>
        <input 
          v-model="localDanmuConfig.speed" 
          @input="updateConfig"
          type="range" 
          min="20" 
          max="100" 
          step="5"
          class="config-slider"
        />
        <span class="config-value">{{ localDanmuConfig.speed }}</span>
      </div>
      
      <div class="config-control">
        <label>透明度:</label>
        <input 
          v-model="localDanmuConfig.opacity" 
          @input="updateConfig"
          type="range" 
          min="0.3" 
          max="1" 
          step="0.1"
          class="config-slider"
        />
        <span class="config-value">{{ Math.round(localDanmuConfig.opacity * 100) }}%</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.status-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 60px;
  background: linear-gradient(135deg, rgba(0, 0, 0, 0.9), rgba(30, 30, 30, 0.95));
  backdrop-filter: blur(15px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 2px 20px rgba(0, 0, 0, 0.3);
  color: white;
  z-index: 1001;
  display: flex;
  align-items: center;
  padding: 0 20px;
  gap: 20px;
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9em;
  min-width: 100px;
}

.status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #ff4757;
  transition: background-color 0.3s ease;
  animation: pulse 2s infinite;
}

.status-indicator.connected {
  background-color: #2ed573;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.status-text {
  font-weight: 500;
  font-size: 0.85em;
}

.stats {
  display: flex;
  gap: 12px;
  align-items: center;
}

.stat-item {
  font-size: 0.8em;
  color: #ccc;
  font-weight: 500;
  padding: 4px 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  white-space: nowrap;
}

.connection-buttons {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.8em;
  font-weight: 500;
  transition: all 0.3s ease;
  white-space: nowrap;
}

.btn-connect {
  background: linear-gradient(135deg, #2ed573, #1e90ff);
  color: white;
}

.btn-connect:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(46, 213, 115, 0.4);
}

.btn-disconnect {
  background: linear-gradient(135deg, #ff4757, #ff6b7a);
  color: white;
}

.btn-disconnect:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(255, 71, 87, 0.4);
}

.danmu-controls {
  display: flex;
  gap: 8px;
  align-items: center;
}

.control-btn {
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 0.8em;
  font-weight: 500;
  border: 1px solid rgba(255, 255, 255, 0.1);
  white-space: nowrap;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}

.control-btn.active {
  background: linear-gradient(135deg, #2ed573, #1e90ff);
  color: white;
  border-color: rgba(46, 213, 115, 0.3);
}

.config-controls {
  display: flex;
  gap: 20px;
  align-items: center;
  margin-left: auto;
}

.config-control {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8em;
  white-space: nowrap;
}

.config-control label {
  min-width: 40px;
  color: #ccc;
  font-weight: 500;
  font-size: 0.75em;
}

.config-slider {
  width: 80px;
  height: 3px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.2);
  outline: none;
  appearance: none;
  -webkit-appearance: none;
}

.config-slider::-webkit-slider-thumb {
  appearance: none;
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: linear-gradient(135deg, #2ed573, #1e90ff);
  cursor: pointer;
  box-shadow: 0 2px 6px rgba(46, 213, 115, 0.4);
  transition: all 0.3s ease;
}

.config-slider::-webkit-slider-thumb:hover {
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(46, 213, 115, 0.6);
}

.config-slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: linear-gradient(135deg, #2ed573, #1e90ff);
  cursor: pointer;
  border: none;
  box-shadow: 0 2px 6px rgba(46, 213, 115, 0.4);
}

.config-value {
  min-width: 35px;
  text-align: center;
  color: #2ed573;
  font-weight: 600;
  font-size: 0.75em;
}

/* 响应式设计 */
@media (max-width: 1200px) {
  .status-bar {
    gap: 15px;
    padding: 0 15px;
  }
  
  .config-controls {
    gap: 15px;
  }
  
  .config-slider {
    width: 60px;
  }
}

@media (max-width: 768px) {
  .status-bar {
    height: 50px;
    gap: 10px;
    padding: 0 10px;
    font-size: 0.9em;
  }
  
  .stats {
    gap: 8px;
  }
  
  .stat-item {
    font-size: 0.7em;
    padding: 3px 6px;
  }
  
  .btn, .control-btn {
    padding: 4px 8px;
    font-size: 0.7em;
  }
  
  .config-controls {
    gap: 10px;
  }
  
  .config-slider {
    width: 50px;
  }
  
  .config-control label {
    font-size: 0.7em;
  }
  
  .config-value {
    font-size: 0.7em;
  }
}

@media (max-width: 480px) {
  .status-bar {
    height: 45px;
    gap: 8px;
    padding: 0 8px;
    flex-wrap: nowrap;
    overflow-x: auto;
  }
  
  .connection-status {
    min-width: 80px;
  }
  
  .stats {
    gap: 6px;
  }
  
  .stat-item {
    font-size: 0.65em;
    padding: 2px 4px;
  }
  
  .btn, .control-btn {
    padding: 3px 6px;
    font-size: 0.65em;
  }
  
  .config-controls {
    gap: 8px;
  }
  
  .config-control {
    gap: 4px;
  }
  
  .config-slider {
    width: 40px;
  }
}
</style>
