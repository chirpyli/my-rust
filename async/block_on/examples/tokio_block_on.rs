// tokio block_on example.

use tokio::time::Duration;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build().unwrap();

    runtime.block_on(hello());

}

async fn hello() {
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("hello world.");
}