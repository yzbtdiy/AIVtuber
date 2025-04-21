package bilibili

import (
	"encoding/json"
	"fmt"
	"log"
	"time"
)

// ClientConfig 客户端配置
type ClientConfig struct {
	AccessKey            string // access_key
	AccessKeySecret      string // access_key_secret
	OpenPlatformHttpHost string // 开放平台地址，如 https://live-open.biliapi.com
	IdCode               string // 主播身份码
	AppId                int64  // 应用ID
}

// Client Bilibili直播客户端
type Client struct {
	config        ClientConfig
	apiClient     *APIClient
	wsClient      *WebsocketClient
	activeGameId  string
	heartbeatStop chan struct{}
}

// NewClient 创建新的Bilibili直播客户端
func NewClient(config ClientConfig) *Client {
	return &Client{
		config:        config,
		apiClient:     NewAPIClient(config.AccessKey, config.AccessKeySecret, config.OpenPlatformHttpHost),
		heartbeatStop: make(chan struct{}),
	}
}

// StartApp 启动应用
func (c *Client) StartApp() (*StartAppRespData, error) {
	startAppReq := StartAppRequest{
		Code:  c.config.IdCode,
		AppId: c.config.AppId,
	}

	reqJson, _ := json.Marshal(startAppReq)
	resp, err := c.apiClient.Request(string(reqJson), "/v2/app/start")
	if err != nil {
		return nil, fmt.Errorf("启动应用失败: %w", err)
	}

	startAppRespData := &StartAppRespData{}
	if err = json.Unmarshal(resp.Data, &startAppRespData); err != nil {
		return nil, fmt.Errorf("解析返回数据失败: %w", err)
	}

	c.activeGameId = startAppRespData.GameInfo.GameId

	return startAppRespData, nil
}

// EndApp 关闭应用
func (c *Client) EndApp() error {
	if c.activeGameId == "" {
		return fmt.Errorf("没有活跃的游戏会话")
	}

	// 停止心跳
	if c.heartbeatStop != nil {
		close(c.heartbeatStop)
		c.heartbeatStop = make(chan struct{})
	}

	endAppReq := EndAppRequest{
		GameId: c.activeGameId,
		AppId:  c.config.AppId,
	}

	reqJson, _ := json.Marshal(endAppReq)
	_, err := c.apiClient.Request(string(reqJson), "/v2/app/end")
	if err != nil {
		return fmt.Errorf("关闭应用失败: %w", err)
	}

	c.activeGameId = ""
	return nil
}

// StartAppHeartbeat 开始发送应用心跳包
func (c *Client) StartAppHeartbeat(gameId string) {
	go func() {
		ticker := time.NewTicker(time.Second * 20)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				_, err := c.AppHeartbeat(gameId)
				if err != nil {
					log.Printf("发送应用心跳失败: %v", err)
				}
			case <-c.heartbeatStop:
				return
			}
		}
	}()
}

// AppHeartbeat 发送应用心跳
func (c *Client) AppHeartbeat(gameId string) (*BaseResp, error) {
	appHeartbeatReq := AppHeartbeatReq{
		GameId: gameId,
	}
	reqJson, _ := json.Marshal(appHeartbeatReq)
	resp, err := c.apiClient.Request(string(reqJson), "/v2/app/heartbeat")
	if err != nil {
		return nil, fmt.Errorf("应用心跳请求失败: %w", err)
	}

	return &resp, nil
}

// MessageCallback 消息回调函数类型
type MessageCallback func(msgType string, msg interface{})

// ConnectWebsocketWithCallback 连接Websocket服务并使用结构化回调
func (c *Client) ConnectWebsocketWithCallback(wsAddr, authBody string, callback MessageCallback) error {
	msgHandler := func(data []byte) {
		// 解析消息
		msg, err := DecodeBilibiliMsg(data)
		if err != nil {
			log.Printf("解析消息失败: %v", err)
			return
		}

		// 根据消息类型调用回调
		switch m := msg.(type) {
		case *DanmuMessage:
			callback("danmu", m)
		case *SendGiftMessage:
			callback("send_gift", m)
		case **SuperChatMessage:
			callback("super_chat", m)
		case *SuperChatDeleteMessage:
			callback("super_chat_del", m)
		case *GuardMessage:
			callback("guard", m)
		case *LikeMessage:
			callback("like", m)
		case *LiveRoomEnterMessage:
			callback("live_room_enter", m)
		case *LiveStartMessage:
			callback("live_start", m)
		case *LiveEndMessage:
			callback("live_end", m)
		case *InteractionEndMessage:
			callback("interaction_end", m)
		default:
			callback("unknown", m)
		}
	}

	wsClient, err := NewWebsocketClient(wsAddr, authBody, msgHandler)
	if err != nil {
		return fmt.Errorf("连接WebSocket失败: %w", err)
	}

	c.wsClient = wsClient
	return nil
}

// ConnectWebsocket 连接Websocket服务
func (c *Client) ConnectWebsocket(wsAddr, authBody string, msgHandler func([]byte)) error {
	wsClient, err := NewWebsocketClient(wsAddr, authBody, msgHandler)
	if err != nil {
		return fmt.Errorf("连接WebSocket失败: %w", err)
	}

	c.wsClient = wsClient
	return nil
}

// Close 关闭客户端及所有连接
func (c *Client) Close() error {
	// 先关闭应用
	if c.activeGameId != "" {
		if err := c.EndApp(); err != nil {
			log.Printf("关闭应用失败: %v", err)
		}
	}

	// 关闭WebSocket连接
	if c.wsClient != nil {
		if err := c.wsClient.Close(); err != nil {
			log.Printf("关闭WebSocket连接失败: %v", err)
		}
	}

	// 关闭API客户端
	if c.apiClient != nil {
		c.apiClient.Close()
	}

	return nil
}
