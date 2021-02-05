use std::future::Future;
use std::task::{Poll, Context, Waker, RawWaker, RawWakerVTable};


pub trait Wake: Clone {
    fn wake(&self);
}

/*
clone: unsafe fn(*const ()) -> RawWaker,
wake: unsafe fn(*const ()),
wake_by_ref: unsafe fn(*const ()),
drop: unsafe fn(*const ()),
*/
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
                Box::from_raw(data as *mut W).wake()
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
        self.inner.unpark();
    }
}


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
