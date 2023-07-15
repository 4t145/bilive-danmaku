use bilive_danmaku::Connector;
use futures_util::StreamExt;
fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,bilive_danmaku=debug");
    }
    env_logger::init();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(tokio_main());
}

fn read_roomid() -> u64 {
    let mut roomid = String::new();
    println!("请输入房间号:");
    std::io::stdin().read_line(&mut roomid).unwrap();
    roomid.trim().parse().unwrap()
}

async fn tokio_main() {
    let roomid = std::env::var("room_id")
        .map(|s| str::parse::<u64>(&s).expect("invalid room id"))
        .unwrap_or(read_roomid());
    let connector = Connector::init(roomid).await.unwrap();
    let mut stream = connector.connect().await.unwrap();
    while let Some(maybe_evt) = stream.next().await {
        match maybe_evt {
            Ok(evt) => {
                log::info!("{:?}", evt);
            }
            Err(e) => {
                log::warn!("{:?}", e);
            }
        }
    }
    stream.abort();
}
