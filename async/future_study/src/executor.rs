use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread, time::Duration};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}};

use super::timefuture::*;


pub fn run_executor() {
    let (executor, spawner) = new_executor_and_spawner();
    spawner.spawn( async {
        let v = TimerFuture::new(Duration::new(10, 0)).await;
        println!("return value: {}", v);
        v
    });

    drop(spawner);
    executor.run();
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUE_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUE_TASKS);
    (Executor{ready_queue}, Spawner{task_sender})
}

// executor , received ready task to execute.
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
    fn run(&self) {
        let mut count = 0;
        while let Ok(task) = self.ready_queue.recv() {
            count = count + 1;
            println!("received task. {}", count);
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                if let Poll::Pending = future.as_mut().poll(context) {
                    *future_slot = Some(future);
                    println!("executor run the future task, but is not ready, create a future again.");
                } else {
                    println!("executor run the future task, is ready. the future task is done.");
                }
            }
        }
    }
}


#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    // encapsul a future object to task , wakeup to executor.
    fn spawn(&self, future: impl Future<Output = String> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        println!("first dispatch the future task to executor.");
        self.task_sender.send(task).expect("too many tasks queued.");
    }
}

// future task.
struct Task {
    future: Mutex<Option<BoxFuture<'static, String>>>,
    task_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    // A way of waking up a specific task.
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let clone = arc_self.clone();
        arc_self.task_sender.send(clone).expect("too many tasks queued");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread, time::Duration};
    use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
    use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}};


    #[test]
    fn test_executor() {
        run_executor();
    }
}
