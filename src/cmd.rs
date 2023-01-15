#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Debug, serde::Deserialize)]
pub struct OnlineRankTop3ListItem {
    msg: String,
    rank: u64
}
#[derive(Debug, serde::Deserialize)]
pub struct BlindGiftInfo {
    gift_action: String,
    original_gift_id: u64,
    original_gift_name: String 
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "cmd", content="data", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Cmd {
    ComboSend{
        action: String,
        batch_combo_num: u64,
        combo_total_coin: u64,
        gift_name: String,
        gift_id: u64,
        user: User,
    },
    CommonNoticeDanmaku{},
    EntryEffect{

    },
    GuardBuy {
        gift_id: u64,
        gift_name: String,
        guard_level: u64,
        price: u64,
        num: u64,
        uid: u64,
        username: String
    },
    HotBuyNum{},
    HotRankChangedV2 {
        area_name: String,
        rank: u64,
        rank_desc: String,
    },
    HotRankSettlementV2 {
        area_name: String,
        rank: u64,
        uname: String,
        face: String
    },
    LiveInteractiveGame{},
    OnlineRankV2{},
    OnlineRankTop3{
        dmscore: u64,
        list: Vec<OnlineRankTop3ListItem>
    },
    PopularityRedPocketStart {

    },
    RoomRealTimeMessageUpdate {
        fans: u64,
        fans_club: u64,
        red_notice: i64,
        roomid: u64
    },
    UserToastMsg {

    },
    StopLiveRoomList{},
    InteractWord{
        fans_medal: Option<FansMedal>,
        #[serde(flatten)]
        user: User,
    },
    WatchedChange {
        num: u64
    },
    OnlineRankCount {
        count: u64
    },
    DanmuMsg {
        danmaku_type: u64,
        fans_medal: Option<FansMedal>,
        user: User,
        message: String,
        emoticon: Option<Emoticon>
    },
    SendGift {
        action: String,
        #[serde(flatten)]
        user : User,
        medal_info: Option<FansMedal>,
        #[serde(rename = "giftName")]
        gift_name: String,
        #[serde(rename = "giftId")]
        gift_id: u64,
        num: u64,
        price: u64,
        coin_type: CoinType,
        total_coin: u64,
        blind_gift: Option<BlindGiftInfo>
    },
    SuperChatMessage {
        medal_info: Option<FansMedal>,
        message: String,
        price: u64,
        uid: u64,
        user_info: SuperChatUser
    },
    SuperChatMessageJpn {
        medal_info: Option<FansMedal>,
        message: String,
        message_jpn: String,
        price: u64,
        uid: u64,
        user_info: SuperChatUser
    }
}

use std::fmt::Display;

use serde_json::Value;

use crate::{model::*, event::EventData};

fn medal_filter(fans_medal: Option<FansMedal>) -> Option<FansMedal> {
    match fans_medal {
        Some(FansMedal { medal_level: 0, .. }) | None => None,
        _ => fans_medal,
    }
}

#[derive(Debug)]
pub enum CmdDeserError {
    CannotDeser {
        json_error: serde_json::Error,
        text: String
    },
    Untagged {
        text: String
    },
    Ignored {
        tag: String
    }
}

impl Display for CmdDeserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmdDeserError::CannotDeser { json_error, text } => {
                f.write_fmt(format_args!("无法反序列化，可能是由于未知的cmd tag \n json_error: \n{}, json文本: \n{}", json_error, text))
            },
            CmdDeserError::Untagged { text } => {
                f.write_fmt(format_args!("缺少 tag 的消息\n , json文本: \n{}", text))
            },
            CmdDeserError::Ignored{tag} =>  {
                f.write_fmt(format_args!("被省略的tag: \n{}", tag))
            }
        }
    }
}

