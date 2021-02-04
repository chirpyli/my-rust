
/// https://rust-lang.github.io/async-book/02_execution/02_future.html
/// 自己定义一个Future，其实就是一个供将来调用的一个接口。未来执行的一个任务。
/// trait SimpleFuture {
///     type Output;
///
///     /// 调度器去调度， 如果是未就绪，要告诉它怎么在就绪时唤醒它
///     fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
/// }
/// 执行调用后的结果有两种
/// enum Poll<T> {
///     Ready(T),       // 就绪状态
///     Pending,        // 未就绪
/// }
/// 但是仅仅这样是不够的，wake还需要一定的参数去明确唤醒的是哪一个任务

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let arc_state = shared_state.clone();
        thread::spawn( move || {
            thread::sleep(duration);
            let mut lock = arc_state.lock().unwrap();
            lock.completed = true;
            if let Some(waker) = lock.waker.take() {
                info!("timer is done. to wake the task.");
                // waker.wake(),查看调度器相关的代码就知道唤醒其实就是send到任务队列中。
                waker.wake()    // 前面已满足就绪条件，唤醒任务，其实就是让调度器调度该任务并执行
            }
        });

        TimerFuture { shared_state }
    }
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        // 如果定时时间到，就绪状态
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // 定时时间没到，未就绪状态，并且要告诉怎么唤醒它
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}