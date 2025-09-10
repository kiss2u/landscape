use std::collections::HashMap;
use std::sync::Arc;

use bollard::query_parameters::CreateImageOptions;
use bollard::secret::CreateImageInfo;
use bollard::Docker;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use landscape_common::docker::image::{
    ImgPullEvent, PullImgTask, PullImgTaskItem, PullManagerInfo,
};
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct PullManager {
    sock_tx: broadcast::Sender<ImgPullEvent>,
    tasks: Arc<RwLock<HashMap<String, Arc<PullImgTask>>>>,
}

impl PullManager {
    pub fn new() -> Self {
        let (sock_tx, _) = broadcast::channel(2048);

        Self {
            sock_tx,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_info(&self) -> PullManagerInfo {
        let mut inners = vec![];
        {
            let read_outer = self.tasks.read().await;
            for (_, value) in read_outer.iter() {
                inners.push(value.clone());
            }
        }

        let mut result = HashMap::new();
        for item in inners {
            let img_name = item.img_name.clone();
            let read_lock = item.layer_current_info.read().await;
            result.insert(img_name, read_lock.clone());
            drop(read_lock);
        }

        PullManagerInfo { tasks: result }
    }

    pub fn get_event_sock(&self) -> broadcast::Receiver<ImgPullEvent> {
        self.sock_tx.subscribe()
    }

    pub async fn pull_img(&self, image_name: String) {
        let (split_image_name, image_tag) = if let Some((name, tag)) = image_name.split_once(':') {
            (name.to_string(), tag.to_string())
        } else {
            (image_name.to_string(), "latest".to_string())
        };

        let options = CreateImageOptions {
            from_image: Some(split_image_name),
            tag: Some(image_tag),
            ..Default::default()
        };

        let task_info = PullImgTask {
            img_name: image_name.clone(),
            layer_current_info: Arc::new(RwLock::new(HashMap::new())),
        };

        let item_map = task_info.layer_current_info.clone();
        {
            let mut write = self.tasks.write().await;
            write.insert(image_name.clone(), Arc::new(task_info));
            drop(write);
        }
        let sock_tx = self.sock_tx.clone();
        tokio::spawn(async move {
            let docker = Docker::connect_with_local_defaults().unwrap();

            let mut stream = docker.create_image(Some(options), None, None);

            while let Some(res) = stream.next().await {
                match res {
                    Ok(CreateImageInfo {
                        id,
                        status: Some(_),
                        progress: Some(_),
                        progress_detail: Some(progress_detail),
                        ..
                    }) => {
                        let mut write = item_map.write().await;
                        let info = write.entry(id.clone()).or_insert(Default::default());

                        *info = PullImgTaskItem {
                            id: id.clone(),
                            current: progress_detail.current,
                            total: progress_detail.total,
                        };
                        let _ = sock_tx.send(ImgPullEvent {
                            img_name: image_name.clone(),
                            id: id.clone(),
                            current: progress_detail.current,
                            total: progress_detail.total,
                        });
                        // println!("[拉取中: {id:?}] {}: {}{:?}", status, progress, progress_detail);
                    }
                    Ok(CreateImageInfo { id, status: Some(status), .. }) => {
                        println!("[status: {id:?}] {}", status)
                    }

                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("fail: {}", e);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {

    use tokio::sync::broadcast;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test1() {
        // Create a broadcast channel with a capacity of 5 messages
        let (tx, _) = broadcast::channel::<String>(5);

        // Start 3 consumers
        for id in 0..3 {
            let mut rx = tx.subscribe();
            tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(msg) => {
                            println!("Consumer {} received: {}", id, msg);
                            sleep(Duration::from_millis(300)).await; // Simulate slow consumption
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            println!("Consumer {} lagged behind by {} messages!", id, n);
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        // Simulate producer
        for i in 1..=20 {
            let msg = format!("Message {}", i);
            println!("Producer sends: {}", msg);
            tx.send(msg).unwrap();
            sleep(Duration::from_millis(100)).await;
        }
    }
}
