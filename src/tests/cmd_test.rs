use crate::cmd::Cmd;
#[test]
fn super_chat_test() {
    let json = include_str!("./mock/cmd/SuperChatMessage.json");
    let json_val = serde_json::from_str(json).expect("json parse error");
    let cmd = Cmd::deser(json_val).expect("cmd deser error");
    dbg!(cmd);
}

#[test]
fn send_gift_test() {
    let json = include_str!("./mock/cmd/SendGift.json");
    let json_val = serde_json::from_str(json).expect("json parse error");
    let cmd = Cmd::deser(json_val).expect("cmd deser error");
    dbg!(cmd);
}

#[test]
fn stop_list_test() {
    let json = include_str!("./mock/cmd/StopLiveRoomList.json");
    let json_val = serde_json::from_str(json).expect("json parse error");
    let cmd = Cmd::deser(json_val).expect("cmd deser error");
    dbg!(cmd);
}
