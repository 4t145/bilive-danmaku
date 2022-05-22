# bilive-danmaku
这个库提供模拟bilibili直播的wss连接的功能，目前还在开发中

## 使用例
```toml
bilive-danmaku = { git = "https://github.com/4t145/bilive-danmaku", branch = "master" }
```

```rust
use bilive_danmaku::{RoomService}


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

## 已经支持的事件
```rust
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
    }
}
```
