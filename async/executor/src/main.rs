//! build my own async executor
#[macro_use]
extern crate log;

mod executor;

use custom_futures::TimerFuture;


fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
    futures::executor::block_on( test_run());
}

async fn test_run() {
    info!("test my own executor.");

    let h1 = executor::spawn(async {
        info!("spawn task 1.");
        let timer = TimerFuture::new(std::time::Duration::new(3, 0));
        timer.await
    });

    // let h2 = executor::spawn(async {
    //     info!("spawn task 2.");
    //     let timer = TimerFuture::new(std::time::Duration::new(10, 0));
    //     timer.await
    // });
    //
    // let h3 = executor::spawn( async {
    //     info!("spawn task3.");
    //     1 + 2
    // });

    let r1 = h1.await;
    // let r2 = h2.await;
    // let r3 = h3.await;
}
