pub trait Operation {
    
}

pub struct Auth {
    uid: String,
    roomid: String,
    protover: i32,
    platform: &'static str,
    r#type: i32,
    key: Option<String>
}

