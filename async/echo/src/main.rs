#[macro_use]
extern crate log;

use futures::join;

mod pinning;

fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    info!("{}-{}.", name, version);

    let runtime = tokio::runtime::Builder::new_multi_thread().enable_time().enable_io().build().unwrap();
    runtime.block_on(async {
        info!("echo example.");
        pinning::run_pinning().await;
        join!(task1(), task2());
    });

    info!("echo end.");
}

async fn task1() {
    info!("task one.");
}

async fn task2() {
    info!("task two.");
}