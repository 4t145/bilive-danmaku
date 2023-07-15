#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Debug, serde::Deserialize)]
pub struct OnlineRankTop3ListItem {
    msg: String,
    rank: u64,
}
#[derive(Debug, serde::Deserialize)]
pub struct BlindGiftInfo {
    gift_action: String,
    original_gift_id: u64,
    original_gift_name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "cmd", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Cmd {
    ComboSend {
        action: String,
        batch_combo_num: u64,
        combo_total_coin: u64,
        gift_name: String,
        gift_id: u64,
        user: User,
    },
    CommonNoticeDanmaku {},
    EntryEffect {},
    GuardBuy {
        gift_id: u64,
        gift_name: String,
        guard_level: u64,
        price: u64,
        num: u64,
        uid: u64,
        username: String,
    },
    HotBuyNum {},
    HotRankChangedV2 {
        area_name: String,
        rank: u64,
        rank_desc: String,
    },
    HotRankSettlementV2 {
        area_name: String,
        rank: u64,
        uname: String,
        face: String,
    },
    LiveInteractiveGame {},
    OnlineRankV2 {},
    OnlineRankTop3 {
        dmscore: u64,
        list: Vec<OnlineRankTop3ListItem>,
    },
    PopularityRedPocketStart {},
    RoomRealTimeMessageUpdate {
        fans: u64,
        fans_club: u64,
        red_notice: i64,
        roomid: u64,
    },
    UserToastMsg {},
    StopLiveRoomList {
        room_id_list: Vec<u64>,
    },
    InteractWord {
        fans_medal: Option<FansMedal>,
        #[serde(flatten)]
        user: User,
    },
    WatchedChange {
        num: u64,
    },
    OnlineRankCount {
        count: u64,
    },
    DanmuMsg {
        danmaku_type: u64,
        fans_medal: Option<FansMedal>,
        user: User,
        message: String,
        emoticon: Option<Emoticon>,
    },
    SendGift {
        action: String,
        #[serde(flatten)]
        user: User,
        medal_info: Option<FansMedal>,
        #[serde(rename = "giftName")]
        gift_name: String,
        #[serde(rename = "giftId")]
        gift_id: u64,
        num: u64,
        price: u64,
        coin_type: CoinType,
        total_coin: u64,
        blind_gift: Option<BlindGiftInfo>,
    },
    SuperChatMessage {
        medal_info: Option<FansMedal>,
        message: String,
        price: u64,
        uid: u64,
        user_info: SuperChatUser,
    },
    SuperChatMessageJpn {
        medal_info: Option<FansMedal>,
        message: String,
        message_jpn: String,
        price: u64,
        uid: u64,
        user_info: SuperChatUser,
    },
}

use std::fmt::Display;

use serde_json::Value;

use crate::{event::EventData, model::*};

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
        text: String,
    },
    Untagged {
        text: String,
    },
    Ignored {
        tag: String,
    },
    Custom {
        text: String,
    },
}

impl Display for CmdDeserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmdDeserError::CannotDeser { json_error, text } => f.write_fmt(format_args!(
                "无法反序列化\n json_error: \n{}, json文本: \n{}",
                json_error, text
            )),
            CmdDeserError::Untagged { text } => {
                f.write_fmt(format_args!("缺少 tag 的消息\n , json文本: \n{}", text))
            }
            CmdDeserError::Ignored { tag } => f.write_fmt(format_args!("被省略的tag: \n{}", tag)),
            CmdDeserError::Custom { text } => f.write_fmt(format_args!("错误: \n{}", text)),
        }
    }
}

impl std::error::Error for CmdDeserError {}

