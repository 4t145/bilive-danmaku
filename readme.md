# bilive-danmaku
这个库提供模拟bilibili直播的wss连接的功能，目前还在开发中
## 使用
因为使用了尚未稳定的`split_array`，因此需要切换到nightly版本
```
rustup override set nightly
```
在`Cargo.toml`中加入
```toml
bilive-danmaku = { git = "https://github.com/4t145/bilive-danmaku", branch = "master" }
```
使用
```rust
use bilive_danmaku::{RoomService};


async fn service() {
    let service = RoomService::new(477317922).init().await.unwrap();
    let service = service.connect().await.unwrap();
    // 这里会获得一个 broadcast::Reciever<Event>
    let mut events_rx = service.subscribe();
    while let Some(evt) = events_rx.recv().await {
        // 处理事件
        todo!()
    }
    let service = service.close();
}
```
数据类型在`model`模块中, 事件类型在`event`模块中
```rust
use model::{User, FansMedal};
use event::Event as BiliEvent;
```
## 已经支持的事件


```rust
#[derive(Clone, Debug)]
pub enum Event {
    Danmaku {
        message: DanmakuMessage,
        user: User,
        fans_medal: Option<FansMedal>
    },
    EnterRoom {
        user: User,
        fans_medal: Option<FansMedal>
    },
    Gift {
        user: User,
        fans_medal: Option<FansMedal>,
        gift: Gift,
    },
    GuardBuy {
        level: u64,
        price: u64,
        user: User
    },
    SuperChat {
        user: User,
        fans_medal: Option<FansMedal>,
        price: u64, 
        message: String,
        message_jpn: Option<String>
    },
    WatchedUpdate {
        num: u64
    },
    PopularityUpdate {
        popularity: u32,
    },
    GuardEnterRoom {
        user: User,
    },
    HotRankChanged {
        area: String,
        rank: u64,
        description: String,
    },
    HotRankSettlement {
        uname: String,
        face: String,
        area: String,
        rank: u64,
    },
}
```
可参考：
- [命令原始数据](./src//tests/mock/cmd/)
- [源文件](./src/event.rs)

