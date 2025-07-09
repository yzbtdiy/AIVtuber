use serde::{Deserialize, Serialize};
use std::io::Read;
use flate2::read::ZlibDecoder;
use serde_json::Value;

const HEADER_LENGTH: usize = 16;
const MAX_BODY_SIZE: usize = 1024 * 1024; // 增加到1MB以支持更大的消息

#[derive(Debug, Clone)]
pub struct Proto {
    pub packet_len: u32,
    pub header_len: u16,
    pub ver: u16,
    pub op: u32,
    pub seq: u32,
    pub body: Vec<u8>,
}

impl Proto {
    pub fn new() -> Self {
        Proto {
            packet_len: 0,
            header_len: HEADER_LENGTH as u16,
            ver: 0,
            op: 0,
            seq: 0,
            body: Vec::new(),
        }
    }

    pub fn pack(&mut self) -> Vec<u8> {
        self.packet_len = (self.body.len() + HEADER_LENGTH) as u32;
        
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.packet_len.to_be_bytes());
        buf.extend_from_slice(&self.header_len.to_be_bytes());
        buf.extend_from_slice(&self.ver.to_be_bytes());
        buf.extend_from_slice(&self.op.to_be_bytes());
        buf.extend_from_slice(&self.seq.to_be_bytes());
        buf.extend_from_slice(&self.body);
        
        buf
    }

    pub fn unpack(&mut self, buf: &[u8]) -> Result<(), String> {
        if buf.len() < HEADER_LENGTH {
            return Err("包头不够".to_string());
        }

        self.packet_len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        self.header_len = u16::from_be_bytes([buf[4], buf[5]]);
        self.ver = u16::from_be_bytes([buf[6], buf[7]]);
        self.op = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        self.seq = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);

        if self.packet_len as usize > MAX_BODY_SIZE {
            return Err(format!("包体长度不对: {}", self.packet_len));
        }

        if self.header_len as usize != HEADER_LENGTH {
            return Err("包头长度不对".to_string());
        }

        let body_len = self.packet_len as usize - HEADER_LENGTH;
        if body_len > 0 {
            if buf.len() < self.packet_len as usize {
                return Err("数据不完整".to_string());
            }
            self.body = buf[HEADER_LENGTH..self.packet_len as usize].to_vec();
        }

        Ok(())
    }

    pub fn get_body_string(&self) -> Result<String, String> {
        match self.ver {
            0 => {
                // 未压缩的数据
                String::from_utf8(self.body.clone())
                    .map_err(|e| format!("解码UTF-8失败: {}", e))
            }
            2 => {
                // zlib压缩的数据
                let mut decoder = ZlibDecoder::new(&self.body[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)
                    .map_err(|e| format!("解压缩失败: {}", e))?;
                
                // 解压缩后的数据可能包含多个消息包，需要递归处理
                if decompressed.len() >= HEADER_LENGTH {
                    // 检查是否是嵌套的协议包
                    let nested_packet_len = u32::from_be_bytes([
                        decompressed[0], decompressed[1], decompressed[2], decompressed[3]
                    ]);
                    
                    if nested_packet_len as usize == decompressed.len() {
                        // 这是一个嵌套的协议包，递归解析
                        let mut nested_proto = Proto::new();
                        nested_proto.unpack(&decompressed)?;
                        return nested_proto.get_body_string();
                    }
                }
                
                // 直接解码为字符串
                String::from_utf8(decompressed)
                    .map_err(|e| format!("解码UTF-8失败: {}", e))
            }
            _ => Err(format!("不支持的版本: {}", self.ver))
        }
    }
}

impl Default for Proto {
    fn default() -> Self {
        Self::new()
    }
}

