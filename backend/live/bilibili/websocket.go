package bilibili

import (
	"bytes"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

const (
	MaxBodySize     = int32(1 << 11)
	CmdSize         = 4
	PackSize        = 4
	HeaderSize      = 2
	VerSize         = 2
	OperationSize   = 4
	SeqIdSize       = 4
	HeartbeatSize   = 4
	RawHeaderSize   = PackSize + HeaderSize + VerSize + OperationSize + SeqIdSize
	MaxPackSize     = MaxBodySize + int32(RawHeaderSize)
	PackOffset      = 0
	HeaderOffset    = PackOffset + PackSize
	VerOffset       = HeaderOffset + VerSize
	OperationOffset = VerOffset + VerSize
	SeqIdOffset     = OperationOffset + OperationSize
	HeartbeatOffset = SeqIdOffset + SeqIdSize
)

const (
	OP_HEARTBEAT       = int32(2)
	OP_HEARTBEAT_REPLY = int32(3)
	OP_SEND_SMS_REPLY  = int32(5)
	OP_AUTH            = int32(7)
	OP_AUTH_REPLY      = int32(8)
)

// WebsocketClient WebSocket客户端
type WebsocketClient struct {
	conn       *websocket.Conn
	msgBuf     chan *Proto
	sequenceId int32
	dispather  map[int32]protoLogic
	authed     bool
	msgHandler func([]byte) // 外部消息处理函数
	stopChan   chan struct{}
}

type protoLogic func(p *Proto) (err error)

// Proto WebSocket协议
type Proto struct {
	PacketLength int32
	HeaderLength int16
	Version      int16
	Operation    int32
	SequenceId   int32
	Body         []byte
	BodyMuti     [][]byte
}

// NewWebsocketClient 创建新的WebSocket客户端
func NewWebsocketClient(wsAddr, authBody string, msgHandler func([]byte)) (*WebsocketClient, error) {
	// 建立连接
	conn, _, err := websocket.DefaultDialer.Dial(wsAddr, nil)
	if err != nil {
		return nil, fmt.Errorf("连接WebSocket失败: %w", err)
	}

	wc := &WebsocketClient{
		conn:       conn,
		msgBuf:     make(chan *Proto, 1024),
		dispather:  make(map[int32]protoLogic),
		msgHandler: msgHandler,
		stopChan:   make(chan struct{}),
	}

	// 注册分发处理函数
	wc.dispather[OP_AUTH_REPLY] = wc.authResp
	wc.dispather[OP_HEARTBEAT_REPLY] = wc.heartBeatResp
	wc.dispather[OP_SEND_SMS_REPLY] = wc.msgResp

	// 发送鉴权信息
	err = wc.sendAuth(authBody)
	if err != nil {
		conn.Close()
		return nil, fmt.Errorf("发送鉴权信息失败: %w", err)
	}

	// 读取信息
	go wc.ReadMsg()

	// 处理信息
	go wc.DoEvent()

	return wc, nil
}

// ReadMsg 读取长连信息
func (wc *WebsocketClient) ReadMsg() {
	for {
		select {
		case <-wc.stopChan:
			return
		default:
			retProto := &Proto{}
			_, buf, err := wc.conn.ReadMessage()
			if err != nil {
				log.Printf("读取WebSocket消息失败: %v", err)
				continue
			}

			if len(buf) < int(RawHeaderSize) {
				log.Println("收到无效数据包")
				continue
			}

			retProto.PacketLength = int32(binary.BigEndian.Uint32(buf[PackOffset:HeaderOffset]))
			retProto.HeaderLength = int16(binary.BigEndian.Uint16(buf[HeaderOffset:VerOffset]))
			retProto.Version = int16(binary.BigEndian.Uint16(buf[VerOffset:OperationOffset]))
			retProto.Operation = int32(binary.BigEndian.Uint32(buf[OperationOffset:SeqIdOffset]))
			retProto.SequenceId = int32(binary.BigEndian.Uint32(buf[SeqIdOffset:]))

			if retProto.PacketLength < 0 || retProto.PacketLength > MaxPackSize {
				log.Println("无效的包长度")
				continue
			}

			if retProto.HeaderLength != RawHeaderSize {
				log.Println("无效的头部长度")
				continue
			}

			if bodyLen := int(retProto.PacketLength - int32(retProto.HeaderLength)); bodyLen > 0 {
				retProto.Body = buf[retProto.HeaderLength:retProto.PacketLength]
			} else {
				log.Println("无效的消息体长度")
				continue
			}

			retProto.BodyMuti = [][]byte{retProto.Body}
			if len(retProto.BodyMuti) > 0 {
				retProto.Body = retProto.BodyMuti[0]
			}

			select {
			case wc.msgBuf <- retProto:
			case <-wc.stopChan:
				return
			}
		}
	}
}

// DoEvent 处理信息
func (wc *WebsocketClient) DoEvent() {
	ticker := time.NewTicker(time.Second * 20)
	defer ticker.Stop()

	for {
		select {
		case <-wc.stopChan:
			return
		case p := <-wc.msgBuf:
			if p == nil {
				continue
			}
			if handler := wc.dispather[p.Operation]; handler != nil {
				if err := handler(p); err != nil {
					log.Printf("处理消息失败: %v", err)
				}
			}
		case <-ticker.C:
			wc.sendWsHeartBeat()
		}
	}
}

// sendAuth 发送鉴权
func (wc *WebsocketClient) sendAuth(authBody string) (err error) {
	p := &Proto{
		Operation: OP_AUTH,
		Body:      []byte(authBody),
	}
	return wc.sendMsg(p)
}

// sendWsHeartBeat 发送WebSocket心跳
func (wc *WebsocketClient) sendWsHeartBeat() {
	if !wc.authed {
		return
	}
	msg := &Proto{}
	msg.Operation = OP_HEARTBEAT
	msg.SequenceId = wc.sequenceId
	wc.sequenceId++
	err := wc.sendMsg(msg)
	if err != nil {
		log.Printf("发送WebSocket心跳失败: %v", err)
		return
	}
	// 优化心跳日志，使用更简洁的格式
	// if wc.sequenceId%10 == 0 { // 每10次心跳只输出一次日志，避免日志过多
	// 	log.Printf("WebSocket心跳发送 [seq:%d]", msg.SequenceId)
	// }
}

// sendMsg 发送信息
func (wc *WebsocketClient) sendMsg(msg *Proto) (err error) {
	select {
	case <-wc.stopChan:
		return fmt.Errorf("客户端已关闭")
	default:
		dataBuff := &bytes.Buffer{}
		packLen := int32(RawHeaderSize + len(msg.Body))
		msg.HeaderLength = RawHeaderSize
		binary.Write(dataBuff, binary.BigEndian, packLen)
		binary.Write(dataBuff, binary.BigEndian, int16(RawHeaderSize))
		binary.Write(dataBuff, binary.BigEndian, msg.Version)
		binary.Write(dataBuff, binary.BigEndian, msg.Operation)
		binary.Write(dataBuff, binary.BigEndian, msg.SequenceId)
		binary.Write(dataBuff, binary.BigEndian, msg.Body)
		err = wc.conn.WriteMessage(websocket.BinaryMessage, dataBuff.Bytes())
		if err != nil {
			log.Println("[WebsocketClient | sendMsg] send msg err:", err)
			return
		}
		return
	}
}

// authResp 鉴权处理函数
func (wc *WebsocketClient) authResp(msg *Proto) (err error) {
	resp := &AuthRespParam{}
	err = json.Unmarshal(msg.Body, resp)
	if err != nil {
		log.Printf("解析鉴权响应失败: %v", err)
		return
	}
	if resp.Code != 0 {
		return fmt.Errorf("鉴权失败，返回码: %d", resp.Code)
	}
	wc.authed = true
	log.Println("[WebsocketClient | authResp] auth success")
	return
}

// heartBeatResp WebSocket心跳响应处理
func (wc *WebsocketClient) heartBeatResp(msg *Proto) (err error) {
	// 优化心跳响应日志，只在需要时输出
	if wc.sequenceId%10 == 0 { // 与发送保持一致，每10次才输出
		log.Printf("WebSocket心跳响应 [seq:%d] 已收到", msg.SequenceId)
	}
	return
}

// msgResp 消息接收处理函数
func (wc *WebsocketClient) msgResp(msg *Proto) (err error) {
	for _, cmd := range msg.BodyMuti {
		if wc.msgHandler != nil {
			wc.msgHandler(cmd)
		} else {
			log.Printf("[WebsocketClient | msgResp] recv MsgResp: %s", string(cmd))
		}
	}
	return
}

// Close 关闭WebSocket连接和相关资源
func (wc *WebsocketClient) Close() error {
	select {
	case <-wc.stopChan:
		return nil // 已经关闭
	default:
		close(wc.stopChan)
		return wc.conn.Close()
	}
}

// MessageHandler 消息处理函数类型
type MessageHandler func([]byte)

// MessageProcessor 实现消息解析和处理
type MessageProcessor struct {
	handler MessageHandler
}

// NewMessageProcessor 创建消息处理器
func NewMessageProcessor(handler MessageHandler) *MessageProcessor {
	return &MessageProcessor{
		handler: handler,
	}
}

// Process 处理消息
func (mp *MessageProcessor) Process(data []byte) error {
	if mp.handler != nil {
		mp.handler(data)
	}
	return nil
}

type LiveMessage struct {
	Cmd  string          `json:"cmd"`
	Data json.RawMessage `json:"data"`
}

// DecodeBilibiliMsg 解析B站消息
func DecodeBilibiliMsg(data []byte) (interface{}, error) {
	// log.Println("[DecodeBilibiliMsg] data:", string(data))
	var msg = &LiveMessage{}
	err := json.Unmarshal(data, &msg)
	if err != nil {
		return nil, fmt.Errorf("解析消息失败: %w", err)
	}

	// 判断消息类型
	cmd := msg.Cmd
	switch {
	case cmd == "LIVE_OPEN_PLATFORM_DM": // 弹幕消息
		return DecodeDanmuMsg(msg.Data)
	case cmd == "LIVE_OPEN_PLATFORM_SEND_GIFT": // 礼物消息
		return DecodeGiftMsg(msg.Data)
	case cmd == "LIVE_OPEN_PLATFORM_SUPER_CHAT": // 欢迎消息
		return DecodeSuperChatMsg(msg.Data)
	case cmd == "LIVE_OPEN_PLATFORM_SUPER_CHAT_DEL": // SC消息
		return DecodeSuperChatMsgDelete(msg.Data)
	case cmd == "GuardMessage": // 礼物消息
		return DecodeGuardMsg(msg.Data)
	case cmd == "LIVE_OPEN_PLATFORM_LIKE": // 点赞消息
		return DecodeLikeMsg(msg.Data)
	case cmd == "LIVE_OPEN_PLATFORM_LIVE_ROOM_ENTER": // 互动结束消息
		return DecodeLiveRoomEnterMsg(msg.Data)
	case cmd == "LIVE_OPEN_PLATFORM_LIVE_START": // 进入房间消息
		return DecodeLiveStartMsg(msg.Data)
	case cmd == "LIVE_OPEN_PLATFORM_LIVE_END": // 欢迎消息
		return DecodeLiveEndMsg(msg.Data)
	case cmd == "LIVE_OPEN_PLATFORM_INTERACTION_END": // 互动结束消息
		return DecodeInteractionEndMsg(msg.Data)
	default:
		// 其他消息类型
		return msg, nil
	}

}

// DecodeDanmuMsg 解析弹幕消息
func DecodeDanmuMsg(data json.RawMessage) (*DanmuMessage, error) {
	var danmu = &DanmuMessage{}
	err := json.Unmarshal(data, danmu)
	// log.Println("[DecodeDanmuMsg] danmu!!!!!!!:", danmu)
	if err != nil {
		return danmu, err
	} else {
		return danmu, nil
	}
}

// DecodeGiftMsg 解析礼物消息
func DecodeGiftMsg(data json.RawMessage) (*SendGiftMessage, error) {
	gift := &SendGiftMessage{}
	err := json.Unmarshal(data, gift)
	if err != nil {
		return gift, err
	} else {
		return gift, nil
	}
}

// DecodeSuperChatMsg 解析醒目留言消息
func DecodeSuperChatMsg(data json.RawMessage) (*SuperChatMessage, error) {
	sc := &SuperChatMessage{}
	err := json.Unmarshal(data, sc)
	if err != nil {
		return sc, err
	} else {
		return sc, nil
	}
}

func DecodeSuperChatMsgDelete(data json.RawMessage) (*SuperChatDeleteMessage, error) {
	scDel := &SuperChatDeleteMessage{}
	err := json.Unmarshal(data, scDel)
	if err != nil {
		return scDel, err
	} else {
		return scDel, nil
	}
}

func DecodeGuardMsg(data json.RawMessage) (*GuardMessage, error) {
	guard := &GuardMessage{}
	err := json.Unmarshal(data, guard)
	if err != nil {
		return guard, err
	} else {
		return guard, nil
	}
}

func DecodeLikeMsg(data json.RawMessage) (*LikeMessage, error) {
	like := &LikeMessage{}
	err := json.Unmarshal(data, like)
	if err != nil {
		return like, err
	} else {
		return like, nil
	}
}

func DecodeLiveRoomEnterMsg(data json.RawMessage) (*LiveRoomEnterMessage, error) {
	liveEnter := &LiveRoomEnterMessage{}
	err := json.Unmarshal(data, liveEnter)
	if err != nil {
		return liveEnter, err
	} else {
		return liveEnter, nil
	}
}

func DecodeLiveStartMsg(data json.RawMessage) (*LiveStartMessage, error) {
	liveStart := &LiveStartMessage{}
	err := json.Unmarshal(data, liveStart)
	if err != nil {
		return liveStart, err
	} else {
		return liveStart, nil
	}
}

func DecodeLiveEndMsg(data json.RawMessage) (*LiveEndMessage, error) {
	liveEnd := &LiveEndMessage{}
	err := json.Unmarshal(data, liveEnd)
	if err != nil {
		return liveEnd, err
	} else {
		return liveEnd, nil
	}
}

func DecodeInteractionEndMsg(data json.RawMessage) (*InteractionEndMessage, error) {
	end := &InteractionEndMessage{}
	err := json.Unmarshal(data, end)
	if err != nil {
		return end, err
	} else {
		return end, nil
	}
}
