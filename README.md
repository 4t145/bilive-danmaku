# bilive-danmaku
[![Crates.io][crates-badge]][crates-url]
[![Publish][ci-publish-badge]][ci-publish-url]
[![Build][ci-build-badge]][ci-build-url]

[crates-badge]: https://img.shields.io/crates/v/bilive-danmaku.svg
[crates-url]: https://crates.io/crates/bilive-danmaku
[ci-publish-badge]: https://github.com/4t145/bilive-danmaku/actions/workflows/publish.yml/badge.svg?branch=publish
[ci-publish-url]: https://github.com/4t145/bilive-danmaku/actions/workflows/publish.yml/?branch=publish
[ci-build-badge]: https://github.com/4t145/bilive-danmaku/actions/workflows/rust.yml/badge.svg?branch=master
[ci-build-url]: https://github.com/4t145/bilive-danmaku/actions/workflows/rust.yml/?branch=master

这个库提供模拟bilibili直播的wss连接的功能，持续迭代中

关于发送弹幕等主动api，可以看我这个仓库: https://github.com/4t145/bilibili-client

## 使用
### 通过websocket
通过使用 https://github.com/4t145/rudanmaku-core

这使你可以通过ws来获取事件，计划在未来支持ipc通讯（uds for linux，命名管道 for windows）

在`Cargo.toml`中加入
```toml
bilive-danmaku = { version = "0.3", features = ["rt_tokio"] }
```
使用
```rust
use bilive_danmaku::Connector;
use futures_util::StreamExt;

async fn tokio_main() {
    let roomid = 851181;
    let connector = Connector::init(roomid, None).await.unwrap();
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
```

数据类型在`model`模块中, 事件类型在`event`模块中
```rust
use model::{User, FansMedal};
use event::Event as BiliEvent;
```
## 已经支持的事件
[参考这个文件](./src/event.rs)

可参考：
- [命令原始数据](./src/tests/mock/cmd/)

## feature flag
|flag|功能|
|:---:|:--:|
|`event`|只启用model和event，不包含连接，默认启用|
|`rt_tokio`|使用tokio连接直播间|
|`rt_wasm`|运行在wasm直播间|
|`bincode`|启用bincode正反序列化|
|`json`|启用json正反序列化|

默认只启用`event`
比如你想把收到的消息序列化为json格式，启用
```toml
[dependencies.bilive-danmaku]
# ****
features = ["rt_tokio", "json"]
```

# 提交代码
提交代码请fork一份，在自己的那一份签出新分支，然后提交到master分支

提交前请进行格式化和clippy check，可以直接运行根目录的脚本文件

windows
```shell
./fix-all
```

linux
```bash
bash fix-all.sh
```
