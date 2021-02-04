use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread, time::Duration};

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

pub struct TimerFuture {
    share_state: Arc<Mutex<SharedState>>,
}

// impl Future trait for TimerFuture.
impl Future for TimerFuture {
    type Output = String;
    // executor will run this poll ,and Context is to tell future how to wakeup the task.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut share_state = self.share_state.lock().unwrap();
        if share_state.completed {
            println!("future ready. execute poll to return.");
            Poll::Ready(String::from("timer done."))
        } else {
            println!("future not ready, tell the future task how to wakeup to executor");
            share_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let share_state = Arc::new(Mutex::new(SharedState{completed:false, waker:None}));
        let thread_shared_state = share_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut share_state = thread_shared_state.lock().unwrap();
            share_state.completed = true;
            if let Some(waker) = share_state.waker.take() {
                println!("detect future is ready, wakeup the future task to executor.");
                waker.wake()    // wakeup the future task to executor.
            }
        });

        TimerFuture {share_state}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread, time::Duration};
    use futures;

    #[test]
    fn timerfuture() {
        futures::executor::block_on(TimerFuture::new(Duration::new(10, 0)));
    }
}
