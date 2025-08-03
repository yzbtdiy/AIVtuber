// 基础用户信息
export interface UserInfo {
  uid: number; // 已废弃，固定为0
  open_id: string; // 用户唯一标识
  union_id?: string; // 用户在同一个开发者下的唯一标识(默认为空)
  uname: string; // 用户昵称
  uface: string; // 用户头像
}

// 主播信息
export interface AnchorInfo {
  uid: number; // 收礼主播uid
  open_id: string; // 收礼主播唯一标识
  union_id?: string; // 用户在同一个开发者下的唯一标识
  uname: string; // 收礼主播昵称
  uface: string; // 收礼主播头像
}

// 连击信息
export interface ComboInfo {
  combo_base_num: number; // 每次连击赠送的道具数量
  combo_count: number; // 连击次数
  combo_id: string; // 连击id
  combo_timeout: number; // 连击有效期秒
}

// 盲盒信息
export interface BlindGift {
  blind_gift_id: number; // 盲盒id
  status: boolean; // 是否是盲盒
}

// 弹幕消息
export interface DanmakuMessage {
  uname: string; // 用户昵称
  uid: number; // 用户UID（已废弃，固定为0）
  open_id: string; // 用户唯一标识
  union_id?: string; // 用户在同一个开发者下的唯一标识
  uface: string; // 用户头像
  timestamp: number; // 弹幕发送时间秒级时间戳
  room_id: number; // 弹幕接收的直播间
  msg: string; // 弹幕内容
  msg_id: string; // 消息唯一id
  guard_level: number; // 对应房间大航海等级
  fans_medal_wearing_status: boolean; // 该房间粉丝勋章佩戴情况
  fans_medal_name: string; // 粉丝勋章名
  fans_medal_level: number; // 对应房间勋章信息
  emoji_img_url: string; // 表情包图片地址
  dm_type: number; // 弹幕类型 0：普通弹幕 1：表情包弹幕
  glory_level: number; // 直播荣耀等级
  reply_open_id: string; // 被at用户唯一标识
  reply_uname: string; // 被at的用户昵称
  is_admin: number; // 发送弹幕的用户是否是房管，0或1
}

// 礼物消息
export interface GiftMessage {
  room_id: number; // 房间号
  uid: number; // 用户UID（已废弃，固定为0）
  open_id: string; // 用户唯一标识
  union_id?: string; // 用户在同一个开发者下的唯一标识
  uname: string; // 送礼用户昵称
  uface: string; // 送礼用户头像
  gift_id: number; // 道具id(盲盒:爆出道具id)
  gift_name: string; // 道具名(盲盒:爆出道具名)
  gift_num: number; // 赠送道具数量
  price: number; // 礼物爆出单价，(1000 = 1元 = 10电池)
  r_price: number; // 实际价值(1000 = 1元 = 10电池)
  paid: boolean; // 是否是付费道具
  fans_medal_level: number; // 实际送礼人的勋章信息
  fans_medal_name: string; // 粉丝勋章名
  fans_medal_wearing_status: boolean; // 该房间粉丝勋章佩戴情况
  guard_level: number; // 大航海等级
  timestamp: number; // 收礼时间秒级时间戳
  anchor_info: AnchorInfo; // 主播信息
  msg_id: string; // 消息唯一id
  gift_icon: string; // 道具icon
  combo_gift: boolean; // 是否是combo道具
  combo_info: ComboInfo; // 连击信息
  blind_gift: BlindGift; // 盲盒信息
}

// 付费留言消息
export interface SuperChatMessage {
  room_id: number; // 直播间id
  uid: number; // 用户UID（已废弃，固定为0）
  open_id: string; // 用户唯一标识
  union_id?: string; // 用户在同一个开发者下的唯一标识
  uname: string; // 购买的用户昵称
  uface: string; // 购买用户头像
  message_id: number; // 留言id(风控场景下撤回留言需要)
  message: string; // 留言内容
  rmb: number; // 支付金额(元)
  timestamp: number; // 赠送时间秒级
  start_time: number; // 生效开始时间
  end_time: number; // 生效结束时间
  guard_level: number; // 对应房间大航海等级
  fans_medal_level: number; // 对应房间勋章信息
  fans_medal_name: string; // 对应房间勋章名字
  fans_medal_wearing_status: boolean; // 该房间粉丝勋章佩戴情况
  msg_id: string; // 消息唯一id
}

