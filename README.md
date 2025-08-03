# AIVtuber(开发中...)

> 本项目绝大多数代码都由Claude4生成

基于 Tauri + Vue.js + Three VRM + Rust 的哔哩哔哩虚拟主播

![alt text](app.png)


## 项目简介

这是一个用于接收哔哩哔哩直播间实时消息的桌面应用程序。通过哔哩哔哩开放平台的API，可以实时接收弹幕、礼物、醒目留言、大航海、点赞等各种直播间消息，大语言模型理解和生成回应内容，语音合成将文本转换为虚拟角色的声音，虚拟形象能够做出相应的表情和动作。

## 功能特性

- 🎯 **实时消息接收**：支持接收哔哩哔哩直播间多种消息类型
- 💾 **消息管理**：消息历史记录和清空功能（最多保存100条）
- 🎨 **现代UI**：基于Vue.js的现代化用户界面
- ⚡ **高性能**：Rust后端确保高性能和低资源占用
- 🔄 **自动重连**：连接断开时自动重连机制
- 📝 **详细日志**：完整的消息解析和错误处理日志
- 🎪 **消息格式化**：友好的消息显示格式

## 技术栈

- **前端**: Vue.js 3 + TypeScript + Vite
- **后端**: Rust + Tauri
- **通信**: WebSocket + HTTP
- **协议**: 哔哩哔哩直播开放平台协议

## 安装和运行

### 前提条件

1. 安装 [Node.js](https://nodejs.org/) (推荐LTS版本)
2. 安装 [Rust](https://rustup.rs/)
3. 安装 [Bun](https://bun.sh/) (可选，也可以使用npm/yarn)

### 克隆项目

```bash
git clone <项目地址>
cd AIVtuber
```

### 安装依赖

```bash
# 使用 bun
bun install
```

### 开发模式

```bash
# 启动开发模式
bun run tauri dev
```

### 构建应用

```bash
# 构建应用
bun run tauri build
```

## 配置说明

使用前需要在哔哩哔哩开放平台获取以下配置信息：

1. **主播身份码 (id_code)**
2. **应用ID (app_id)**
3. **Access Key (access_key)**
4. **Access Secret (access_secret)**

## 使用方法

1. 启动应用程序
2. 在配置表单中填入从哔哩哔哩开放平台获得的配置信息
3. 点击"连接直播间"按钮
4. 连接成功后，开始直播，消息会实时显示在下方的消息区域
5. 使用完毕后记得点击"断开连接"

### 消息类型

- 弹幕消息 (LIVE_OPEN_PLATFORM_DM)
- 礼物消息 (LIVE_OPEN_PLATFORM_SEND_GIFT)
- 醒目留言 (LIVE_OPEN_PLATFORM_SUPER_CHAT)
- 醒目留言删除 (LIVE_OPEN_PLATFORM_SUPER_CHAT_DEL)
- 大航海 (LIVE_OPEN_PLATFORM_GUARD)
- 点赞 (LIVE_OPEN_PLATFORM_LIKE)
- 进入直播间 (LIVE_OPEN_PLATFORM_LIVE_ROOM_ENTER)
- 开始直播 (LIVE_OPEN_PLATFORM_LIVE_START)
- 结束直播 (LIVE_OPEN_PLATFORM_LIVE_END)
- 消息推送结束 (LIVE_OPEN_PLATFORM_INTERACTION_END)

### 消息显示格式

- `[弹幕] 用户名: 弹幕内容`
- `[礼物] 用户名 送出了 3 个 小花花`
- `[醒目留言] 用户名: 留言内容 (50元)`
- `[上舰] 用户名 购买了 3 级大航海`
- `[点赞] 用户名 点赞了 10 次`
- `[进入直播间] 用户名 进入了直播间`

## 项目结构

```
AIVtuber-tauri/
├── src/                    # 前端代码
│   ├── App.vue            # 主应用组件
│   ├── main.ts            # 前端入口
│   └── assets/            # 静态资源
├── src-tauri/             # Rust后端代码
│   ├── src/
│   │   ├── main.rs        # 程序入口
│   │   ├── lib.rs         # 核心逻辑
│   │   ├── bilibili.rs    # 哔哩哔哩API客户端
│   │   └── proto.rs       # 协议解析
│   ├── Cargo.toml         # Rust依赖配置
│   └── tauri.conf.json    # Tauri配置
├── package.json
├── README.md
└── BILIBILI_CONFIG.md
```

## 开发说明

### 添加新功能

1. 前端功能：编辑 `src/App.vue`
2. 后端功能：编辑 `src-tauri/src/lib.rs`
3. 新增消息类型：编辑 `src-tauri/src/proto.rs`

### 调试

- 前端调试：浏览器开发者工具
- 后端调试：使用 `log` 宏输出日志
- 运行时错误：检查终端输出

## 贡献指南

1. Fork 本项目
2. 创建新的功能分支
3. 提交你的修改
4. 创建 Pull Request

## 许可证

[MIT License](LICENSE)

## 注意事项

1. 请确保你的哔哩哔哩账号有直播权限
2. 配置信息请妥善保管，不要泄露给他人
3. 本工具仅用于学习和开发目的
4. 使用时请遵守哔哩哔哩的相关条款和政策

## 问题反馈

如果遇到问题，请在 GitHub Issues 中提交问题报告。
