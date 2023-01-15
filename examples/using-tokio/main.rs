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
    let start_time = tokio::time::Instant::now();
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
    let timespan = tokio::time::Instant::now().duration_since(start_time).as_secs();
    println!("close after {timespan}");
    stream.abort();
}