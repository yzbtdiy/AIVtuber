<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { Events } from "@wailsio/runtime";

// 定义单个弹幕项的接口
interface DanmuItem {
  id: number; // 添加唯一 ID 以便 v-for key 绑定
  uface: string;
  msg: string;
}

// 定义接收到的事件数据结构
interface DanmuEventData {
  data: [{
    uface: string;
    msg: string;
  }];
}

// 创建一个 ref 来引用 DOM 元素 (用于滚动控制，如果需要)
const danmuContainerRef = ref<HTMLDivElement | null>(null);
// 创建响应式数组来存储弹幕
const danmus = ref<DanmuItem[]>([]);
// 用于生成唯一 ID
let nextId = 0;
// 最大弹幕数量
const maxDanmuCount = 100;

// 添加弹幕的函数 - 更新响应式数组
const addDanmu = (eventData: DanmuEventData) => {
  console.log('Received message:', eventData);
  if (eventData?.data?.[0]) {
    const danmuInfo = eventData.data[0];

    // 创建新的弹幕对象
    const newDanmu: DanmuItem = {
      id: nextId++,
      // 注意：代理 URL 仍然需要确保安全和可用性
      uface: `http://127.0.0.1:12345/proxy?url=${encodeURIComponent(danmuInfo.uface)}`,
      msg: danmuInfo.msg,
    };

    // 将新弹幕添加到数组开头
    danmus.value.unshift(newDanmu);

    // 限制弹幕数量
    if (danmus.value.length > maxDanmuCount) {
      danmus.value.pop(); // 移除数组末尾（最旧的）弹幕
    }

    // 可选：如果希望新弹幕出现时滚动到底部或顶部，可以在这里处理
    // 例如，滚动到顶部 (因为我们用 unshift)
    // nextTick(() => {
    //   if (danmuContainerRef.value) {
    //     danmuContainerRef.value.scrollTop = 0;
    //   }
    // });

  } else {
      console.warn("Invalid event data received:", eventData);
  }
};

// 处理图片加载失败
const handleAvatarError = (event: Event) => {
    const imgElement = event.target as HTMLImageElement;
    imgElement.style.display = 'none'; // 隐藏加载失败的头像
    // 或者设置一个默认头像
    // imgElement.src = '/path/to/default/avatar.png';
    console.error(`Failed to load avatar: ${imgElement.src}`);
};


// 在组件挂载后监听事件
onMounted(() => {
  Events.On("CHAT:QUESTION", addDanmu);
});

// 在组件卸载前移除监听
onUnmounted(() => {
  Events.Off("CHAT:QUESTION");
});

</script>

<template>
  <div class="danmu-container" ref="danmuContainerRef">
    <!-- 使用 v-for 渲染弹幕列表 -->
    <div v-for="danmu in danmus" :key="danmu.id" class="danmu-item">
      <img :src="danmu.uface" alt="avatar" class="danmu-avatar" @error="handleAvatarError" />
      <span class="danmu-message">{{ danmu.msg }}</span>
    </div>
  </div>
</template>

<style scoped>
.danmu-container {
  width: 30vw;
  height: 90vh;
  /* border: 1px solid #ccc; */ /* 移除或调整边框 */
  overflow-y: auto;
  background-color: rgba(245, 245, 220, 0.8);
  color: black;
  padding: 10px;
  box-sizing: border-box;
  border-radius: 1rem;
  position: fixed;
  right: 3vw;
  top: 5vh;
  display: flex;
  flex-direction: column; /* 保持垂直排列 */
  /* gap: 10px; */ /* 可以用 gap 代替 margin-bottom */
  border: 1px solid rgba(0, 0, 0, 0.1);
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  scrollbar-width: thin;
  scrollbar-color: rgba(0,0,0,0.2) transparent;
}



/* 弹幕项样式 */
.danmu-item {
  display: flex;
  align-items: center;
  margin-bottom: 10px; /* 如果不用 gap，则保留 */
  word-break: break-word;
}

/* 头像样式 */
.danmu-avatar {
   width: 30px;
   height: 30px;
   border-radius: 50%;
   margin-right: 10px;
   background-color: transparent; /* 可以设置一个占位背景色 */
   flex-shrink: 0; /* 防止头像被压缩 */
   object-fit: cover; /* 保持图片比例 */
}

/* 消息样式 */
.danmu-message {
    padding: 5px 10px;
    background-color: rgba(0, 255, 47, 0.2);
    border-radius: 5px;
    color: black;
}
</style>