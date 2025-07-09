<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// 定义配置类型
interface BilibiliConfig {
  id_code: string;
  app_id: number;
  access_key: string;
  access_secret: string;
  host: string;
}

// bilibili连接状态
const isConnected = ref(false);
const connectionStatus = ref("未连接");

// bilibili配置
const bilibiliConfig = ref<BilibiliConfig>({
  id_code: "",
  app_id: 0,
  access_key: "",
  access_secret: "",
  host: "https://live-open.biliapi.com"
});

// 消息列表
const messages = ref<any[]>([]);

// 消息监听器
let messageUnlisten: (() => void) | null = null;

async function connectBilibili() {
  try {
    connectionStatus.value = "连接中...";
    const result = await invoke("connect_bilibili", { 
      config: bilibiliConfig.value 
    });
    
    isConnected.value = true;
    connectionStatus.value = "已连接";
    console.log("连接成功:", result);
  } catch (error) {
    console.error("连接失败:", error);
    connectionStatus.value = "连接失败: " + error;
    isConnected.value = false;
  }
}

async function disconnectBilibili() {
  try {
    await invoke("disconnect_bilibili");
    isConnected.value = false;
    connectionStatus.value = "未连接";
    console.log("断开连接成功");
  } catch (error) {
    console.error("断开连接失败:", error);
  }
}

async function checkConnectionStatus() {
  try {
    const status = await invoke("get_connection_status");
    isConnected.value = status as boolean;
    connectionStatus.value = status ? "已连接" : "未连接";
  } catch (error) {
    console.error("检查连接状态失败:", error);
  }
}

function formatMessage(message: any) {
  switch (message.cmd) {
    case "LIVE_OPEN_PLATFORM_DM":
      return `[弹幕] ${message.data.uname}: ${message.data.msg}`;
    case "LIVE_OPEN_PLATFORM_SEND_GIFT":
      return `[礼物] ${message.data.uname} 送出了 ${message.data.gift_num} 个 ${message.data.gift_name}`;
    case "LIVE_OPEN_PLATFORM_SUPER_CHAT":
      return `[醒目留言] ${message.data.uname}: ${message.data.message} (${message.data.rmb}元)`;
    case "LIVE_OPEN_PLATFORM_SUPER_CHAT_DEL":
      return `[醒目留言删除] 删除了 ${message.data.message_ids.length} 条留言`;
    case "LIVE_OPEN_PLATFORM_GUARD":
      return `[上舰] ${message.data.user_info.uname} 购买了 ${message.data.guard_level} 级大航海`;
    case "LIVE_OPEN_PLATFORM_LIKE":
      return `[点赞] ${message.data.uname} ${message.data.like_text || '点赞了'} ${message.data.like_count} 次`;
    case "LIVE_OPEN_PLATFORM_LIVE_ROOM_ENTER":
      return `[进入直播间] ${message.data.uname} 进入了直播间`;
    case "LIVE_OPEN_PLATFORM_LIVE_START":
      return `[开始直播] 直播间 ${message.data.room_id} 开始直播，标题：${message.data.title}`;
    case "LIVE_OPEN_PLATFORM_LIVE_END":
      return `[结束直播] 直播间 ${message.data.room_id} 结束直播`;
    case "LIVE_OPEN_PLATFORM_INTERACTION_END":
      return `[消息推送结束] 连接 ${message.data.game_id} 已结束`;
    default:
      return `[未知消息] ${JSON.stringify(message)}`;
  }
}

function addMessage(message: any) {
  const formattedMessage = {
    id: Date.now(),
    timestamp: new Date().toLocaleString(),
    formatted: formatMessage(message),
    raw: message
  };
  messages.value.unshift(formattedMessage);
  
  // 只保留最近100条消息
  if (messages.value.length > 100) {
    messages.value = messages.value.slice(0, 100);
  }
}

function clearMessages() {
  messages.value = [];
}

async function loadConfigFromFile() {
  try {
    const result = await invoke("load_config_from_file");
    if (result) {
      bilibiliConfig.value = result as BilibiliConfig;
      console.log("配置文件加载成功:", result);
      alert("配置文件加载成功！");
    } else {
      console.log("未找到配置文件");
      alert("未找到配置文件。配置文件应该放在项目根目录下，命名为 config.json");
    }
  } catch (error) {
    console.error("加载配置文件失败:", error);
    alert("加载配置文件失败: " + error);
  }
}

async function saveConfigToFile() {
  try {
    const result = await invoke("save_config_to_file", { 
      config: bilibiliConfig.value 
    });
    console.log("配置文件保存成功:", result);
    alert("配置文件保存成功！\n" + result);
  } catch (error) {
    console.error("保存配置文件失败:", error);
    alert("保存配置文件失败: " + error);
  }
}

onMounted(async () => {
  // 检查连接状态
  await checkConnectionStatus();
  
  // 尝试自动加载配置文件
  try {
    const result = await invoke("load_config_from_file");
    if (result) {
      bilibiliConfig.value = result as BilibiliConfig;
      console.log("自动加载配置文件成功:", result);
    }
  } catch (error) {
    console.log("自动加载配置文件失败:", error);
  }
  
  // 监听bilibili消息
  messageUnlisten = await listen("bilibili-message", (event) => {
    addMessage(event.payload);
  });
});

onUnmounted(() => {
  if (messageUnlisten) {
    messageUnlisten();
  }
});
</script>

