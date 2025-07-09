use serde_json;
use crate::proto::BilibiliMessage;

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_gift_message() {
        let json = r#"
        {
            "cmd": "LIVE_OPEN_PLATFORM_SEND_GIFT",
            "data": {
                "room_id": 1,
                "uid": 0,
                "open_id": "39b8fedb-60a5-4e29-ac75-b16955f7e632",
                "union_id": "U_05ad57b6655a44528cb95a892c491232",
                "uname": "测试用户",
                "uface": "https://example.com/avatar.jpg",
                "gift_id": 123,
                "gift_name": "小花花",
                "gift_num": 5,
                "price": 1000,
                "r_price": 1000,
                "paid": true,
                "fans_medal_level": 5,
                "fans_medal_name": "测试勋章",
                "fans_medal_wearing_status": true,
                "guard_level": 0,
                "timestamp": 1625097600,
                "anchor_info": {
                    "uid": 0,
                    "open_id": "anchor_open_id",
                    "union_id": "anchor_union_id",
                    "uname": "主播名",
                    "uface": "https://example.com/anchor_avatar.jpg"
                },
                "msg_id": "gift_msg_12345",
                "gift_icon": "https://example.com/gift_icon.jpg",
                "combo_gift": true,
                "combo_info": {
                    "combo_base_num": 1,
                    "combo_count": 5,
                    "combo_id": "combo_12345",
                    "combo_timeout": 60
                },
                "blind_gift": {
                    "blind_gift_id": 456,
                    "status": false
                }
            }
        }
        "#;

        let message: BilibiliMessage = serde_json::from_str(json).unwrap();
        match message {
            BilibiliMessage::Gift { data } => {
                assert_eq!(data.uname, "测试用户");
                assert_eq!(data.gift_name, "小花花");
                assert_eq!(data.gift_num, 5);
                assert_eq!(data.price, 1000);
            }
            _ => panic!("Expected Gift message"),
        }
    }

    #[test]
    fn test_like_message() {
        let json = r#"
        {
            "cmd": "LIVE_OPEN_PLATFORM_LIKE",
            "data": {
                "room_id": 1,
                "uid": 0,
                "open_id": "39b8fedb-60a5-4e29-ac75-b16955f7e632",
                "union_id": "U_05ad57b6655a44528cb95a892c491232",
                "uname": "测试用户",
                "uface": "https://example.com/avatar.jpg",
                "timestamp": 1625097600,
                "like_text": "为主播点赞了",
                "like_count": 10,
                "fans_medal_level": 5,
                "fans_medal_name": "测试勋章",
                "fans_medal_wearing_status": true,
                "guard_level": 0,
                "msg_id": "like_msg_12345"
            }
        }
        "#;

        let message: BilibiliMessage = serde_json::from_str(json).unwrap();
        match message {
            BilibiliMessage::Like { data } => {
                assert_eq!(data.uname, "测试用户");
                assert_eq!(data.like_count, 10);
                assert_eq!(data.like_text, "为主播点赞了");
            }
            _ => panic!("Expected Like message"),
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
