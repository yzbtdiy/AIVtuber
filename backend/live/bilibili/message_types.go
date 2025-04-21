package bilibili

// 获取弹幕信息 LIVE_OPEN_PLATFORM_DM
type DanmuMessage struct {
	Uname                  string `json:"uname"`                     // 用户昵称
	Uid                    int64  `json:"uid"`                       // 用户UID（已废弃，固定为0）
	OpenId                 string `json:"open_id"`                   // 用户唯一标识
	UFace                  string `json:"uface"`                     // 用户头像
	Timestamp              int64  `json:"timestamp"`                 // 弹幕发送时间秒级时间戳
	RoomId                 int64  `json:"room_id"`                   // 弹幕接收的直播间
	Msg                    string `json:"msg"`                       // 弹幕内容
	MsgId                  string `json:"msg_id"`                    // 消息唯一id
	GuardLevel             int64  `json:"guard_level"`               // 对应房间大航海等级
	FansMedalWearingStatus bool   `json:"fans_medal_wearing_status"` // 该房间粉丝勋章佩戴情况
	FansMedalName          string `json:"fans_medal_name"`           // 粉丝勋章名
	FansMedalLevel         int64  `json:"fans_medal_level"`          // 对应房间勋章信息
	EmojiImgUrl            string `json:"emoji_img_url"`             // 表情包图片地址
	DmType                 int64  `json:"dm_type"`                   // 弹幕类型 0：普通弹幕 1：表情包弹幕
	GloryLevel             int    `json:"glory_level"`               //直播荣耀等级
	ReplyOpenId            string `json:"reply_open_id"`             //被at用户唯一标识
	ReplyUname             string `json:"reply_uname"`               //被at的用户昵称
	IsAdmin                int    `json:"is_admin"`                  //发送弹幕的用户是否是房管，取值范围0或1，取值为1时是房管
}

type AnchorInfor struct {
	Uid    int64  `json:"uid"`     // 收礼主播uid
	OpenId string `json:"open_id"` // 收礼主播唯一标识(2024-03-11后上线)
	Uname  string `json:"uname"`   // 收礼主播昵称
	Uface  string `json:"uface"`   // 收礼主播头像
}

type ComboInfo struct {
	ComboBaseNum int64  `json:"combo_base_num"` // 每次连击赠送的道具数量
	ComboCount   int64  `json:"combo_count"`    // 连击次数
	ComboId      string `json:"combo_id"`       // 连击id
	ComboTimeout int64  `json:"combo_timeout"`  // 连击有效期秒
}

// 获取礼物信息 LIVE_OPEN_PLATFORM_SEND_GIFT
type SendGiftMessage struct {
	RoomId                 int64       `json:"room_id"`                   // 房间号
	Uid                    int64       `json:"uid"`                       // 用户UID（已废弃，固定为0）
	OpenId                 string      `json:"open_id"`                   // 用户唯一标识
	Uname                  string      `json:"uname"`                     // 送礼用户昵称
	UFace                  string      `json:"uface"`                     // 送礼用户头像
	GiftID                 int64       `json:"gift_id"`                   // 道具id(盲盒:爆出道具id)
	GiftName               string      `json:"gift_name"`                 // 道具名(盲盒:爆出道具名)
	GiftNum                int64       `json:"gift_num"`                  // 赠送道具数量
	Price                  int64       `json:"price"`                     // 礼物爆出单价，(1000 = 1元 = 10电池),盲盒:爆出道具的价值
	Rprice                 int64       `json:"rprice"`                    // 实际价值(1000 = 1元 = 10电池),盲盒:爆出道具的实际价值
	Paid                   bool        `json:"paid"`                      // 是否是付费道具
	FansMedalLevel         int64       `json:"fans_medal_level"`          // 实际送礼人的勋章信息
	FansMedalName          string      `json:"fans_medal_name"`           // 粉丝勋章名
	FansMedalWearingStatus bool        `json:"fans_medal_wearing_status"` // 该房间粉丝勋章佩戴情况
	GuardLevel             int64       `json:"guard_level"`               // 大航海等级
	Timestamp              int64       `json:"timestamp"`                 // 收礼时间秒级时间戳
	AnchorInfor            AnchorInfor `json:"anchor_info"`               // 主播信息
	MsgId                  string      `json:"msg_id"`                    // 消息唯一id
	GiftIcon               string      `json:"gift_icon"`                 // 道具icon
	ComboGift              bool        `json:"combo_gift"`                // 是否是combo道具
	ComboInfo              ComboInfo   `json:"combo_info"`                // 连击信息
}

