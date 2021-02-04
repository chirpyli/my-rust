use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use futures::future::{BoxFuture, FutureExt};
use futures::Future;
use futures::task::{waker_ref, ArcWake};
use std::task::{Context, Poll};

/// 自己实现一个简单的调度器，最简单的调度器就是单线程调度，接收任务放到一个队列中，然后不断从队列中取任务执行。
/// 能想到的最简单实现就是利用channel，唤醒其实就是sender，调度就是receiver。

pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
    pub fn run(&self) {
        let mut id = 0;
        loop {
            id = id + 1;
            let task = self.ready_queue.recv().unwrap();
            info!("executor new task. id: {}. pointer: {:?}", id, Arc::as_ptr(&task));
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                // 调度执行future.
                if let Poll::Pending = future.as_mut().poll(context) {
                    // 如果是未就绪，就重新放入，待以后就绪后再被调度
                    info!("not ready, pending.");
                    *future_slot = Some(future);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        info!("spawn a new task to execute. pointer: {:?}", Arc::as_ptr(&task));
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

/// 供调度器调度执行，里面封的是future.
struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    /// 唤醒一个任务，其实就是把它丢到任务队列中去调度执行。
    fn wake_by_ref(arc_self: &Arc<Self>) {
        info!("prepare to wake the task.");
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("too many tasks queued");
    }
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    let (task_sender, ready_queue) = sync_channel(10);
    (Executor { ready_queue }, Spawner { task_sender } )
}