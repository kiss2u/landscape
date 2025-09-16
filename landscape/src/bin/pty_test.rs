use landscape::sys_service::web_pty::LandscapePtySession;
use landscape_common::pty::{LandscapePtyConfig, LandscapePtySize, PtyInMessage};

#[tokio::main]
async fn main() {
    let config = LandscapePtyConfig::default();
    let session = LandscapePtySession::new(config).await.unwrap();

    let mut out = session.out_events.subscribe();
    tokio::spawn(async move {
        while let Ok(data) = out.recv().await {
            match data {
                landscape_common::pty::PtyOutMessage::Data { data } => {
                    println!("{:?}", String::from_utf8_lossy(&data))
                }
                landscape_common::pty::PtyOutMessage::Exit { msg } => {
                    println!("{:?}", msg)
                }
            };
        }
    });

    session
        .input_events
        .send(PtyInMessage::Data { data: Box::new("ls -la\n".as_bytes().to_vec()) })
        .await
        .unwrap();

    session
        .input_events
        .send(PtyInMessage::Size {
            size: LandscapePtySize { rows: 5, cols: 5, pixel_width: 0, pixel_height: 0 },
        })
        .await
        .unwrap();

    session
        .input_events
        .send(PtyInMessage::Data { data: Box::new("ls -la\n".as_bytes().to_vec()) })
        .await
        .unwrap();

    tokio::signal::ctrl_c().await.unwrap();
}
