use crate::model::*;

use serde::{Deserialize, Serialize};

macro_rules! define_event {
    ($(
        $name:ident{$(
            $(#[$attrs:meta])*
            $arg:ident: $ty:ty
        ),*$(,)?}
    ),*$(,)?) => {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[serde(tag = "cmd", content="data")]
        pub enum EventData {
            $($name ($name)),*
        }

        $(
            #[derive(Clone, Debug, Serialize, Deserialize)]
            pub struct $name {
                $(
                    $(#[$attrs])*
                    pub $arg: $ty
                ),*
            }
            impl From<$name> for EventData {
                fn from(event: $name) -> Self {
                    EventData::$name(event)
                }
            }
        )*
    };
}

define_event! {
    DanmakuEvent {
        /// 第一位：是否是抽奖弹幕，2~4位，舰长类型
        flag: u64,
        message: DanmakuMessage,
        user: User,
        fans_medal: Option<FansMedal>
    },
    EnterRoomEvent {
        user: User,
        fans_medal: Option<FansMedal>
    },
    BlindboxGiftEvent {
        user: User,
        fans_medal: Option<FansMedal>,
        blindbox_gift_type: GiftType,
        gift: Gift,
    },
    GiftEvent {
        user: User,
        fans_medal: Option<FansMedal>,
        blindbox: Option<GiftType>,
        gift: Gift,
    },
    GuardBuyEvent {
        level: u64,
        price: u64,
        user: User
    },
    SuperChatEvent {
        user: User,
        fans_medal: Option<FansMedal>,
        price: u64,
        message: String,
        message_jpn: Option<String>
    },
    WatchedUpdateEvent {
        num: u64
    },
    PopularityUpdateEvent {
        popularity: u32,
    },
    GuardEnterRoomEvent {
        user: User,
    },
    HotRankChangedEvent {
        area: String,
        rank: u64,
        description: String,
    },
    HotRankSettlementEvent {
        uname: String,
        face: String,
        area: String,
        rank: u64,
    },
    StopLiveEvent{
        list: Vec<u64>
    }
}

impl From<EventData> for Event {
    fn from(val: EventData) -> Self {
        use std::time::*;
        Event {
            data: val,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("时间倒流")
                .as_millis() as u64,
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

#[cfg(feature = "rt_wasm")]
impl From<Event> for wasm_bindgen::JsValue {
    fn from(val: Event) -> Self {
        serde_wasm_bindgen::to_value(&val)
            .expect("this should not happen, event data are defined by ourselves")
    }
}

#[cfg(feature = "rt_wasm")]
impl wasm_bindgen::describe::WasmDescribe for Event {
    fn describe() {}
}
