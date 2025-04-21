# Bilibili直播通信鉴权库

这是一个用于Bilibili直播开放平台API通信的Go语言库，提供了简单易用的API接口来连接和管理Bilibili直播应用。

## 功能特性

- 完整的HTTP API请求鉴权
- WebSocket长连接支持
- 自动心跳维护
- 结构化消息解析与处理
- 资源自动释放

## 使用示例

### 基本用法

```go
// 创建客户端配置
config := bilibili.ClientConfig{
    AccessKey:            "你的AccessKey",
    AccessKeySecret:      "你的AccessKeySecret",
    OpenPlatformHttpHost: "https://live-open.biliapi.com",
    AppId:                12345, // 你的应用ID
}

// 创建客户端
client := bilibili.NewClient(config)
defer client.Close()

// 启动应用
startAppResp, err := client.StartApp("主播身份码")
if err != nil {
    log.Fatalf("启动应用失败: %v", err)
}

// 开始发送心跳
client.StartAppHeartbeat(startAppResp.GameInfo.GameId)
```

### 使用结构化消息处理

```go
// 使用结构化消息回调连接WebSocket
err = client.ConnectWebsocketWithCallback(
    startAppResp.WebsocketInfo.WssLink[0],
    startAppResp.WebsocketInfo.AuthBody,
    func(msgType string, msg interface{}) {
        switch msgType {
        case "danmu": // 弹幕消息
            if danmu, ok := msg.(*bilibili.DanmuMessage); ok {
                log.Printf("收到弹幕: %s 说: %s", danmu.UserName, danmu.Content)
            }
        case "gift": // 礼物消息
            if gift, ok := msg.(*bilibili.GiftMessage); ok {
                log.Printf("收到礼物: %s 赠送了 %d 个 %s", gift.UserName, gift.Num, gift.GiftName)
            }
        }
    },
)
```

## 支持的消息类型

该库支持以下B站直播消息类型的解析：

1. **弹幕消息** (`DanmuMessage`)
   - 包含用户ID、用户名、弹幕内容、粉丝勋章等信息
   - 支持普通弹幕和表情包弹幕

2. **礼物消息** (`SendGiftMessage`) 
   - 包含用户信息、礼物名称、数量、价值和连击信息

3. **付费留言** (`SuperChatMessage`)
   - 包含用户信息、留言内容、金额和展示时间

4. **付费留言下线** (`SuperChatDeleteMessage`)
   - 包含被删除的留言ID信息

5. **大航海消息** (`GuardMessage`)
   - 包含用户上舰信息、舰队等级和金额

6. **点赞消息** (`LikeMessage`)
   - 包含用户点赞信息和点赞数量

7. **进入直播间** (`LiveRoomEnterMessage`)
   - 包含用户进入直播间的信息

8. **开始直播** (`LiveStartMessage`)
   - 包含直播开始的信息、分区和标题

9. **结束直播** (`LiveEndMessage`)
   - 包含直播结束的信息

10. **互动结束** (`InteractionEndMessage`)
   - 包含消息推送结束的通知

## API文档

### 客户端配置

```go
type ClientConfig struct {
    AccessKey            string // access_key
    AccessKeySecret      string // access_key_secret
    OpenPlatformHttpHost string // 开放平台地址，如 https://live-open.biliapi.com
    AppId                int64  // 应用ID
}
```

### 主要方法

- `NewClient(config ClientConfig) *Client`: 创建新的客户端
- `StartApp(idCode string) (*StartAppRespData, error)`: 启动应用
- `EndApp() error`: 关闭应用
- `StartAppHeartbeat(gameId string)`: 开始发送应用心跳包
- `AppHeartbeat(gameId string) (*BaseResp, error)`: 发送单次应用心跳
- `ConnectWebsocket(wsAddr, authBody string, msgHandler func([]byte)) error`: 连接WebSocket服务(原始消息)
- `ConnectWebsocketWithCallback(wsAddr, authBody string, callback MessageCallback) error`: 连接WebSocket服务(结构化消息)
- `Close() error`: 关闭客户端及所有连接

## 注意事项

- 请确保在应用退出前调用`Close()`方法释放资源
- WebSocket连接会自动处理WebSocket心跳，无需额外操作
- 应用心跳和WebSocket心跳是两个不同的机制，系统会自动处理
- 推荐使用`ConnectWebsocketWithCallback`方法处理消息，更加结构化和类型安全
