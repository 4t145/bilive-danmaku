# bilive-danmaku
这个库提供模拟bilibili直播的wss连接的功能，此readme是为`ver-0.1.0`准备的
## 使用
因为使用了尚未稳定的`split_array`，因此需要切换到nightly版本
```
rustup override set nightly
```
在`Cargo.toml`中加入
```toml
bilive-danmaku = { git = "https://github.com/4t145/bilive-danmaku", branch = "ver-0.1.0" }
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "tag", content="data")]
pub enum Event {
    Danmaku {
        // junk_flag=0 才是正常弹幕（非抽奖/天选等）
        // 抽奖，天选，junk_flag = 2
        // 其他值不知道有什么意义，所以暂且保留这个字段为u64
        junk_flag: u64,
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

## feature flag
|flag|功能|
|:---:|:--:|
|`connect`|连接直播间，默认启用|
|`event`|只启用model和event，不包含连接|
|`bincode`|启用bincode正反序列化|
|`json`|启用json正反序列化|
|`verbose`|debug用，输出每条解析的json|
|`debug`|debug用，输出解析错误|


默认只启用`connect`
比如你想把收到的消息序列化为json格式，启用
```toml
[dependencies.bilive-danmaku]
# ****
features = ["connect", "json"]
```

## JavaScript/TypeScript 支持
```bash
npm install bilive-danmaku-json@0.1.0-rc4
```
### 使用例
```TypeScript
import {Event, DanmakuEvent} from 'bilive-danmaku-json';
function on_danmaku(data: DanmakuEvent['data']) {
    if(data.junk_flag===0) {
        console.log(data.message);
    }
    // ...
}
// ... 获取data
const evt = JSON.parse(data);
if(evt.tag === 'Danmaku') {
    on_danmaku(evt.data);
}
```