use crate::model::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventWithTime {
    event: Event,
    time: std::time::Instant
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "tag", content="data")]
pub enum Event {
    Danmaku {
        junk_flag: u64,
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

impl Event {
    pub fn wrap_time(self) -> EventWithTime {
        EventWithTime {
            event: self,
            time: std::time::Instant::now()
        }
    }
}

#[cfg(feature = "bincode")]
impl Event {
    pub fn to_bincode(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize::<Self>(self)
    }
    pub fn from_bincode(bincode: &[u8]) -> bincode::Result<Self> {
        bincode::deserialize(bincode)
    }
}

#[cfg(feature = "json")]
impl Event {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
    
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str::<Self>(json)
    }
}