impl Cmd {
    pub fn deser(val: Value) -> Result<Self, CmdDeserError> {
        #[cfg(feature="verbose")]
        {
            println!("{}", val.to_string());
        };
        match &val["cmd"] {
            Value::String(cmd) => {
                match cmd.as_str() {
                    "NOTICE_MSG"
                    |"WIDGET_BANNER"
                    |"HOT_RANK_CHANGED"
                    |"HOT_RANK_SETTLEMENT"
                    => {
                        Err(CmdDeserError::Ignored { tag: cmd.clone() })
                    }
                    "DANMU_MSG" => {
                        let info = val["info"].as_array().unwrap();
                        let message = info[1].as_str().unwrap().clone();
                        let user = info[2].as_array().unwrap();
                        let uid = user[0].as_u64().unwrap();
                        let name = user[1].as_str().unwrap();
                        let danmaku_type = info[0].as_array().unwrap()[10].as_u64().unwrap();
                        let fans_medal = &info[3];
                        let fans_medal = {
                            let medal_level = fans_medal[0].as_u64();
                            let medal_name = fans_medal[1].as_str();
                            let anchor_roomid = fans_medal[3].as_u64();
                            let guard_level = fans_medal[10].as_u64();
                            if let (
                                Some(medal_level), 
                                Some(medal_name), 
                                Some(guard_level),
                                Some(anchor_roomid),
                            ) = (medal_level, medal_name, guard_level, anchor_roomid) {
                                Some(FansMedal {
                                    anchor_roomid,
                                    guard_level, 
                                    medal_level,
                                    medal_name: medal_name.to_owned(),
                                })
                            } else {
                                None
                            }
                        };
                        // 是否为表情？
                        let emoticon = if let Some(emoticon) = info[0].as_array().unwrap()[13].as_object() {
                            let height = emoticon["height"].as_u64().unwrap_or_default();
                            let width = emoticon["width"].as_u64().unwrap_or_default();
                            let emoticon_unique = emoticon["emoticon_unique"].as_str().unwrap_or_default().to_owned();
                            let url = emoticon["url"].as_str().unwrap_or_default().to_owned();
                            Some(Emoticon {
                                height, width, url,
                                unique_id: emoticon_unique
                            })
                        } else {
                            None
                        };
                        // 是否为抽奖弹幕？

                        let res = Cmd::DanmuMsg {
                            danmaku_type,
                            fans_medal,
                            user: User {
                                uname: name.to_owned(),
                                uid,
                                face: None,
                            },
                            message: message.to_owned(),
                            emoticon
                        };
                        Ok(res)
                    },
                    _ => {
                        serde_json::from_value(val.clone()).map_err(|json_error|
                            CmdDeserError::CannotDeser{
                                json_error: json_error, 
                                text: val.to_string()
                            }
                        )
                    }
                }
            },
            _ => {
                Err(CmdDeserError::Untagged { text: val.to_string() })
            }
        }
    }

    pub fn as_event(self) -> Option<EventData> {
        match self {
            Cmd::InteractWord { fans_medal, user } 
                => Some(EventData::EnterRoom {
                    user, 
                    fans_medal:medal_filter(fans_medal)
                }),
            Cmd::DanmuMsg { danmaku_type, fans_medal, user, message ,emoticon} => {
                match emoticon {
                    Some(emoticon) =>  Some(EventData::Danmaku { 
                        junk_flag: danmaku_type, 
                        message: DanmakuMessage::Emoticon { 
                            alt_message:  message,
                            emoticon
                        }, 
                        user, 
                        fans_medal
                    }),
                    None => {
                        Some(EventData::Danmaku { 
                            junk_flag: danmaku_type,
                            message: DanmakuMessage::Plain { message }, 
                            user, 
                            fans_medal
                        })
                    }
                }
            },
            Cmd::SuperChatMessage { uid, medal_info, message, price, user_info} => 
                Some(EventData::SuperChat { user: User { uid, uname: user_info.uname, face: Some(user_info.face) }, fans_medal: medal_info, price, message, message_jpn:None }),
            Cmd::SuperChatMessageJpn { uid, medal_info, message, price, user_info, message_jpn} => 
                Some(EventData::SuperChat { user: User { uid, uname: user_info.uname, face: Some(user_info.face) }, fans_medal: medal_info, price, message, message_jpn: Some(message_jpn) }),
            Cmd::WatchedChange { num } => 
                Some(EventData::WatchedUpdate { num }),
            Cmd::SendGift { action, user, medal_info, gift_name, 
                gift_id, num, price, coin_type, total_coin, blind_gift
            } => {
                if let Some(blind_gift_info) = blind_gift {
                    Some(EventData::Gift {
                        user, 
                        fans_medal: medal_filter(medal_info) , 
                        blindbox: Some(GiftType{
                            action: blind_gift_info.gift_action,
                            gift_id: blind_gift_info.original_gift_id,
                            gift_name: blind_gift_info.original_gift_name
                        }),
                        gift: Gift{ action, num, gift_name, gift_id, price, coin_type, coin_count:total_coin}
                    })
                } else {
                    Some(EventData::Gift {
                        user, 
                        fans_medal: medal_filter(medal_info) , 
                        blindbox: None,
                        gift: Gift{ action, num, gift_name, gift_id, price, coin_type, coin_count:total_coin}
                    })
                }
            }
            Cmd::HotRankChangedV2 { area_name, rank, rank_desc} => 
                Some(EventData::HotRankChanged {area: area_name, rank, description: rank_desc }),
            Cmd::HotRankSettlementV2 { area_name, rank, uname, face } => 
                Some(EventData::HotRankSettlement { uname, face, area: area_name, rank}),
            Cmd::GuardBuy { gift_id, gift_name, guard_level, price, num, uid, username} => 
                Some(EventData::GuardBuy{ level: guard_level, price, user: User{uname: username, uid, face:None}}),
            _ => {
                None
            }
        }
    }
}
