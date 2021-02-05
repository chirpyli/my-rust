### 一个`block_on`代码示例

我们在进行异步编程时，经常会有下面形式的代码：
```rust
use tokio::time::Duration;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build().unwrap();

    runtime.block_on(hello());
}

async fn hello() {
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("hello world.");
}
```
我们看一下tokio中关于`block_on`的定义：
```rust
/// Run a future to completion on the Tokio runtime. This is the runtime's entry point.
/// This runs the given future on the runtime, blocking until it is complete, and yielding its resolved result. 
/// Any tasks or timers which the future spawns internally will be executed on the runtime.
pub fn block_on<F: Future>(&self, future: F) -> F::Output
```
`block_on`正如其名，阻塞在一个`Future`，直到该`Future`就绪并完成。 在tokio中内部是一个线程池，我们先不看tokio中是怎么实现的，我们先想一下如果自己来实现，该如何做呢？

### 自己构建`block_on`
如果构建`block_on`呢？ 我们需要实现如下一个函数，执行一个`Future`，直到就绪运行输入结果。
```rust
fn block_on<F: Future>(future: F) -> F::Output {
    todo!()
}
```
接下来，根据其语义，要在`block_on`中实现运行`Future`直到其就绪完成。这里我们是最简单的实现，所有实现思路就是如果发现`Future`未就绪，就阻塞当前线程，当发现`Futute`就绪再唤醒当前线程。所以有下面的代码：
```rust
pub block_on<F: Future>(future: F) -> F::Output {
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(t) => {
                info!("future is ready, return is final result.");
                return t;
            },
            Poll::Pending => {
                info!("future is not ready, register waker, wait util ready.");
                std::thread::park();        
            }
        }
    }
}
```
因为`fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;`，这个`Future`必须是`Pin<&mut Future>`，所有有如下的代码：
```rust
pub fn block_on<F: Future>(future: F) -> F::Output {
    pin_utils::pin_mut!(future);    // convert self to Pin<&mut Self>. 因为poll(self: Pin<&mut Self>, cx: &mut Context<'_>) ，所以必须将future钉住
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(t) => {
                info!("future is ready, return is final result.");
                return t;
            },
            Poll::Pending => {
                info!("future is not ready, register waker, wait util ready.");
                std::thread::park();
            }
        }
    }
}
```
为什么必须要`Pin`呢？涉及到Rust自引用结构体的问题，待以后再讨论这个问题。实现到这里之后，当前的关键问题是怎么实现`Waker`。我们要告诉
`Reactor`怎么唤醒任务，而`Waker`的实现关键是看`Executor`，不同的`Executor`有不同的唤醒方式，比如我们之前实现的`Executor`唤醒方式就是向任务队列中推送`Future`任务，等待具体的执行线程去从队列中
取任务执行。而这里的`Waker`唤醒方式，则是唤醒当前正在阻塞的线程就可以了，所以我们必须自己实现`Waker`。我们看一下其定义：
```rust
/// A Waker is a handle for waking up a task by notifying its executor that it is ready to be run.
pub struct Waker {
    waker: RawWaker,
}

impl Waker {
    /// Wake up the task associated with this Waker.
    pub fn wake(self) {
        // The actual wakeup call is delegated through a virtual function call
        // to the implementation which is defined by the executor.
        let wake = self.waker.vtable.wake;
        let data = self.waker.data;

        // Don't call `drop` -- the waker will be consumed by `wake`.
        crate::mem::forget(self);

        // SAFETY: This is safe because `Waker::from_raw` is the only way
        // to initialize `wake` and `data` requiring the user to acknowledge
        // that the contract of `RawWaker` is upheld.
        unsafe { (wake)(data) };
    }

    /// Wake up the task associated with this Waker without consuming the Waker.
    pub fn wake_by_ref(&self) {
        // The actual wakeup call is delegated through a virtual function call
        // to the implementation which is defined by the executor.

        // SAFETY: see `wake`
        unsafe { (self.waker.vtable.wake_by_ref)(self.waker.data) }
    }


    /// Creates a new Waker from RawWaker.
    pub unsafe fn from_raw(waker: RawWaker) -> Waker {
        Waker { waker }
    }
    
    // ... others ...

}

impl Clone for Waker {
    fn clone(&self) -> Self {
        Waker {
            // SAFETY: This is safe because `Waker::from_raw` is the only way
            // to initialize `clone` and `data` requiring the user to acknowledge
            // that the contract of [`RawWaker`] is upheld.
            waker: unsafe { (self.waker.vtable.clone)(self.waker.data) },       
        }
    }
}

impl Drop for Waker {
    fn drop(&mut self) {
        // SAFETY: This is safe because `Waker::from_raw` is the only way
        // to initialize `drop` and `data` requiring the user to acknowledge
        // that the contract of `RawWaker` is upheld.
        unsafe { (self.waker.vtable.drop)(self.waker.data) }        // 调用下面具体的虚函数实现，当然rust中并没有virtual关键字，可以理解为是虚函数的实现形式
    }
}
```
定义了`wake()`方法去唤醒任务。我们继续看其内部：
```rust
pub struct RawWaker {
    /// A data pointer, which can be used to store arbitrary data as required
    /// by the executor. This could be e.g. a type-erased pointer to an `Arc`
    /// that is associated with the task.
    /// The value of this field gets passed to all functions that are part of
    /// the vtable as the first parameter.
    data: *const (),        // data是waker的具体的实例，上层抽象为Waker，但不同Executor的Waker的具体实现不同，不同体现在下面的虚函数表中
    /// Virtual function pointer table that customizes the behavior of this waker.
    vtable: &'static RawWakerVTable,        // 可以认为是虚函数的具体实现
}

/// A virtual function pointer table (vtable) that specifies the behavior of a RawWaker.
pub struct RawWakerVTable {
    clone: unsafe fn(*const ()) -> RawWaker,
    wake: unsafe fn(*const ()),  // This function will be called when `wake` is called on the Waker.
    wake_by_ref: unsafe fn(*const ()),
    drop: unsafe fn(*const ()),  // This function gets called when a RawWaker gets dropped.
}
```
也就是说我们如果要实现`Waker`，就要自定义适用于本`Executor`的`clone, wake, wake_by_ref, drop`这4种方法。最核心的当然是`wake`方法了，这里的`wake`其实就是唤醒当前线程，即有下面的代码：
```rust
pub trait Wake: Clone {
    fn wake(&self);
}

#[derive(Clone)]
struct WakeInstance {
    inner: std::thread::Thread,
}

impl WakeInstance {
    pub fn new(thread: std::thread::Thread) -> Self {
        Self {
            inner: thread
        }
    }
}

impl Wake for WakeInstance {
    fn wake(&self) {
        info!("wake instance call wake, unpark thread.");
        self.inner.unpark();        // 唤醒线程
    }
}
```
然后我们自定义其实现方法：
```rust
// 从一个Wake实例中产生RawWaker，继而产生Waker
fn create_raw_waker<W: Wake>(wake: W) -> RawWaker {
    info!("create a raw waker.");
    RawWaker::new(
        Box::into_raw(Box::new(wake)) as *const(),
        &RawWakerVTable::new(
            |data| unsafe {
                info!("raw waker vtable clone");
                create_raw_waker((&*(data as *const W)).clone())    // 把data克隆一份(要求泛型W必须实现Clone Trait)，重新生成RawWaker
            },
            |data| unsafe {
                info!("raw waker vtable wake");
                Box::from_raw(data as *mut W).wake()        // data就是wake实例， 调用wake实例的wake方法唤醒线程
            },
            |data| unsafe {
                info!("raw waker vtable wake_by_ref");
                (&*(data as *const W)).wake()
            },
            |data| unsafe {
                info!("raw waker vtable drop");
                drop(Box::from_raw(data as *mut W))
            }
        )
    )
}
```
这块怎么理解呢？看一下这个就明白了：
```rust
    pub fn wake(self) {
        // The actual wakeup call is delegated through a virtual function call
        // to the implementation which is defined by the executor.
        let wake = self.waker.vtable.wake;
        let data = self.waker.data;

        // Don't call `drop` -- the waker will be consumed by `wake`.
        crate::mem::forget(self);

        // SAFETY: This is safe because `Waker::from_raw` is the only way
        // to initialize `wake` and `data` requiring the user to acknowledge
        // that the contract of `RawWaker` is upheld.
        unsafe { (wake)(data) };
    }
```
其实与c++中类调用方法类似，这个`wake()` 等同于 `(self.waker.vtable.wake)(self.waker.data)`， 类似于c++中，`self.wake.data`是类对象object，其方法function为`self.waker.vtable.wake`，即`object.function()`
到这里，我们已经实现了`Waker`，有如下代码：
```rust
pub fn block_on<F: Future>(future: F) -> F::Output {
    pin_utils::pin_mut!(future);    // convert self to Pin<&mut Self>. 因为poll(self: Pin<&mut Self>, cx: &mut Context<'_>) ，所以必须将future钉住

    // 定义一个waker，如果future为未就绪的话，需要waker去唤醒
    // 不同的Executor有不同的waker实现，这里需要自定义waker,在本block_on的实现中,waker自然就是唤醒当前线程即可
    // 不同的waker实现有同一的接口实现，需要通过自定义虚函数实现，这里自己实现这一部分。
    let thread = std::thread::current();
    let wake_instance = WakeInstance::new(thread);

    let raw_waker = create_raw_waker(wake_instance);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(t) => {
                info!("future is ready, return is final result.");
                return t;
            },
            Poll::Pending => {
                info!("future is not ready, register waker, wait util ready.");
                std::thread::park();
            }
        }
    }
}
```

