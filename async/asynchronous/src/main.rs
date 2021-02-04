// 自己实现future。

#[macro_use]
extern crate log;

use crate::myfuture::TimerFuture;
use std::time::Duration;

mod myfuture;
mod executor;

fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
    let (executor, spawner) = executor::new_executor_and_spawner();
    spawner.spawn( async {
        info!("spawn a timer future.");
        TimerFuture::new(Duration::new(10, 0)).await;
        info!("task done.");
    });

    // drop(spawner);

    executor.run();
}


/*
创建一个future任务，去执行，发现没有就绪，挂起，等待就绪，唤醒，执行。
2020-11-17 16:05:54,135 INFO  [asynchronous::executor] spawn a new task to execute.
2020-11-17 16:05:54,135 INFO  [asynchronous::executor] executor new task 1.
2020-11-17 16:05:54,135 INFO  [asynchronous] spawn a timer future.
2020-11-17 16:05:54,135 INFO  [asynchronous::executor] not ready, pending.
2020-11-17 16:06:04,136 INFO  [asynchronous::myfuture] timer is done. to wake the task.
2020-11-17 16:06:04,136 INFO  [asynchronous::executor] prepare to wake the task.
2020-11-17 16:06:04,136 INFO  [asynchronous::executor] executor new task 2.
2020-11-17 16:06:04,136 INFO  [asynchronous] task done.

*/