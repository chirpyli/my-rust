
#[macro_use]
extern crate log;

mod executor;
mod time_future;

fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
    info!("build your own block_on.");

    executor::block_on( async {
        let f = time_future::TimerFuture::new(std::time::Duration::from_secs(3));
        f.await;
        info!("a future wait 3 s done.");
    });

}

/*
2021-02-05 17:15:06,010 INFO  [block_on] build your own block_on.
2021-02-05 17:15:06,010 INFO  [block_on::executor] create a raw waker.
2021-02-05 17:15:06,010 INFO  [block_on::time_future] time future is not ready.
2021-02-05 17:15:06,010 INFO  [block_on::executor] raw waker vtable clone
2021-02-05 17:15:06,010 INFO  [block_on::executor] create a raw waker.
2021-02-05 17:15:06,010 INFO  [block_on::executor] future is not ready, register waker, wait util ready.
2021-02-05 17:15:09,015 INFO  [block_on::time_future] timer is done. to wake the task.
2021-02-05 17:15:09,016 INFO  [block_on::executor] raw waker vtable wake
2021-02-05 17:15:09,016 INFO  [block_on::executor] wake instance call wake, unpark thread.
2021-02-05 17:15:09,016 INFO  [block_on::time_future] time future is ready.
2021-02-05 17:15:09,016 INFO  [block_on] a future wait 3 s done.
2021-02-05 17:15:09,016 INFO  [block_on::executor] future is ready, return is final result.
2021-02-05 17:15:09,016 INFO  [block_on::executor] raw waker vtable drop
*/

