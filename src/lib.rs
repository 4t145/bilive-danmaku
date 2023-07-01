//! # 使用
//!
//!
//!```
//!use bilive_danmaku::{RoomService}
//!async fn service() {
//!    let service = RoomService::new(477317922).init().await.unwrap();
//!    let service = service.connect().await.unwrap();
//!    // 这里会获得一个 broadcast::Reciever<Event>
//!    let mut events_rx = service.subscribe();
//!    while let Some(evt) = events_rx.recv().await {
//!        // 处理事件
//!        todo!()
//!    }
//!    let service = service.close();
//!}
//!```

// #![allow(dead_code)]
#![deny(clippy::unwrap_used, clippy::print_stdout, clippy::panic)]
#![feature(split_array)]
#[cfg(feature = "connect")]
pub mod connection;
#[cfg(feature = "connect")]
mod room;
#[cfg(feature = "connect")]
pub use crate::room::*;
#[cfg(feature = "connect")]
pub use connection::Connection;
#[cfg(feature = "connect")]
pub(crate) mod cmd;

#[cfg(feature = "event")]
pub mod event;
#[cfg(feature = "event")]
pub mod model;

#[cfg(test)]
mod tests;

#[cfg(feature = "connect")]
mod packet;
