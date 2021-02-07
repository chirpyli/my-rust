//! build my own async executor
#[macro_use]
extern crate log;

mod executor;

use custom_futures::TimerFuture;

fn main() {
    futures::executor::block_on(async {
        let timer = TimerFuture::new(std::time::Duration::from_secs(3));
        timer.await;
    });
}
