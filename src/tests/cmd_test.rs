use crate::cmd::Cmd;
#[test]
fn super_chat_test() {
    let json = include_str!("./mock/cmd/SuperChatMessage.json");
    let json_val = serde_json::from_str(json).unwrap();
    let cmd = Cmd::deser(json_val).unwrap();
    dbg!(cmd);
}

#[test]
fn send_gift_test() {
    let json = include_str!("./mock/cmd/SendGift.json");
    let json_val = serde_json::from_str(json).unwrap();
    let cmd = Cmd::deser(json_val).unwrap();
    dbg!(cmd);
}