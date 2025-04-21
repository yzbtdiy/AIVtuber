package bilibili

import (
	"encoding/json"
)

// StartAppRequest 启动应用请求
type StartAppRequest struct {
	// 主播身份码
	Code string `json:"code"`
	// 项目id
	AppId int64 `json:"app_id"`
}

// StartAppRespData 启动应用响应数据
type StartAppRespData struct {
	// 场次信息
	GameInfo GameInfo `json:"game_info"`
	// 长连信息
	WebsocketInfo WebSocketInfo `json:"websocket_info"`
	// 主播信息
	AnchorInfo AnchorInfo `json:"anchor_info"`
}

// GameInfo 游戏信息
type GameInfo struct {
	GameId string `json:"game_id"`
}

// WebSocketInfo WebSocket信息
type WebSocketInfo struct {
	//  长连使用的请求json体 第三方无需关注内容,建立长连时使用即可
	AuthBody string `json:"auth_body"`
	//  wss 长连地址
	WssLink []string `json:"wss_link"`
}

// AnchorInfo 主播信息
type AnchorInfo struct {
	//主播房间号
	RoomId int64 `json:"room_id"`
	//主播昵称
	Uname string `json:"uname"`
	//主播头像
	Uface string `json:"uface"`
	//主播uid
	Uid int64 `json:"uid"`
	//主播open_id
	OpenId string `json:"open_id"`
}

// EndAppRequest 关闭应用请求
type EndAppRequest struct {
	// 场次id
	GameId string `json:"game_id"`
	// 项目id
	AppId int64 `json:"app_id"`
}

// AppHeartbeatReq 应用心跳请求
type AppHeartbeatReq struct {
	// 主播身份码
	GameId string `json:"game_id"`
}

// BaseResp API响应基础结构
type BaseResp struct {
	Code      int64           `json:"code"`
	Message   string          `json:"message"`
	RequestId string          `json:"request_id"`
	Data      json.RawMessage `json:"data"`
}

// Auth响应参数
type AuthRespParam struct {
	Code int64 `json:"code,omitempty"`
}

// CommonHeader HTTP请求头
type CommonHeader struct {
	ContentType       string
	ContentAcceptType string
	Timestamp         string
	SignatureMethod   string
	SignatureVersion  string
	Authorization     string
	Nonce             string
	AccessKeyId       string
	ContentMD5        string
}
