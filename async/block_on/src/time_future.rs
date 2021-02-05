//! 自己实现一个Time Future
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
        // 这一块相当与Reactor，需要注册waker，以唤醒future task
        thread::spawn( move || {
            thread::sleep(duration);
            let mut lock = arc_state.lock().unwrap();
            lock.completed = true;
            if let Some(waker) = lock.waker.take() {
                info!("timer is done. to wake the task.");
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
            info!("time future is ready.");
            Poll::Ready(())
        } else {
            // 定时时间没到，未就绪状态，并且要告诉怎么唤醒它
            info!("time future is not ready.");
            shared_state.waker = Some(cx.waker().clone());      // 实现future的地方要向Reactor注册waker，Reactor可以是epoll，或者timer等
            Poll::Pending
        }
    }
}