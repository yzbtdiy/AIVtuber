<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Danmu from '../components/Danmu.vue';
import StatusBar from '../components/StatusBar.vue';
import type {
  BilibiliLiveMessage,
  DanmakuMessage,
  GiftMessage,
  SuperChatMessage,
  SuperChatDelMessage,
  GuardMessage,
  LikeMessage,
  RoomEnterMessage,
  LiveStartMessage,
  LiveEndMessage,
  InteractionEndMessage
} from '../types/bilibililive';
import { LivePlatformCmd } from '../types/bilibililive';
import VTuberCanvas from '../components/VTuberCanvas.vue';
import { base64ToArrayBuffer, chatAndSpeakIntegrated } from "../api/chat";

const vtuberCanvasRef = ref<InstanceType<typeof VTuberCanvas>>()

const playAudio = (audioData: ArrayBuffer) => {
  vtuberCanvasRef.value?.playAudio(audioData)
}
// 定义配置类型
interface BilibiliConfig {
  id_code: string;
  app_id: number;
  access_key: string;
  access_secret: string;
  host: string;
}

// 定义格式化后的消息类型
interface FormattedMessage {
  id: number;
  timestamp: string;
  type: 'danmu' | 'gift' | 'superchat' | 'guard' | 'like' | 'enter' | 'system';
  content: string;
  user?: {
    name: string;
    avatar: string;
    open_id: string;
  };
  raw: BilibiliLiveMessage;
}

// bilibili连接状态
const isConnected = ref(false);
const connectionStatus = ref("未连接");

// 代理服务状态
const isProxyRunning = ref(false);
const proxyStatus = ref("未运行");

// bilibili配置
const bilibiliConfig = ref<BilibiliConfig>({
  id_code: "",
  app_id: 0,
  access_key: "",
  access_secret: "",
  host: "https://live-open.biliapi.com"
});

// 消息列表
const messages = ref<FormattedMessage[]>([]);

// 弹幕列表 - 仅存储弹幕消息
const danmus = ref<DanmakuMessage[]>([]);

// 消息监听器
let messageUnlisten: (() => void) | null = null;

// 连接状态相关
const connectionStats = ref({
  totalMessages: 0,
  danmuCount: 0,
  giftCount: 0,
  connectTime: null as Date | null
});

// 弹幕配置
const danmuConfig = ref({
  enabled: true,
  speed: 50,
  fontSize: 16,
  opacity: 0.9,
  maxRows: 10,
  minInterval: 500,
});

// 活跃弹幕数量
const activeDanmusCount = ref(0);

// Danmu组件引用
const danmuRef = ref();

