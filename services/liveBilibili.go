package services

import (
	"context"
	"fmt"
	"log"
	"strings"
	"sync"
	"time"

	"github.com/wailsapp/wails/v3/pkg/application"

	"github.com/yzbtdiy/AIVtuber/backend/api/zhipuai"
	"github.com/yzbtdiy/AIVtuber/backend/live/bilibili"
	"github.com/yzbtdiy/AIVtuber/backend/utils"
)

// Messageprocessor 消息处理器接口
type Messageprocessor interface {
	process(msgType string, msg interface{})
}

// BiliBiliLiveService B站直播服务
type BiliBiliLiveService struct {
	// 客户端配置相关
	config bilibili.ClientConfig
	client *bilibili.Client
	gameId string

	// 服务状态相关
	connected   bool
	started     bool
	statusMutex sync.RWMutex // 用于保护状态变量的互斥锁

	// 消息处理相关
	handlers     map[string]func(interface{})
	handlerMutex sync.Mutex

	// 上下文控制相关
	wg sync.WaitGroup

	ctx     context.Context
	options application.ServiceOptions

	glmClient       *zhipuai.GLM4VoiceModelClient
	cogVideoXClient *zhipuai.CogVideoXModelClient
}

// NewBiliBiliLiveService 创建B站直播服务
func NewBiliBiliLiveService() *BiliBiliLiveService {
	return &BiliBiliLiveService{
		config: bilibili.ClientConfig{
			AccessKey:            utils.GlobalConfig.GetString("bilibili.access_key_id"),
			AccessKeySecret:      utils.GlobalConfig.GetString("bilibili.access_key_secret"),
			OpenPlatformHttpHost: utils.GlobalConfig.GetString("bilibili.api_url"),
			IdCode:               utils.GlobalConfig.GetString("bilibili.id_code"),
			AppId:                utils.GlobalConfig.GetInt64("bilibili.app_id"),
		},
		handlers: make(map[string]func(interface{})),
	}
}

func (s *BiliBiliLiveService) Name() string {
	return "BilibiliLive"
}

// IsStarted 返回服务是否已启动
func (s *BiliBiliLiveService) IsStarted() bool {
	s.statusMutex.RLock()
	defer s.statusMutex.RUnlock()
	return s.started
}

// IsConnected 返回WebSocket是否已连接
func (s *BiliBiliLiveService) IsConnected() bool {
	s.statusMutex.RLock()
	defer s.statusMutex.RUnlock()
	return s.connected
}

// setStarted 设置服务启动状态
func (s *BiliBiliLiveService) setStarted(started bool) {
	s.statusMutex.Lock()
	defer s.statusMutex.Unlock()
	s.started = started
}

// setConnected 设置WebSocket连接状态
func (s *BiliBiliLiveService) setConnected(connected bool) {
	s.statusMutex.Lock()
	defer s.statusMutex.Unlock()
	s.connected = connected
}

// GetStatus 获取服务当前状态
func (s *BiliBiliLiveService) GetStatus() (bool, bool) {
	s.statusMutex.RLock()
	defer s.statusMutex.RUnlock()
	return s.started, s.connected
}

// ServiceStartup 启动B站直播服务
func (s *BiliBiliLiveService) OnStartup(ctx context.Context, options application.ServiceOptions) error {
	log.Println("正在启动B站直播连接...")
	if s.IsStarted() {
		return fmt.Errorf("服务已经启动")
	}

	// 直接使用传入的上下文
	s.ctx = ctx
	s.options = options

	if s.handlers == nil {
		s.handlers = make(map[string]func(interface{}))
	}

	// 注册默认消息处理函数
	s.registerDefaultHandlers()

	// 创建客户端
	s.client = bilibili.NewClient(s.config)

	// 启动应用
	startAppResp, err := s.client.StartApp()
	if err != nil {
		return fmt.Errorf("启动应用失败: %w", err)
	}

	log.Printf("应用启动成功，游戏ID: %s", startAppResp.GameInfo.GameId)
	log.Printf("主播信息: %s (房间号: %d)", startAppResp.AnchorInfo.Uname, startAppResp.AnchorInfo.RoomId)

	s.gameId = startAppResp.GameInfo.GameId

	// 启动应用心跳(使用客户端内置的心跳机制)
	s.client.StartAppHeartbeat(s.gameId)

	// 检查WebSocket连接信息
	if len(startAppResp.WebsocketInfo.WssLink) == 0 {
		s.OnShutdown()
		return fmt.Errorf("未获取到WebSocket连接地址")
	}

	// 标记服务为已启动
	s.setStarted(true)

	// 异步连接WebSocket
	s.wg.Add(1)
	go func() {
		defer s.wg.Done()
		s.connectWebSocketAsync(startAppResp.WebsocketInfo.WssLink[0], startAppResp.WebsocketInfo.AuthBody)
	}()

	log.Println("Bilibili直播监听服务已启动")
	// 初始化GLM4Voice模型客户端
	s.glmClient = zhipuai.NewGLM4VoiceModelClient()
	if err := s.glmClient.Init(); err != nil {
		log.Printf("GLM4Voice模型初始化失败: %v", err)
	}

	// 初始化CogVideoX模型客户端
	s.cogVideoXClient = zhipuai.NewCogVideoXModelClient()
	if err := s.cogVideoXClient.Init(); err != nil {
		log.Printf("CogVideoX模型初始化失败: %v", err)
	}
	return nil
}

