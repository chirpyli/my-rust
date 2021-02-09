use std::sync::Arc;
use std::future::Future;
use std::task::{Waker, Poll, Context, RawWaker, RawWakerVTable};
use std::pin::Pin;
use futures::channel::oneshot;
use pin_utils::core_reexport::sync::atomic::AtomicUsize;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use once_cell::sync::Lazy;
use futures::FutureExt;

// 这里的状态很粗，更多状态可以参考 async-task库中的实现
const WOKEN: usize = 0b01;
const RUNNING: usize = 0b10;
const TASK: usize = 0b100;

static QUEUE: Lazy<crossbeam::channel::Sender<Arc<Task>>> = Lazy::new(|| {
    info!("create static queue with lazy.");
    let (s, r) = crossbeam::channel::unbounded::<Arc<Task>>();
    let num_cores = num_cpus::get();
    for i in 0..num_cores {
        let receiver = r.clone();
        std::thread::spawn(move || {
            receiver.iter().for_each(|task| task.run());
        });
    }

    s
});

// 这里以及后面暂时忽略wake与wake_ref的区别
trait Wake: Clone {
    fn wake(&self);
}

#[derive(Clone)]
struct OwnWeaker {
    inner: Arc<Task>,       // 之所以在本实现中inner中含有Task，是因为Waker需要知道去唤醒具体哪一个Task
}

impl OwnWeaker {
    pub fn new(task: Arc<Task>) -> Self {
        Self {
            inner: task
        }
    }
}

impl Wake for OwnWeaker {
    // 对应本Executor的实现，唤醒任务就是将本任务发送到任务队列中
    fn wake(&self) {
        // 更新状态
        self.inner.state.store(WOKEN, Ordering::SeqCst);
        QUEUE.send(self.inner.clone()).unwrap();
        // if self.inner.state.fetch_or(WOKEN, Ordering::SeqCst) == 0 {
        //     QUEUE.send(self.inner.clone()).unwrap()
        // }
    }
}

fn create_waker<W: Wake>(wake: W) -> Waker {
    let raw_waker = create_raw_waker(wake);
    unsafe {
        Waker::from_raw(raw_waker)
    }
}

fn create_raw_waker<W: Wake>(wake: W) -> RawWaker {
    let data = Box::new(wake);
    RawWaker::new(Box::into_raw(data) as *const (), &RawWakerVTable::new(
        |data| unsafe {
            create_raw_waker((&*(data as *const W)).clone())
        },
        |data| unsafe {
            (&*(data as *const W)).wake()
        },
        |data| unsafe {
            (&*(data as *const W)).wake()
        },
        |data| unsafe {
            info!("raw waker drop, do nothing, tmp");
        },
    ))
}


// An owned permission to join on a task (await its termination)
// type JoinHandle<R> = Pin<Box<dyn Future<Output = R>>>;
pub struct JoinHandle<R> {
    inner: Pin<Box<dyn Future<Output = R>>>,
}

// Spawns a new asynchronous task, returning a JoinHandle for it.
// Spawning a task enables the task to execute concurrently to other tasks.
// The spawned task may execute on the current thread, or it may be sent to a different thread to be executed.
pub fn spawn<F, R>(future: F) -> JoinHandle<R>
    where F: Future<Output = R> + Send + 'static, R: Send + 'static {
    let (s, r) = oneshot::channel();
    let future = async move { let _ = s.send(future.await);};

    let task = Arc::new(Task {
        state: AtomicUsize::new(0),
        future: Mutex::new(Box::pin(future)),
    });

    QUEUE.send(task).unwrap();

    JoinHandle {
        inner: Box::pin(async { r.await.unwrap()})
    }
}


impl<R: Send> Future for JoinHandle<R> {
    type Output = R;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let handle = Pin::into_inner(self);
        handle.inner.as_mut().poll(cx)
    }
}

pub struct Task {
    state: AtomicUsize,
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

impl Task {
    pub fn run(self: Arc<Task>) {
        self.state.store(RUNNING, Ordering::SeqCst);
        let waker = create_waker(OwnWeaker::new(self.clone()));
        let cx = &mut Context::from_waker(&waker);
        let poll = self.future.try_lock().unwrap().as_mut().poll(cx);
        // 如果没有就绪的话，要更新状态，擦掉RUNNING的状态
        if poll.is_pending() {
            self.state.store(TASK, Ordering::SeqCst);
        }
    }
}