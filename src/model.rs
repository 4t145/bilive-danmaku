use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct Emoticon {
    pub unique_id: String,
    pub height: u64,
    pub width: u64,
    pub url: String,
}
///
/// # 说明
/// - `guard_level`字段，1，2，3分别为总督，提督，舰长；0为无。
/// - `anchor_roomid` 大航海房间id
#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct FansMedal {
    pub anchor_roomid: u64,
    #[serde(default)]
    pub guard_level: u64,
    pub medal_level: u64,
    pub medal_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct User {
    pub uid: u64,
    pub uname: String,
    pub face: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub(crate) struct SuperChatUser {
    pub(crate) uname: String,
    pub(crate) face: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum CoinType {
    Silver,
    Gold,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash)]
pub struct Gift {
    pub coin_type: CoinType,
    pub coin_count: u64,
    pub action: String,
    pub gift_name: String,
    pub gift_id: u64,
    pub num: u64,
    pub price: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash)]
pub struct GiftType {
    pub action: String,
    pub gift_name: String,
    pub gift_id: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash)]
#[serde(tag = "tag", content = "data")]
pub enum DanmakuMessage {
    Plain {
        message: String,
    },
    Emoticon {
        emoticon: Emoticon,
        alt_message: String,
    },
}

impl Display for FansMedal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}|{}]", self.medal_name, self.medal_level))
    }
}

impl Display for DanmakuMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DanmakuMessage::Plain { message } => f.write_str(message),
            DanmakuMessage::Emoticon {
                emoticon: _,
                alt_message,
            } => f.write_fmt(format_args!("[表情:{}]", alt_message)),
        }
    }
}

impl Display for Gift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{}x{}[{:.2}CNY]",
            self.action,
            self.gift_name,
            self.num,
            ((self.price * self.num) as f32) / 1000.0
        ))
    }
}
