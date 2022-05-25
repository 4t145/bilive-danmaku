use crate::model::*;

#[derive(Clone, Debug)]
pub enum Event {
    Danmaku {
        message: DanmakuMessage,
        user: User,
        fans_medal: Option<FansMedal>
    },
    EnterRoom {
        user: User,
        fans_medal: Option<FansMedal>
    },
    Gift {
        user: User,
        fans_medal: Option<FansMedal>,
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