use futures_util::Stream;
use std::{
    collections::{hash_map::DefaultHasher, HashMap, VecDeque, HashSet},
    hash::{Hash, Hasher},
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::Poll, ops::AddAssign,
};

use crate::event::Event;
type SyncChannelId = u64;
#[derive(Debug, Default)]
pub struct SyncHub {
    next_id: AtomicU64,
    pub channels: HashMap<SyncChannelId, SyncChannel>,
}

impl SyncHub {
    pub fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    #[allow(clippy::unnecessary_fold)]
    pub fn push(&mut self, id: SyncChannelId, event: Event) -> Option<Event> {
        let mut hasher = DefaultHasher::new();
        event.data.hash(&mut hasher);
        let hash = hasher.finish();
        self.channels
            .values_mut()
            .fold(false, |update, chan| update || chan.pull(id, hash))
            .then_some(event)
    }

    pub fn add_channel(
        &mut self,
        backend: impl Stream<Item = Event> + Sync + Send + 'static,
    ) -> SyncChannelId {
        let id = self.next_id();
        let channel = SyncChannel {
            id,
            backend: Box::pin(backend),
            hash: Default::default(),
            memory: Default::default(),
            source: 0,
        };
        self.channels.insert(id, channel);
        id
    }

    pub fn remove_channel(&mut self, id: SyncChannelId) -> Option<SyncChannel> {
        self.channels.remove(&id)
    }

    pub fn reset_all(&mut self) {
        for chan in self.channels.values_mut() {
            chan.hash.store(0, Ordering::SeqCst);
            chan.memory.0.clear();
            chan.memory.1.clear();
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        for (_, chan) in other.channels {
            self.channels.insert(self.next_id(), chan);
        }
        self.reset_all();
        self
    }
}

impl Stream for SyncHub {
    type Item = Event;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut new_event = None;
        for (id, chan) in self.channels.iter_mut() {
            if let Poll::Ready(Some(event)) = chan.backend.as_mut().poll_next(cx) {
                if env!("CARGO_PKG_VERSION") != event.meta.lib_version {
                    log::warn!(
                        "版本不匹配：本地版本 {}，数据源版本 {}, 数据源： {:?}",
                        env!("CARGO_PKG_VERSION"),
                        event.meta.lib_version,
                        event.meta.source
                    );
                } else {
                    new_event = Some((*id, event));
                    break;
                }
            }
        }
        if let Some((id, event)) = new_event {
            if let Some(event) = self.push(id, event) {
                return Poll::Ready(Some(event));
            }
        }
        Poll::Pending
    }
}

pub struct SyncChannel {
    id: SyncChannelId,
    source: SyncChannelId,
    hash: AtomicU64,
    memory: (VecDeque<u64>, HashSet<u64>),
    backend: Pin<Box<dyn Stream<Item = Event> + Sync + Send>>,
}

impl std::fmt::Debug for SyncChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncChannel")
            .field("id", &self.id)
            .field("hash", &self.hash)
            .finish()
    }
}

impl SyncChannel {
    pub fn pull(&mut self, _id: SyncChannelId, hash: u64) -> bool {
        const MEMORY_SIZE: usize = 128;
        self.memory.0.push_back(hash);
        let is_new = self.memory.1.insert(hash);
        if self.memory.0.len() > MEMORY_SIZE {
            let x = self.memory.0.pop_front().expect("memory size error");
            self.memory.1.remove(&x);
        }
        is_new
    }
}