// 获取付费留言 LIVE_OPEN_PLATFORM_SUPER_CHAT
type SuperChatMessage struct {
	RoomId                 int64  `json:"room_id"`                   // 直播间id
	Uid                    int64  `json:"uid"`                       // 用户UID（已废弃，固定为0）
	OpenId                 string `json:"open_id"`                   // 用户唯一标识
	Uname                  string `json:"uname"`                     // 购买的用户昵称
	Uface                  string `json:"uface"`                     // 购买用户头像
	MessageId              int64  `json:"message_id"`                // 留言id(风控场景下撤回留言需要)
	Message                string `json:"message"`                   // 留言内容
	Rmb                    int64  `json:"rmb"`                       // 支付金额(元)
	Timestamp              int64  `json:"timestamp"`                 // 赠送时间秒级
	StartTime              int64  `json:"start_time"`                // 生效开始时间
	EndTime                int64  `json:"end_time"`                  // 生效结束时间
	GuardLevel             int64  `json:"guard_level"`               // 对应房间大航海等级
	FansMedalLevel         int64  `json:"fans_medal_level"`          // 对应房间勋章信息
	FansMedalName          string `json:"fans_medal_name"`           // 对应房间勋章名字
	FansMedalWearingStatus bool   `json:"fans_medal_wearing_status"` // 该房间粉丝勋章佩戴情况
	MsgId                  string `json:"msg_id"`                    // 消息唯一id
}

// 付费留言下线 LIVE_OPEN_PLATFORM_SUPER_CHAT_DEL
type SuperChatDeleteMessage struct {
	RoomId     int64   `json:"room_id"`     // 直播间id
	MessageIds []int64 `json:"message_ids"` // 留言id
	MsgId      string  `json:"msg_id"`      // 消息唯一id
}

type GuardUserInfo struct {
	Uid    int64  `json:"uid"`     // 用户UID（已废弃，固定为0）
	OpenId string `json:"open_id"` // 用户唯一标识
	Uname  string `json:"uname"`   // 用户昵称
	Uface  string `json:"uface"`   // 用户头像
}

// 付费大航海 LIVE_OPEN_PLATFORM_GUARD
type GuardMessage struct {
	UserInfo               GuardUserInfo `json:"user_info"`                 // 用户信息
	GuardLevel             int64         `json:"guard_level"`               // 大航海等级
	GuardNum               int64         `json:"guard_num"`                 // 大航海数量
	GuardUnit              string        `json:"guard_unit"`                // 大航海单位
	Price                  int64         `json:"price"`                     // 大航海金瓜子
	FansMedalLevel         int64         `json:"fans_medal_level"`          // 粉丝勋章等级
	FansMedalName          string        `json:"fans_medal_name"`           // 粉丝勋章名
	FansMedalWearingStatus bool          `json:"fans_medal_wearing_status"` // 该房间粉丝勋章佩戴情况
	RoomId                 int64         `json:"room_id"`                   // 房间号
	MsgId                  string        `json:"msg_id"`                    // 消息唯一id
	Timestamp              int64         `json:"timestamp"`                 // 上舰时间秒级时间戳
}

// 点赞信息 LIVE_OPEN_PLATFORM_LIKE
type LikeMessage struct {
	Uname                  string `json:"uname"`                     // 用户昵称
	Uid                    int64  `json:"uid"`                       // 用户UID（已废弃，固定为0）
	OpenId                 string `json:"open_id"`                   // 用户唯一标识
	Uface                  string `json:"uface"`                     // 用户头像
	Timestamp              int64  `json:"timestamp"`                 // 时间秒级时间戳
	RoomId                 int64  `json:"room_id"`                   // 发生的直播间
	LikeText               string `json:"like_text"`                 // 点赞文案( “xxx点赞了”)
	LikeCount              int64  `json:"like_count"`                // 对单个用户最近2秒的点赞次数聚合
	FansMedalWearingStatus bool   `json:"fans_medal_wearing_status"` // 该房间粉丝勋章佩戴情况
	FansMedalName          string `json:"fans_medal_name"`           // 粉丝勋章名
	FansMedalLevel         int64  `json:"fans_medal_level"`          // 对应房间勋章信息
}

// 进入房间 LIVE_OPEN_PLATFORM_LIVE_ROOM_ENTER
type LiveRoomEnterMessage struct {
	RoomId    int64  `json:"room_id"`   // 发生的直播间
	UFace     string `json:"uface"`     // 用户头像
	Uname     string `json:"uname"`     // 用户昵称
	OpenId    string `json:"open_id"`   // 用户唯一标识
	Timestamp int64  `json:"timestamp"` // 发生的时间戳
}

// 开始直播 LIVE_OPEN_PLATFORM_LIVE_START
type LiveStartMessage struct {
	RoomId    int64  `json:"room_id"`   // 发生的直播间
	OpenId    string `json:"open_id"`   // 用户唯一标识
	Timestamp int64  `json:"timestamp"` // 发生的时间戳
	AreaName  string `json:"area_name"` // 开播二级分区名称
	Title     int64  `json:"title"`     // 开播即刻, 直播间的标题
}

// 结束直播 LIVE_OPEN_PLATFORM_LIVE_END
type LiveEndMessage struct {
	RoomId    int64  `json:"room_id"`   // 发生的直播间
	OpenId    string `json:"open_id"`   // 用户唯一标识
	Timestamp int64  `json:"timestamp"` // 发生的时间戳
	AreaName  string `json:"area_name"` // 开播二级分区名称
	Title     int64  `json:"title"`     // 开播即刻, 直播间的标题
}

// 消息推送结束通知 LIVE_OPEN_PLATFORM_INTERACTION_END
type InteractionEndMessage struct {
	GameId    string `json:"game_id"`   // 结束消息推送的game_id
	Timestamp int64  `json:"timestamp"` // 发生的时间戳
}
