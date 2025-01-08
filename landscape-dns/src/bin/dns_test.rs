use landscape_dns::LandscapeDnsService;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), ()> {
    // let data_path = PathBuf::new();
    // let dns_service = Arc::new(Mutex::new(LandscapeDnsService::new(data_path).await));

    // let status = {
    //     let lock = dns_service.lock().await;
    //     lock.start(1053, None).await;
    //     lock.status.clone()
    // };

    // loop {
    //     let _ = status.subscribe().changed().await;
    //     let result = status.borrow().clone();
    //     match result {
    //         landscape_dns::ServiceStatus::Staring => {
    //             println!("启动")
    //         }
    //         landscape_dns::ServiceStatus::Running => {
    //             println!("Running")
    //         }
    //         landscape_dns::ServiceStatus::Stopping => todo!(),
    //         landscape_dns::ServiceStatus::Stop { message } => todo!(),
    //     }
    // }

    let mut data_map: BTreeMap<i32, MyData> = BTreeMap::new();

    // 插入数据，BTreeMap 会自动按 index 排序
    data_map.insert(10, MyData { index: 10, value: "data1".to_string() });
    data_map.insert(5, MyData { index: 5, value: "data2".to_string() });
    data_map.insert(15, MyData { index: 15, value: "data3".to_string() });

    // 根据 index 读取数据
    if let Some(data) = data_map.get(&10) {
        println!("找到 index 为 10 的数据: {:?}", data);
    }

    // 按照 index 的大小顺序遍历数据
    for (index, data) in &data_map {
        println!("index: {}, data: {:?}", index, data);
    }

    Ok(())
}

#[derive(Debug)]
struct MyData {
    index: i32,
    value: String,
}