async function connectBilibili() {
  await startProxyServer();
  try {
    connectionStatus.value = "连接中...";
    const result = await invoke("connect_bilibili", {
      config: bilibiliConfig.value
    });

    isConnected.value = true;
    connectionStatus.value = "已连接";
    connectionStats.value.connectTime = new Date();
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
    connectionStats.value.connectTime = null;
    console.log("断开连接成功");
  } catch (error) {
    console.error("断开连接失败:", error);
  } finally {
    await stopProxyServer();
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

// 类型安全的消息格式化函数
async function formatMessage(message: BilibiliLiveMessage): Promise<FormattedMessage> {
  const baseMessage = {
    id: Date.now() + Math.random(),
    timestamp: new Date().toLocaleString(),
    raw: message
  };

  switch (message.cmd) {
    case LivePlatformCmd.DM: {
      const data = message.data as DanmakuMessage;
      const chatResp = await chatAndSpeakIntegrated(data.msg);
      playAudio(base64ToArrayBuffer(chatResp.audio_data!));
      return {
        ...baseMessage,
        type: 'danmu',
        content: data.msg,
        user: {
          name: data.uname,
          avatar: data.uface,
          open_id: data.open_id
        }
      };
    }

    case LivePlatformCmd.SEND_GIFT: {
      const data = message.data as GiftMessage;
      return {
        ...baseMessage,
        type: 'gift',
        content: `送出了 ${data.gift_num} 个 ${data.gift_name}`,
        user: {
          name: data.uname,
          avatar: data.uface,
          open_id: data.open_id
        }
      };
    }

    case LivePlatformCmd.SUPER_CHAT: {
      const data = message.data as SuperChatMessage;
      return {
        ...baseMessage,
        type: 'superchat',
        content: `${data.message} (${data.rmb}元)`,
        user: {
          name: data.uname,
          avatar: data.uface,
          open_id: data.open_id
        }
      };
    }

    case LivePlatformCmd.SUPER_CHAT_DEL: {
      const data = message.data as SuperChatDelMessage;
      return {
        ...baseMessage,
        type: 'system',
        content: `删除了 ${data.message_ids.length} 条醒目留言`
      };
    }

    case LivePlatformCmd.GUARD: {
      const data = message.data as GuardMessage;
      const guardLevelText = data.guard_level === 1 ? '总督' : data.guard_level === 2 ? '提督' : '舰长';
      return {
        ...baseMessage,
        type: 'guard',
        content: `购买了 ${guardLevelText}`,
        user: {
          name: data.user_info.uname,
          avatar: data.user_info.uface,
          open_id: data.user_info.open_id
        }
      };
    }

    case LivePlatformCmd.LIKE: {
      const data = message.data as LikeMessage;
      return {
        ...baseMessage,
        type: 'like',
        content: `${data.like_text} ${data.like_count} 次`,
        user: {
          name: data.uname,
          avatar: data.uface,
          open_id: data.open_id
        }
      };
    }

    case LivePlatformCmd.LIVE_ROOM_ENTER: {
      const data = message.data as RoomEnterMessage;
      return {
        ...baseMessage,
        type: 'enter',
        content: '进入了直播间',
        user: {
          name: data.uname,
          avatar: data.uface,
          open_id: data.open_id
        }
      };
    }

    case LivePlatformCmd.LIVE_START: {
      const data = message.data as LiveStartMessage;
      return {
        ...baseMessage,
        type: 'system',
        content: `直播间 ${data.room_id} 开始直播，标题：${data.title}`
      };
    }

    case LivePlatformCmd.LIVE_END: {
      const data = message.data as LiveEndMessage;
      return {
        ...baseMessage,
        type: 'system',
        content: `直播间 ${data.room_id} 结束直播`
      };
    }

    case LivePlatformCmd.INTERACTION_END: {
      const data = message.data as InteractionEndMessage;
      return {
        ...baseMessage,
        type: 'system',
        content: `连接 ${data.game_id} 已结束`
      };
    }

    default:
      return {
        ...baseMessage,
        type: 'system',
        content: `未知消息类型: ${message.cmd}`
      };
  }
}




// 优化的消息添加函数
async function addMessage(message: BilibiliLiveMessage) {
  console.log('addMessage called with:', message);

  const formattedMessage = formatMessage(message);

  // 更新统计
  connectionStats.value.totalMessages++;
  if (message.cmd === LivePlatformCmd.DM) {
    connectionStats.value.danmuCount++;
  } else if (message.cmd === LivePlatformCmd.SEND_GIFT) {
    connectionStats.value.giftCount++;
  }

  messages.value.unshift(await formattedMessage);

  // 如果是弹幕消息，直接调用 Danmu 组件方法显示
  if (message.cmd === LivePlatformCmd.DM) {
    const danmuData = message.data as DanmakuMessage;
    console.log('Processing danmu message:', danmuData);
    // 直接调用 Danmu 组件的方法添加弹幕
    if (danmuRef.value && danmuRef.value.addSingleDanmu) {
      console.log('直接调用Danmu组件方法显示弹幕:', danmuData);
      danmuRef.value.addSingleDanmu(danmuData);
    } else {
      console.log('Danmu组件引用不可用，无法显示弹幕');
    }

    // 同时添加到弹幕列表用于统计（但不用于Watch监听）
    danmus.value.unshift(danmuData);

    // 限制弹幕列表长度
    if (danmus.value.length > 50) {
      danmus.value.splice(50);
    }
  }

  // 保留所有类型消息，但限制总数
  if (messages.value.length > 200) {
    messages.value = messages.value.slice(0, 200);
  }
}

// 清空弹幕
function clearMessages() {
  messages.value = [];
  danmus.value = [];
  connectionStats.value = {
    totalMessages: 0,
    danmuCount: 0,
    giftCount: 0,
    connectTime: connectionStats.value.connectTime
  };
}

// 代理服务控制函数
async function startProxyServer() {
  try {
    proxyStatus.value = "启动中...";
    const result = await invoke("start_proxy_server", { port: 12345 });
    console.log("代理服务启动结果:", result);

    isProxyRunning.value = true;
    proxyStatus.value = "运行中";
    // alert("代理服务已启动在端口 12345");
  } catch (error) {
    console.error("启动代理服务失败:", error);
    proxyStatus.value = "启动失败";
    isProxyRunning.value = false;
    alert("启动代理服务失败: " + error);
  }
}

async function stopProxyServer() {
  try {
    const result = await invoke("stop_proxy_server");
    console.log("代理服务停止结果:", result);

    isProxyRunning.value = false;
    proxyStatus.value = "未运行";
    // alert("代理服务已停止");
  } catch (error) {
    console.error("停止代理服务失败:", error);
    alert("停止代理服务失败: " + error);
  }
}



// StatusBar事件处理函数
const handleToggleDanmu = () => {
  danmuConfig.value.enabled = !danmuConfig.value.enabled;
  console.log('切换弹幕状态:', danmuConfig.value.enabled);
};

const handleClearAllDanmus = () => {
  clearMessages();
  console.log('清空所有弹幕');
};

// const handleUpdateDanmuConfig = (config: typeof danmuConfig.value) => {
//   danmuConfig.value = { ...config };
//   console.log('更新弹幕配置:', config);
// };

const handleUpdateActiveCount = (count: number) => {
  activeDanmusCount.value = count;
  console.log('活跃弹幕数量:', count);
};

onMounted(async () => {
  // 检查连接状态
  await checkConnectionStatus();
  // 尝试自动加载配置文件
  try {
    const result = await invoke("load_config_from_file");
    if (result) {
      bilibiliConfig.value = result as BilibiliConfig;
      console.log("自动加载配置文件成功:", result);
    } else {
      console.log("读取配置文件失败");
    }
    // await connectBilibili();

  } catch (error) {
    console.log("初始化直播间失败:", error);
  }

  // 监听bilibili消息
  messageUnlisten = await listen("bilibili-message", (event) => {
    const message = event.payload as BilibiliLiveMessage;
    console.log('收到bilibili消息:', message);
    addMessage(message);
  });
});

onUnmounted(() => {
  if (messageUnlisten) {
    messageUnlisten();
  }
});
</script>

<template>
  <!-- 状态栏组件 -->
  <VTuberCanvas ref="vtuberCanvasRef" />
  <StatusBar :is-connected="isConnected" :connection-status="connectionStatus" :connection-stats="connectionStats"
    :danmu-config="danmuConfig" :active-danmus-count="activeDanmusCount" @connect-bilibili="connectBilibili"
    @disconnect-bilibili="disconnectBilibili" @toggle-danmu="handleToggleDanmu"
    @clear-all-danmus="handleClearAllDanmus" />

  <!-- 弹幕组件 -->
  <Danmu ref="danmuRef" :danmus="danmus" :connection-stats="connectionStats" :is-connected="isConnected"
    :danmu-config="danmuConfig" @clear-messages="clearMessages" @update-active-count="handleUpdateActiveCount" />
</template>

<style scoped>
/* LiveRoom 专用样式，主要样式已迁移到 StatusBar */
</style>