// ServiceShutdown 关闭服务
func (s *BiliBiliLiveService) OnShutdown() error {
	log.Println("正在关闭B站直播连接...")
	if !s.IsStarted() {
		return nil
	}

	// 等待所有任务结束
	s.wg.Wait()

	// 关闭客户端连接
	if s.client != nil {
		if err := s.client.Close(); err != nil {
			log.Printf("关闭客户端连接时发生错误: %v", err)
			return err
		}
	}

	// 关闭GLM4Voice模型客户端
	if s.glmClient != nil {
		s.glmClient.Close()
	}

	// 关闭CogVideoX模型客户端
	if s.cogVideoXClient != nil {
		s.cogVideoXClient.Close()
	}

	// 更新状态
	s.setConnected(false)
	s.setStarted(false)

	log.Println("B站直播连接已关闭")
	return nil
}

// registerMessageHandler 注册消息处理函数
func (s *BiliBiliLiveService) registerMessageHandler(msgType string, handler func(interface{})) {
	s.handlerMutex.Lock()
	defer s.handlerMutex.Unlock()
	s.handlers[msgType] = handler
}

// registerDefaultHandlers 注册默认的消息处理函数
func (s *BiliBiliLiveService) registerDefaultHandlers() {
	s.registerMessageHandler("danmu", s.handleDanmu)
	s.registerMessageHandler("send_gift", s.handleGift)
	s.registerMessageHandler("super_chat", s.handleSuperChat)
	s.registerMessageHandler("super_chat_del", s.handleSuperChatDelete)
	s.registerMessageHandler("guard", s.handleGuard)
	s.registerMessageHandler("like", s.handleLike)
	s.registerMessageHandler("live_room_enter", s.handleRoomEnter)
	s.registerMessageHandler("live_start", s.handleLiveStart)
	s.registerMessageHandler("live_end", s.handleLiveEnd)
	s.registerMessageHandler("interaction_end", s.handleInteractionEnd)
}

// process 处理消息
func (s *BiliBiliLiveService) process(msgType string, msg interface{}) {
	s.handlerMutex.Lock()
	handler, exists := s.handlers[msgType]
	s.handlerMutex.Unlock()

	if exists {
		handler(msg)
	} else {
		log.Printf("收到未知类型消息: %s %v", msgType, msg)
	}
}

// connectWebSocketAsync 异步连接WebSocket
func (s *BiliBiliLiveService) connectWebSocketAsync(wsAddr, authBody string) {
	// 尝试连接前先将连接状态设为false
	s.setConnected(false)

	// 使用结构化消息回调连接WebSocket
	err := s.client.ConnectWebsocketWithCallback(
		wsAddr,
		authBody,
		func(msgType string, msg interface{}) {
			// 处理消息前检查上下文是否已取消
			select {
			case <-s.ctx.Done():
				return
			default:
				s.process(msgType, msg)
			}
		},
	)

	if err != nil {
		log.Printf("连接WebSocket失败: %v", err)
		return
	}

	// 连接成功，更新状态
	s.setConnected(true)
	log.Println("WebSocket连接成功")

	// 保持WebSocket连接直到上下文取消
	<-s.ctx.Done()
	log.Println("WebSocket连接任务收到退出信号")

	// 连接结束，更新状态
	s.setConnected(false)
}

// 以下是处理不同消息类型的方法
func (s *BiliBiliLiveService) handleDanmu(msg interface{}) {
	dm, ok := msg.(*bilibili.DanmuMessage)
	if !ok {
		log.Println("无效的弹幕消息类型")
		return
	}

	log.Printf("[弹幕] %s: %s", dm.Uname, dm.Msg)

	// 特殊弹幕处理
	switch {
	case dm.DmType == 1:
		log.Printf("  - 这是一个表情包弹幕，表情链接: %s", dm.EmojiImgUrl)
	case dm.GuardLevel > 0:
		guardType := ""
		switch dm.GuardLevel {
		case 1:
			guardType = "总督"
		case 2:
			guardType = "提督"
		case 3:
			guardType = "舰长"
		}
		log.Printf("  - 这是一个%s的弹幕", guardType)
	}

	// 如果是回复弹幕
	if dm.ReplyUname != "" {
		log.Printf("  - 这是回复 %s 的弹幕", dm.ReplyUname)
	}

	// 粉丝勋章信息
	if dm.FansMedalWearingStatus {
		log.Printf("  - 佩戴了 %s %d级 勋章", dm.FansMedalName, dm.FansMedalLevel)
	}

	// 异步处理弹幕内容，例如AI回复
	s.wg.Add(1)
	go s.processMessageAsync(dm)

	// 检查是否包含特定命令
	switch dm.Msg {
	case "!帮助":
		log.Println("收到帮助命令，异步处理...")
		go func() {
			// 模拟异步处理
			time.Sleep(time.Millisecond * 500)
			log.Println("已发送帮助信息")
		}()
	}
}

