use crate::model::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "tag", content="data")]
pub enum EventData {
    Danmaku {
        /// 第一位：是否是抽奖弹幕，2~4位，舰长类型
        flag: u64,
        message: DanmakuMessage,
        user: User,
        fans_medal: Option<FansMedal>
    },
    EnterRoom {
        user: User,
        fans_medal: Option<FansMedal>
    },
    BlindboxGift {
        user: User,
        fans_medal: Option<FansMedal>,
        blindbox_gift_type: GiftType,
        gift: Gift,
    },
    Gift {
        user: User,
        fans_medal: Option<FansMedal>,
        blindbox: Option<GiftType>,
        gift: Gift,
    },
    GuardBuy {
        level: u64,
        price: u64,
        user: User
    },
    SuperChat {
        user: User,
        fans_medal: Option<FansMedal>,
        price: u64, 
        message: String,
        message_jpn: Option<String>
    },
    WatchedUpdate {
        num: u64
    },
    PopularityUpdate {
        popularity: u32,
    },
    GuardEnterRoom {
        user: User,
    },
    HotRankChanged {
        area: String,
        rank: u64,
        description: String,
    },
    HotRankSettlement {
        uname: String,
        face: String,
        area: String,
        rank: u64,
    },
}

impl Into<Event> for EventData {
    fn into(self) -> Event {
        use std::time::*;
        Event {
            data: self,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    #[serde(flatten)]
    pub data: EventData,
    pub timestamp: u64,
}

#[cfg(feature = "bincode")]
impl EventData {
    pub fn to_bincode(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize::<Self>(self)
    }
    pub fn from_bincode(bincode: &[u8]) -> bincode::Result<Self> {
        bincode::deserialize(bincode)
    }
}

#[cfg(feature = "json")]
impl EventData {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
    
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str::<Self>(json)
    }
}

#[cfg(feature = "rt_wasm")]
impl Into<wasm_bindgen::JsValue> for Event {
    fn into(self) -> wasm_bindgen::JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
}

#[cfg(feature = "rt_wasm")]
impl wasm_bindgen::describe::WasmDescribe for Event {
    fn describe() {
        
    }
}