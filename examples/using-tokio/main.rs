use bilive_danmaku::Connection;
use bilive_danmaku::connector::{ TokioConnector, Connector };
use futures_util::StreamExt;
fn main() {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(tokio_main());
}


async fn tokio_main() {
    let connection = Connection::init(21470454).await.unwrap();
    let mut stream = connection.connect::<TokioConnector>().await.unwrap();
    while let Some(Ok(event)) = stream.next().await {
        dbg!(event);
    }
    stream.abort();
}