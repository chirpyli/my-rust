对异步的学习，我们先从Future开始，学习异步的实现原理。等理解了异步是怎么实现的后，再学习Rust异步编程涉及的2个库（futures、tokio）的时候就容易理解多了。

### Future
rust中`Future`的定义如下，一个`Future`可以理解为一段供将来调度执行的代码。我们为什么需要异步呢，异步相比同步高效在哪里呢？就是异步环境下，当前调用就绪时则执行，没有就绪时则不等待任务就绪，而是返回一个`Future`，等待将来任务就绪时再调度执行。当然，这里返回`Future`时关键的是要声明事件什么时候就绪，就绪后怎么唤醒这个任务到调度器去调度执行。
```rust
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[lang = "future_trait"]
pub trait Future {  // A future represents an asynchronous computation.
    type Output;
    /* The core method of future, poll, attempts to resolve the future into a final value. This method does not block if the value is not ready. Instead, the current task is scheduled to be woken up when it's possible to make further progress by polling again. */ 
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
}
```
可以看到执行后的返回结果，一个是就绪返回执行结果，另一个是未就绪待定。
```rust
#[must_use = "this `Poll` may be a `Pending` variant, which should be handled"]
pub enum Poll<T> {
    Ready(T),
    Pending,
}
```
可能到这里你还是云里雾里，我们写一段代码，帮助你理解。完整代码见：[future_study](./future_study)
```rust
use futures;
use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread, time::Duration};

fn main() {
    // 我们现在还没有实现调度器，所以要用一下futues库里的一个调度器。
    futures::executor::block_on(TimerFuture::new(Duration::new(10, 0)));    
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

// 我们想要实现一个定时器Future
pub struct TimerFuture {
    share_state: Arc<Mutex<SharedState>>,
}

// impl Future trait for TimerFuture.
impl Future for TimerFuture {
    type Output = ();
    // executor will run this poll ,and Context is to tell future how to wakeup the task.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut share_state = self.share_state.lock().unwrap();
        if share_state.completed {
            println!("future ready. execute poll to return.");
            Poll::Ready(())
        } else {
            println!("future not ready, tell the future task how to wakeup to executor");
            // 你要告诉future，当事件就绪后怎么唤醒任务去调度执行，而这个waker根具体的调度器有关
            // 调度器执行的时候会将上下文信息传进来，里面最重要的一项就是Waker
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
```
执行结果如下：
```
future not ready, tell the future task how to wakeup to executor
detect future is ready, wakeup the future task to executor.
future ready. execute poll to return.
```
可以看到，刚开始的时候，定时10s事件还未完成，处在`Pending`状态，这时要告诉这个任务后面就绪后怎么唤醒去调度执行。等10s后，定时事件完成了，通过前面的设置的`Waker`，唤醒这个`Future`任务去调度执行。这里，我们看一下`Context`和`Waker`是怎么定义的：
```rust
/// The `Context` of an asynchronous task.
///
/// Currently, `Context` only serves to provide access to a `&Waker`
/// which can be used to wake the current task.
#[stable(feature = "futures_api", since = "1.36.0")]
pub struct Context<'a> {
    waker: &'a Waker,
    // Ensure we future-proof against variance changes by forcing
    // the lifetime to be invariant (argument-position lifetimes
    // are contravariant while return-position lifetimes are
    // covariant).
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

// A Waker is a handle for waking up a task by notifying its executor that it is ready to be run.
#[repr(transparent)]
#[stable(feature = "futures_api", since = "1.36.0")]
pub struct Waker {
    waker: RawWaker,
}
```

现在你应该对`Future`有新的理解了，上面的代码，我们并没有实现调度器，而是使用的`futures`库中提供的一个调度器去执行，下面自己实现一个调度器，看一下它的原理。而在Rust中，真正要用的话，还是要学习`tokio`库，这里我们只是为了讲述一下实现原理，以便于理解异步是怎么一回事。完整代码见：[future_study](./future_study)， 关键代码如下：
```rust
use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread, time::Duration};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}};

use super::timefuture::*;

pub fn run_executor() {
    let (executor, spawner) = new_executor_and_spawner();
    // 将Future封装成一个任务，分发到调度器去执行
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
    // 实际运行具体的Future任务，不断的接收Future task执行。
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

// 负责将一个Future封装成一个Task，分发到调度器去执行。
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

// 等待调度执行的Future任务，这个任务必须要实现ArcWake，表明怎么去唤醒任务去调度执行。
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
```
运行结果如下：
```
first dispatch the future task to executor.     
received task. 1                                
future not ready, tell the future task how to wakeup to executor
executor run the future task, but is not ready, create a future again.
detect future is ready, wakeup the future task to executor.     
received task. 2
future ready. execute poll to return.
return value: timer done.
executor run the future task, is ready. the future task is done.
```
第一次调度的时候，因为还没有就绪，在Pending状态，告诉这个任务，后面就绪是怎么唤醒该任务。然后当事件就绪的时候，因为前面告诉了如何唤醒，按方法唤醒了该任务去调度执行。其实，在实际应用场景中，难的地方还在于，你怎么知道什么时候事件就绪，去唤醒任务，我们很容易联想到Linux系统的epoll，tokio等底层，也是基于epoll实现的。通过epoll，我们就能方便的知道事件什么时候就绪了。



---

### 参考资料
主要学习资料如下：
- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html)
- [Futures Explained in 200 Lines of Rust](https://cfsamson.github.io/books-futures-explained/introduction.html)
- [200行代码讲透RUST FUTURES](https://stevenbai.top/rust/futures_explained_in_200_lines_of_rust/)

上面的文章主要是学习异步的实现原理，理解异步是怎么实现的，而进行Rust异步编程时的具体实现，则主要依赖下面2个库：
- [future](https://docs.rs/futures/0.3.5/futures/) —— 主要完成了对异步的抽象
- [tokio](https://docs.rs/tokio/0.2.20/tokio/) —— 异步Future运行时
学习这两个库的时候，一定要注意版本问题，这两个库最近变化的比较快，一定要学最新的。
