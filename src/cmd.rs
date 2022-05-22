
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "cmd", content="data", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Cmd {
    ComboSend,
    CommanNoticeDanmuku,
    EnterEffect,
    HotBuyNum,
    LiveInteractiveGame,
    OnlineRankV2,
    StopLiveRoomList,
    InteractWord{
        fans_medal: Option<FansMedal>,
        #[serde(flatten)]
        user: User,
    },
    WatchedChange {
        num: u64
    },
    OnlineRankCount {
        _count: u64
    },
    DanmuMsg {
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
        total_coin: u64
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

use crate::{model::*, event::Event};

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
        match &val["cmd"] {
            Value::String(cmd) => {
                match cmd.as_str() {
                    "NOTICE_MSG"|"WIDGET_BANNER" => {
                        Err(CmdDeserError::Ignored { tag: cmd.clone() })
                    }
                    "DANMU_MSG" => {
                        let info = val["info"].as_array().unwrap();
                        let message = info[1].as_str().unwrap().clone();
                        let user = info[2].as_array().unwrap();
                        let uid = user[0].as_u64().unwrap();
                        let name = user[1].as_str().unwrap();
                        let fans_medal = &info[3];
                        let fans_medal = {
                            let medal_level = fans_medal[0].as_u64();
                            let medal_name = fans_medal[1].as_str();
                            if medal_level.is_some() && medal_name.is_some() {
                                let medal_level = medal_level.unwrap();
                                Some(FansMedal {
                                    medal_level,
                                    medal_name: medal_name.unwrap().to_owned(),
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
                        let res = Cmd::DanmuMsg {
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

    pub fn as_event(self) -> Option<Event> {
        match self {
            Cmd::InteractWord { fans_medal, user } 
                => Some(Event::EnterRoom {user, fans_medal}),
            Cmd::DanmuMsg { fans_medal, user, message ,emoticon} => {
                match emoticon {
                    Some(emoticon) =>  Some(Event::Danmaku { 
                        message: DanmakuMessage::Emoticon { 
                            alt_message:  message,
                            emoticon
                        }, 
                        user, 
                        fans_medal
                    }),
                    None => {
                        Some(Event::Danmaku { 
                            message: DanmakuMessage::Plain { message }, 
                            user, 
                            fans_medal
                        })
                    }
                }
            },
            Cmd::SuperChatMessage { uid, medal_info, message, price, user_info} => 
                Some(Event::SuperChat { user: User { uid, uname: user_info.uname, face: Some(user_info.face) }, fans_medal: medal_info, price, message, message_jpn:None }),
            Cmd::SuperChatMessageJpn { uid, medal_info, message, price, user_info, message_jpn} => 
                Some(Event::SuperChat { user: User { uid, uname: user_info.uname, face: Some(user_info.face) }, fans_medal: medal_info, price, message, message_jpn: Some(message_jpn) }),
            Cmd::WatchedChange { num } => 
                Some(Event::WatchedUpdate { num }),
            Cmd::SendGift { action, user, medal_info, gift_name, gift_id, num, price, coin_type, total_coin } => 
                Some(Event::Gift {user, fans_medal: medal_info, gift: Gift{ action, num, gift_name, gift_id, price, coin_type, coin_count:total_coin}}),

            _ => {
                None
            }
        }
    }
}