到这里，我们已经自己构造了一个`block_on`，我们跑一个例子来验证一下我们的代码：
```rust
#[macro_use]
extern crate log;

mod executor;
mod time_future;

fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
    info!("build your own block_on.");

    executor::block_on( async {
        let f = time_future::TimerFuture::new(std::time::Duration::from_secs(3));   // 自定义的time future 
        f.await;
        info!("a future wait 3 s done.");
    });

}
```
完整代码见[block_on](../block_on)，运行后有如下日志：
```log
2021-02-05 17:15:06,010 INFO  [block_on] build your own block_on.
2021-02-05 17:15:06,010 INFO  [block_on::executor] create a raw waker.      创建Waker
2021-02-05 17:15:06,010 INFO  [block_on::time_future] time future is not ready.     发现未就绪，等待
2021-02-05 17:15:06,010 INFO  [block_on::executor] raw waker vtable clone
2021-02-05 17:15:06,010 INFO  [block_on::executor] create a raw waker.
2021-02-05 17:15:06,010 INFO  [block_on::executor] future is not ready, register waker, wait util ready.
2021-02-05 17:15:09,015 INFO  [block_on::time_future] timer is done. to wake the task.
2021-02-05 17:15:09,016 INFO  [block_on::executor] raw waker vtable wake    调用自定义的Waker 唤醒线程
2021-02-05 17:15:09,016 INFO  [block_on::executor] wake instance call wake, unpark thread.
2021-02-05 17:15:09,016 INFO  [block_on::time_future] time future is ready.
2021-02-05 17:15:09,016 INFO  [block_on] a future wait 3 s done.
2021-02-05 17:15:09,016 INFO  [block_on::executor] future is ready, return is final result.
2021-02-05 17:15:09,016 INFO  [block_on::executor] raw waker vtable drop    
```
可以通过这个理解`Waker`是怎么一回事。



---

>参考文档：
>[Build your own block_on()](https://web.archive.org/web/20200511234503/https://stjepang.github.io/2020/01/25/build-your-own-block-on.html)
>[wakeful](https://github.com/sagebind/wakeful/blob/master/src/lib.rs)
