use bilive_danmaku::Connector;
use futures_util::StreamExt;
fn main() {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(tokio_main());
}


async fn tokio_main() {
    let connector = Connector::init(22470216).await.unwrap();
    let mut stream = connector.connect().await.unwrap();
    while let Some(maybe_evt) = stream.next().await {
        match maybe_evt {
            Ok(evt) => {
                dbg!(evt.data);
            },
            Err(e) => {
                dbg!(e);
            },
        }
    }
    stream.abort();
}