// #[test]
// fn cli_test() {
//     use std::io::{stdin, stdout};
//     println!("输入房间号：");
//     let mut input = String::new();
//     match stdin().read_line(&mut input) {
//         Ok(_size) => {
//             if let Ok(roomid) = u64::from_str_radix(&input, 10) {
//                 let service = crate::RoomService::new(roomid);
//                 tokio::spawn(async move {
//                     let service = service.init().await.unwrap();
//                     let service = service.connect().await.unwrap();
//                     let rx = service.subscribe();
//                     while let Some(evt) = {

//                     }
//                 });
//             }
//         },
//         Err(_) => todo!(),
//     }

// }
