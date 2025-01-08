use landscape::get_all_devices;

#[tokio::main]
async fn main() {
    let result = get_all_devices().await;
    println!("{:?}", result);
}