type DmOperate struct {
	Msg   string `json:"msg,omitempty"`
	Uname string `json:"uname,omitempty"`
	Uface string `json:"uface,omitempty"`
	ModId int    `json:"mod_id,omitempty"`
}

// 异步处理弹幕内容
func (s *BiliBiliLiveService) processMessageAsync(dm *bilibili.DanmuMessage) {
	defer s.wg.Done()
	// 模拟一些耗时处理
	// time.Sleep(time.Millisecond * 100)
	var dmOperate DmOperate
	dmOperate.Uname = dm.Uname
	dmOperate.Uface = dm.UFace
	if strings.HasPrefix(dm.Msg, "@") {
		log.Println("收到@ai命令，异步处理...")
		dmOperate.Msg = dm.Msg[len("@"):]
		utils.WailsApp.EmitEvent("CHAT:QUESTION", dmOperate)
		chatResp, chatErr := s.glmClient.Chat(dmOperate.Msg)
		if chatErr != nil {
			log.Printf("GLM4Voice模型处理失败: %v", chatErr)
			return
		}
		utils.WailsApp.EmitEvent("CHAT:ANSWER", chatResp)
	} else if strings.HasPrefix(dm.Msg, "@V") {
		log.Println("收到@ai命令，异步处理...")
		dmOperate.Msg = dm.Msg[len(""):]
		utils.WailsApp.EmitEvent("CHAT:QUESTION", dmOperate)
		chatResp, chatErr := s.glmClient.Chat(dmOperate.Msg)
		if chatErr != nil {
			log.Printf("GLM4Voice模型处理失败: %v", chatErr)
			return
		}
		utils.WailsApp.EmitEvent("CHAT:ANSWER", chatResp)
	}
	// 这里可以添加实际的AI处理逻辑
	log.Printf("[异步处理] 已处理弹幕: %s", dm.Msg)
}

func (s *BiliBiliLiveService) handleGift(msg interface{}) {
	gift, ok := msg.(*bilibili.SendGiftMessage)
	if !ok {
		return
	}
	log.Printf("[礼物] %s 赠送了 %d 个 %s", gift.Uname, gift.GiftNum, gift.GiftName)
}

func (s *BiliBiliLiveService) handleSuperChat(msg interface{}) {
	sc, ok := msg.(*bilibili.SuperChatMessage)
	if !ok {
		return
	}
	log.Printf("[SC] %s: %s (¥%d)", sc.Uname, sc.Message, sc.Rmb)
}

func (s *BiliBiliLiveService) handleSuperChatDelete(msg interface{}) {
	scDel, ok := msg.(*bilibili.SuperChatDeleteMessage)
	if !ok {
		return
	}
	log.Printf("[SC下线] 房间 %d 的SC被下线", scDel.RoomId)
}

func (s *BiliBiliLiveService) handleGuard(msg interface{}) {
	guard, ok := msg.(*bilibili.GuardMessage)
	if !ok {
		return
	}
	log.Printf("[舰长] %s 购买了舰长", guard.UserInfo.Uname)
}

func (s *BiliBiliLiveService) handleLike(msg interface{}) {
	like, ok := msg.(*bilibili.LikeMessage)
	if !ok {
		return
	}
	log.Printf("[点赞] %s 点赞了 %d 次", like.Uname, like.LikeCount)
}

func (s *BiliBiliLiveService) handleRoomEnter(msg interface{}) {
	enter, ok := msg.(*bilibili.LiveRoomEnterMessage)
	if !ok {
		return
	}
	log.Printf("[进入] %s 进入了直播间", enter.Uname)
}

func (s *BiliBiliLiveService) handleLiveStart(msg interface{}) {
	start, ok := msg.(*bilibili.LiveStartMessage)
	if !ok {
		return
	}
	log.Printf("[开播] 主播开始直播了，分区: %s", start.AreaName)
}

func (s *BiliBiliLiveService) handleLiveEnd(msg interface{}) {
	end, ok := msg.(*bilibili.LiveEndMessage)
	if !ok {
		return
	}
	log.Printf("[下播] 主播结束了直播，分区: %s", end.AreaName)
}

func (s *BiliBiliLiveService) handleInteractionEnd(msg interface{}) {
	end, ok := msg.(*bilibili.InteractionEndMessage)
	if !ok {
		return
	}
	log.Printf("[互动结束] 游戏 %s 的互动已结束", end.GameId)
}