// 各种消息类型的结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanmakuMessage {
    pub cmd: String,
    pub data: DanmakuData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanmakuData {
    pub room_id: i64,
    pub uid: i64,
    pub open_id: String,
    pub union_id: Option<String>,
    pub uname: String,
    pub uface: String,
    pub timestamp: i64,
    pub msg: String,
    pub msg_id: String,
    pub guard_level: i64,
    pub fans_medal_wearing_status: bool,
    pub fans_medal_name: String,
    pub fans_medal_level: i64,
    pub emoji_img_url: Option<String>,
    pub dm_type: i64,
    pub glory_level: Option<i32>,
    pub reply_open_id: Option<String>,
    pub reply_uname: Option<String>,
    pub is_admin: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiftMessage {
    pub cmd: String,
    pub data: GiftData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiftData {
    pub room_id: i64,
    pub uid: i64,
    pub open_id: String,
    pub union_id: Option<String>,
    pub uname: String,
    pub uface: String,
    pub gift_id: i64,
    pub gift_name: String,
    pub gift_num: i64,
    pub price: i64,
    pub r_price: i64,
    pub paid: bool,
    pub fans_medal_level: i64,
    pub fans_medal_name: String,
    pub fans_medal_wearing_status: bool,
    pub guard_level: i64,
    pub timestamp: i64,
    pub anchor_info: AnchorInfo,
    pub msg_id: String,
    pub gift_icon: Option<String>,
    pub combo_gift: Option<bool>,
    pub combo_info: Option<ComboInfo>,
    pub blind_gift: Option<BlindGift>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorInfo {
    pub uid: i64,
    pub open_id: String,
    pub union_id: Option<String>,
    pub uname: String,
    pub uface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboInfo {
    pub combo_base_num: i64,
    pub combo_count: i64,
    pub combo_id: String,
    pub combo_timeout: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindGift {
    pub blind_gift_id: i64,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperChatMessage {
    pub cmd: String,
    pub data: SuperChatData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperChatData {
    pub room_id: i64,
    pub uid: i64,
    pub open_id: String,
    pub union_id: Option<String>,
    pub uname: String,
    pub uface: String,
    pub message_id: i64,
    pub message: String,
    pub rmb: i64,
    pub timestamp: i64,
    pub start_time: i64,
    pub end_time: i64,
    pub guard_level: i64,
    pub fans_medal_level: i64,
    pub fans_medal_name: String,
    pub fans_medal_wearing_status: bool,
    pub msg_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardMessage {
    pub cmd: String,
    pub data: GuardData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardData {
    pub user_info: UserInfo,
    pub guard_level: i64,
    pub guard_num: i64,
    pub guard_unit: String,
    pub price: i64,
    pub fans_medal_level: i64,
    pub fans_medal_name: String,
    pub fans_medal_wearing_status: bool,
    pub room_id: i64,
    pub msg_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub uid: i64,
    pub open_id: String,
    pub union_id: Option<String>,
    pub uname: String,
    pub uface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeMessage {
    pub cmd: String,
    pub data: LikeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeData {
    pub room_id: i64,
    pub uid: i64,
    pub open_id: String,
    pub union_id: Option<String>,
    pub uname: String,
    pub uface: String,
    pub timestamp: i64,
    pub like_text: String,
    pub like_count: i64,
    pub fans_medal_level: i64,
    pub fans_medal_name: String,
    pub fans_medal_wearing_status: bool,
    pub guard_level: Option<i64>,
    pub msg_id: String,
}

// 通用消息结构，用于处理未知或新的消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownMessage {
    pub cmd: String,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum BilibiliMessage {
    #[serde(rename = "LIVE_OPEN_PLATFORM_DM")]
    Danmaku { data: DanmakuData },
    #[serde(rename = "LIVE_OPEN_PLATFORM_SEND_GIFT")]
    Gift { data: GiftData },
    #[serde(rename = "LIVE_OPEN_PLATFORM_SUPER_CHAT")]
    SuperChat { data: SuperChatData },
    #[serde(rename = "LIVE_OPEN_PLATFORM_GUARD")]
    Guard { data: GuardData },
    #[serde(rename = "LIVE_OPEN_PLATFORM_LIKE")]
    Like { data: LikeData },
    #[serde(rename = "LIVE_OPEN_PLATFORM_SUPER_CHAT_DEL")]
    SuperChatDel { data: SuperChatDelData },
    #[serde(rename = "LIVE_OPEN_PLATFORM_LIVE_ROOM_ENTER")]
    LiveRoomEnter { data: LiveRoomEnterData },
    #[serde(rename = "LIVE_OPEN_PLATFORM_LIVE_START")]
    LiveStart { data: LiveStartData },
    #[serde(rename = "LIVE_OPEN_PLATFORM_LIVE_END")]
    LiveEnd { data: LiveEndData },
    #[serde(rename = "LIVE_OPEN_PLATFORM_INTERACTION_END")]
    InteractionEnd { data: InteractionEndData },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperChatDelData {
    pub room_id: i64,
    pub message_ids: Vec<i64>,
    pub msg_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveRoomEnterData {
    pub room_id: i64,
    pub uface: String,
    pub uname: String,
    pub open_id: String,
    pub union_id: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStartData {
    pub room_id: i64,
    pub open_id: String,
    pub union_id: Option<String>,
    pub timestamp: i64,
    pub area_name: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEndData {
    pub room_id: i64,
    pub open_id: String,
    pub union_id: Option<String>,
    pub timestamp: i64,
    pub area_name: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEndData {
    pub game_id: String,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_danmaku_message() {
        let json = r#"
        {
            "cmd": "LIVE_OPEN_PLATFORM_DM",
            "data": {
                "room_id": 1,
                "uid": 0,
                "open_id": "39b8fedb-60a5-4e29-ac75-b16955f7e632",
                "union_id": "U_05ad57b6655a44528cb95a892c491232",
                "uname": "测试用户",
                "uface": "https://example.com/avatar.jpg",
                "timestamp": 1625097600,
                "msg": "这是一条弹幕",
                "msg_id": "msg_12345",
                "guard_level": 0,
                "fans_medal_wearing_status": true,
                "fans_medal_name": "测试勋章",
                "fans_medal_level": 5,
                "emoji_img_url": "",
                "dm_type": 0,
                "glory_level": 39,
                "reply_open_id": "",
                "reply_uname": "",
                "is_admin": 0
            }
        }
        "#;

        let message: BilibiliMessage = serde_json::from_str(json).unwrap();
        match message {
            BilibiliMessage::Danmaku { data } => {
                assert_eq!(data.uname, "测试用户");
                assert_eq!(data.msg, "这是一条弹幕");
                assert_eq!(data.room_id, 1);
            }
            _ => panic!("Expected Danmaku message"),
        }
    }

    #[test]
    fn test_live_start_message() {
        let json = r#"
        {
            "cmd": "LIVE_OPEN_PLATFORM_LIVE_START",
            "data": {
                "room_id": 12345,
                "open_id": "streamer_open_id",
                "union_id": "streamer_union_id",
                "timestamp": 1625097600,
                "area_name": "户外",
                "title": "测试直播标题"
            }
        }
        "#;

        let message: BilibiliMessage = serde_json::from_str(json).unwrap();
        match message {
            BilibiliMessage::LiveStart { data } => {
                assert_eq!(data.room_id, 12345);
                assert_eq!(data.area_name, "户外");
                assert_eq!(data.title, "测试直播标题");
            }
            _ => panic!("Expected LiveStart message"),
        }
    }

    #[test]
    fn test_super_chat_del_message() {
        let json = r#"
        {
            "cmd": "LIVE_OPEN_PLATFORM_SUPER_CHAT_DEL",
            "data": {
                "room_id": 12345,
                "message_ids": [1, 2, 3],
                "msg_id": "del_msg_12345"
            }
        }
        "#;

        let message: BilibiliMessage = serde_json::from_str(json).unwrap();
        match message {
            BilibiliMessage::SuperChatDel { data } => {
                assert_eq!(data.room_id, 12345);
                assert_eq!(data.message_ids, vec![1, 2, 3]);
                assert_eq!(data.msg_id, "del_msg_12345");
            }
            _ => panic!("Expected SuperChatDel message"),
        }
    }
}