// 付费留言下线消息
export interface SuperChatDelMessage {
  room_id: number; // 直播间id
  message_ids: number[]; // 留言id
  msg_id: string; // 消息唯一id
}

// 大航海消息
export interface GuardMessage {
  user_info: UserInfo; // 用户信息
  guard_level: number; // 大航海等级
  guard_num: number; // 大航海数量
  guard_unit: string; // 大航海单位
  price: number; // 大航海金瓜子
  fans_medal_level: number; // 粉丝勋章等级
  fans_medal_name: string; // 粉丝勋章名
  fans_medal_wearing_status: boolean; // 该房间粉丝勋章佩戴情况
  room_id: number; // 房间号
  msg_id: string; // 消息唯一id
  timestamp: number; // 上舰时间秒级时间戳
}

// 点赞消息
export interface LikeMessage {
  uname: string; // 用户昵称
  uid: number; // 用户UID（已废弃，固定为0）
  open_id: string; // 用户唯一标识
  union_id?: string; // 用户在同一个开发者下的唯一标识
  uface: string; // 用户头像
  timestamp: number; // 时间秒级时间戳
  room_id: number; // 发生的直播间
  like_text: string; // 点赞文案
  like_count: number; // 对单个用户最近2秒的点赞次数聚合
  fans_medal_wearing_status: boolean; // 该房间粉丝勋章佩戴情况
  fans_medal_name: string; // 粉丝勋章名
  fans_medal_level: number; // 对应房间勋章信息
  msg_id: string; // 消息唯一id
}

// 进入房间消息
export interface RoomEnterMessage {
  room_id: number; // 发生的直播间
  uface: string; // 用户头像
  uname: string; // 用户昵称
  open_id: string; // 用户唯一标识
  union_id?: string; // 用户在同一个开发者下的唯一标识
  timestamp: number; // 发生的时间戳
}

// 开始直播消息
export interface LiveStartMessage {
  room_id: number; // 发生的直播间
  open_id: string; // 用户唯一标识
  union_id?: string; // 用户在同一个开发者下的唯一标识
  timestamp: number; // 发生的时间戳
  area_name: string; // 开播二级分区名称
  title: string; // 开播时刻，直播间的标题
}

// 结束直播消息
export interface LiveEndMessage {
  room_id: number; // 发生的直播间
  open_id: string; // 用户唯一标识
  union_id?: string; // 用户在同一个开发者下的唯一标识
  timestamp: number; // 发生的时间戳
  area_name: string; // 开播二级分区名称
  title: string; // 开播时刻，直播间的标题
}

// 消息推送结束通知
export interface InteractionEndMessage {
  game_id: string; // 结束消息推送的game_id
  timestamp: number; // 发生的时间戳
}

// 消息类型枚举
export enum LivePlatformCmd {
  DM = 'LIVE_OPEN_PLATFORM_DM',
  SEND_GIFT = 'LIVE_OPEN_PLATFORM_SEND_GIFT',
  SUPER_CHAT = 'LIVE_OPEN_PLATFORM_SUPER_CHAT',
  SUPER_CHAT_DEL = 'LIVE_OPEN_PLATFORM_SUPER_CHAT_DEL',
  GUARD = 'LIVE_OPEN_PLATFORM_GUARD',
  LIKE = 'LIVE_OPEN_PLATFORM_LIKE',
  LIVE_ROOM_ENTER = 'LIVE_OPEN_PLATFORM_LIVE_ROOM_ENTER',
  LIVE_START = 'LIVE_OPEN_PLATFORM_LIVE_START',
  LIVE_END = 'LIVE_OPEN_PLATFORM_LIVE_END',
  INTERACTION_END = 'LIVE_OPEN_PLATFORM_INTERACTION_END'
}

// 通用消息结构
export interface LivePlatformMessage<T = any> {
  cmd: LivePlatformCmd;
  data: T;
}

// 所有消息类型的联合类型
export type BilibiliLiveMessage =
  | LivePlatformMessage<DanmakuMessage>
  | LivePlatformMessage<GiftMessage>
  | LivePlatformMessage<SuperChatMessage>
  | LivePlatformMessage<SuperChatDelMessage>
  | LivePlatformMessage<GuardMessage>
  | LivePlatformMessage<LikeMessage>
  | LivePlatformMessage<RoomEnterMessage>
  | LivePlatformMessage<LiveStartMessage>
  | LivePlatformMessage<LiveEndMessage>
  | LivePlatformMessage<InteractionEndMessage>;

