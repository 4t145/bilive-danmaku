use bilive_danmaku::Connector;
use futures_util::StreamExt;
fn main() {
    std::env::set_var("RUST_LOG", "info,bilive_danmaku=debug");
    env_logger::init();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(tokio_main());
}

async fn tokio_main() {
    let connector = Connector::init(21452505).await.unwrap();
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