impl Cmd {
    pub fn deser(val: Value) -> Result<Self, CmdDeserError> {
        log::trace!("deserialize json value: {}", val.to_string());
        match &val["cmd"] {
            Value::String(cmd) => {
                const PROTOCOL_ERROR: &str = "danmu_msg事件协议错误";
                match cmd.as_str() {
                    "NOTICE_MSG" | "WIDGET_BANNER" | "HOT_RANK_CHANGED" | "HOT_RANK_SETTLEMENT" => {
                        Err(CmdDeserError::Ignored { tag: cmd.clone() })
                    }
                    "DANMU_MSG" => {
                        // 如果这里出问题，可能是b站协议发生变更了，所以panic一下无可厚非吧
                        let info = val["info"].as_array().expect(PROTOCOL_ERROR);
                        let message = info[1].as_str().expect(PROTOCOL_ERROR).clone();
                        let user = info[2].as_array().expect(PROTOCOL_ERROR);
                        let uid = user[0].as_u64().expect(PROTOCOL_ERROR);
                        let name = user[1].as_str().expect(PROTOCOL_ERROR);
                        let danmaku_type = info[0].as_array().expect(PROTOCOL_ERROR)[10]
                            .as_u64()
                            .expect(PROTOCOL_ERROR);
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
                            ) = (medal_level, medal_name, guard_level, anchor_roomid)
                            {
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
                        let emoticon = if let Some(emoticon) =
                            info[0].as_array().expect(PROTOCOL_ERROR)[13].as_object()
                        {
                            let height = emoticon["height"].as_u64().unwrap_or_default();
                            let width = emoticon["width"].as_u64().unwrap_or_default();
                            let emoticon_unique = emoticon["emoticon_unique"]
                                .as_str()
                                .unwrap_or_default()
                                .to_owned();
                            let url = emoticon["url"].as_str().unwrap_or_default().to_owned();
                            Some(Emoticon {
                                height,
                                width,
                                url,
                                unique_id: emoticon_unique,
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
                            emoticon,
                        };
                        Ok(res)
                    }
                    _ => serde_json::from_value(val.clone()).map_err(|json_error| {
                        CmdDeserError::CannotDeser {
                            json_error,
                            text: val.to_string(),
                        }
                    }),
                }
            }
            _ => Err(CmdDeserError::Untagged {
                text: val.to_string(),
            }),
        }
    }

    pub fn into_event(self) -> Option<EventData> {
        use crate::event::*;
        match self {
            Cmd::InteractWord { fans_medal, user } => {
                Some(EventData::EnterRoomEvent(EnterRoomEvent {
                    user,
                    fans_medal: medal_filter(fans_medal),
                }))
            }
            Cmd::DanmuMsg {
                danmaku_type,
                fans_medal,
                user,
                message,
                emoticon,
            } => match emoticon {
                Some(emoticon) => Some(EventData::DanmakuEvent(DanmakuEvent {
                    flag: danmaku_type,
                    message: DanmakuMessage::Emoticon {
                        alt_message: message,
                        emoticon,
                    },
                    user,
                    fans_medal,
                })),
                None => Some(EventData::DanmakuEvent(DanmakuEvent {
                    flag: danmaku_type,
                    message: DanmakuMessage::Plain { message },
                    user,
                    fans_medal,
                })),
            },
            Cmd::SuperChatMessage {
                uid,
                medal_info,
                message,
                price,
                user_info,
            } => Some(EventData::SuperChatEvent(SuperChatEvent {
                user: User {
                    uid,
                    uname: user_info.uname,
                    face: Some(user_info.face),
                },
                fans_medal: medal_info,
                price,
                message,
                message_jpn: None,
            })),
            Cmd::SuperChatMessageJpn {
                uid,
                medal_info,
                message,
                price,
                user_info,
                message_jpn,
            } => Some(EventData::SuperChatEvent(SuperChatEvent {
                user: User {
                    uid,
                    uname: user_info.uname,
                    face: Some(user_info.face),
                },
                fans_medal: medal_info,
                price,
                message,
                message_jpn: Some(message_jpn),
            })),
            Cmd::WatchedChange { num } => {
                Some(EventData::WatchedUpdateEvent(WatchedUpdateEvent { num }))
            }
            Cmd::SendGift {
                action,
                user,
                medal_info,
                gift_name,
                gift_id,
                num,
                price,
                coin_type,
                total_coin,
                blind_gift,
            } => {
                if let Some(blind_gift_info) = blind_gift {
                    Some(EventData::GiftEvent(GiftEvent {
                        user,
                        fans_medal: medal_filter(medal_info),
                        blindbox: Some(GiftType {
                            action: blind_gift_info.gift_action,
                            gift_id: blind_gift_info.original_gift_id,
                            gift_name: blind_gift_info.original_gift_name,
                        }),
                        gift: Gift {
                            action,
                            num,
                            gift_name,
                            gift_id,
                            price,
                            coin_type,
                            coin_count: total_coin,
                        },
                    }))
                } else {
                    Some(EventData::GiftEvent(GiftEvent {
                        user,
                        fans_medal: medal_filter(medal_info),
                        blindbox: None,
                        gift: Gift {
                            action,
                            num,
                            gift_name,
                            gift_id,
                            price,
                            coin_type,
                            coin_count: total_coin,
                        },
                    }))
                }
            }
            Cmd::HotRankChangedV2 {
                area_name,
                rank,
                rank_desc,
            } => Some(
                HotRankChangedEvent {
                    area: area_name,
                    rank,
                    description: rank_desc,
                }
                .into(),
            ),
            Cmd::HotRankSettlementV2 {
                area_name,
                rank,
                uname,
                face,
            } => Some(
                HotRankSettlementEvent {
                    uname,
                    face,
                    area: area_name,
                    rank,
                }
                .into(),
            ),
            Cmd::GuardBuy {
                gift_id,
                gift_name,
                guard_level,
                price,
                num,
                uid,
                username,
            } => Some(
                GuardBuyEvent {
                    level: guard_level,
                    price,
                    user: User {
                        uname: username,
                        uid,
                        face: None,
                    },
                }
                .into(),
            ),
            Cmd::StopLiveRoomList { room_id_list } => Some(StopLiveEvent { room_id_list }.into()),
            rest => {
                log::debug!("unhandled cmd: {:?}", rest);
                None
            }
        }
    }
}
