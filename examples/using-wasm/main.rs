use bilive_danmaku::Connection;
use futures_util::StreamExt;
use wasm_bindgen_futures::spawn_local;
fn main() {
    spawn_local(wasm_main());
}

async fn wasm_main() {
    let connection = Connection::init(21470454).await.unwrap();
    let mut stream = connection.connect().await.unwrap();
    while let Some(maybe_evt) = stream.next().await {
        match maybe_evt {
            Ok(evt) => {
                dbg!(evt.data);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }
}