<template>
  <main class="container">
    <h1>AIVtuber - 哔哩哔哩直播间连接工具</h1>

    <!-- 哔哩哔哩连接配置 -->
    <div class="section">
      <h2>哔哩哔哩直播间连接</h2>
      
      <div class="config-form">
        <div class="form-group">
          <label>主播身份码:</label>
          <input 
            v-model="bilibiliConfig.id_code" 
            placeholder="请输入主播身份码"
            :disabled="isConnected"
          />
        </div>
        
        <div class="form-group">
          <label>应用ID:</label>
          <input 
            v-model.number="bilibiliConfig.app_id" 
            type="number"
            placeholder="请输入应用ID"
            :disabled="isConnected"
          />
        </div>
        
        <div class="form-group">
          <label>Access Key:</label>
          <input 
            v-model="bilibiliConfig.access_key" 
            placeholder="请输入Access Key"
            :disabled="isConnected"
          />
        </div>
        
        <div class="form-group">
          <label>Access Secret:</label>
          <input 
            v-model="bilibiliConfig.access_secret" 
            type="password"
            placeholder="请输入Access Secret"
            :disabled="isConnected"
          />
        </div>
        
        <div class="form-group">
          <label>服务器地址:</label>
          <input 
            v-model="bilibiliConfig.host" 
            placeholder="https://live-open.biliapi.com"
            :disabled="isConnected"
          />
        </div>
        
        <div class="connection-controls">
          <button 
            @click="connectBilibili" 
            :disabled="isConnected"
            class="connect-btn"
          >
            连接直播间
          </button>
          
          <button 
            @click="disconnectBilibili" 
            :disabled="!isConnected"
            class="disconnect-btn"
          >
            断开连接
          </button>
          
          <span class="status" :class="{ connected: isConnected }">
            {{ connectionStatus }}
          </span>
        </div>
      </div>
      
      <!-- 配置文件操作按钮 -->
      <div class="config-file-section">
        <h3>配置文件操作</h3>
        <div class="config-file-buttons">
          <button @click="loadConfigFromFile" class="load-config-btn">
            📁 加载配置文件
          </button>
          <button @click="saveConfigToFile" class="save-config-btn">
            💾 保存配置文件
          </button>
        </div>
        <div class="config-file-info">
          <p>• 配置文件位置：项目根目录下的 config.json</p>
          <p>• 加载配置文件会自动填充上方表单</p>
          <p>• 保存配置文件会将当前表单内容写入文件</p>
        </div>
      </div>
    </div>

    <!-- 消息显示区域 -->
    <div class="section">
      <h2>直播间消息</h2>
      
      <div class="message-controls">
        <button @click="clearMessages" class="clear-btn">清空消息</button>
        <span class="message-count">消息数量: {{ messages.length }}</span>
      </div>
      
      <div class="messages-container">
        <div 
          v-for="message in messages" 
          :key="message.id"
          class="message-item"
        >
          <div class="message-time">{{ message.timestamp }}</div>
          <div class="message-content">{{ message.formatted }}</div>
        </div>
        
        <div v-if="messages.length === 0" class="no-messages">
          暂无消息
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.section {
  margin-bottom: 30px;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #f9f9f9;
}

h1 {
  color: #333;
  text-align: center;
  margin-bottom: 30px;
}

h2 {
  color: #555;
  margin-bottom: 20px;
}

.row {
  display: flex;
  gap: 10px;
  align-items: center;
}

.config-form {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.form-group label {
  font-weight: bold;
  color: #555;
}

.form-group input {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.form-group input:disabled {
  background-color: #f5f5f5;
  cursor: not-allowed;
}

.connection-controls {
  display: flex;
  gap: 15px;
  align-items: center;
  margin-top: 20px;
}

.connect-btn, .disconnect-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.connect-btn {
  background-color: #4CAF50;
  color: white;
}

.connect-btn:hover:not(:disabled) {
  background-color: #45a049;
}

.disconnect-btn {
  background-color: #f44336;
  color: white;
}

.disconnect-btn:hover:not(:disabled) {
  background-color: #da190b;
}

.connect-btn:disabled, .disconnect-btn:disabled {
  background-color: #cccccc;
  cursor: not-allowed;
}

.status {
  padding: 5px 10px;
  border-radius: 4px;
  font-weight: bold;
  color: #666;
  background-color: #e0e0e0;
}

.status.connected {
  background-color: #d4edda;
  color: #155724;
}

/* 配置文件操作样式 */
.config-file-section {
  margin-top: 30px;
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #f9f9f9;
}

.config-file-section h3 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #333;
  font-size: 16px;
}

.config-file-buttons {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.load-config-btn,
.save-config-btn {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background-color: #fff;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.load-config-btn:hover {
  background-color: #e8f4fd;
  border-color: #007acc;
}

.save-config-btn:hover {
  background-color: #e8f5e8;
  border-color: #28a745;
}

.config-file-info {
  font-size: 12px;
  color: #666;
  line-height: 1.4;
}

.config-file-info p {
  margin: 4px 0;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }

  .config-file-section {
    background-color: #1a1a1a;
    border-color: #444;
  }
  
  .config-file-section h3 {
    color: #fff;
  }
  
  .load-config-btn,
  .save-config-btn {
    background-color: #2a2a2a;
    border-color: #444;
    color: #fff;
  }
  
  .config-file-info {
    color: #ccc;
  }
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}
</style>