#### 一、线程池相关概念
线程池，就是一组工作线程，工作线程的数量一般与CPU核数相关（如果是CPU密集型任务，可初始设为`N+1`，如果是IO密集型任务，可初始设为`2*N`，`N`为CPU核数，运行过程中可能会依据任务的繁忙程度而动态增减），由线程池负责管理工作线程的创建，异常处理（如果工作线程异常退出，会创建新的工作线程弥补线程池中的工作线程数量），任务分配等工作。线程池中一般会有一个任务队列，所有工作线程从任务队列中取任务，执行，如此反复。其核心是避免大量线程的创建及频繁的线程切换，尽最大可能提高CPU利用率。

线程池有不同的实现形式，主要的区别就是如何指定因任务队列中任务的繁忙程度与调度管理工作线程的数量的调度策略。比如如果任务队列中有大量的任务等待处理，是否需要根据待处理任务队列的任务数量而开启新的工作线程去处理，等任务队列中的任务完成，再关闭部分工作线程。

线程池一般适用于大量短任务的处理，这样可以避免开启大量线程及频繁的线程切换，提高效率。如果是长时任务，则线程池的优势不明显，并且可能造成其他短任务（要求快速得到响应）得不到运行，造成饥饿。同时线程池不适用于有特定优先级的任务。


#### 二、使用示例
该示例实现了接收客户端的连接，并echo回应连接。使用mio+threadpool的方式。threadpool是rust的一个线程池库。
```rust
//! mio+threadpool
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate mio;
extern crate threadpool;
extern crate num_cpus;

use std::thread;
use std::str::FromStr;
use std::time::Duration;
use std::io::{Read,Write};
use threadpool::{ThreadPool,Builder};
use mio::*;
use mio::tcp::{TcpListener, TcpStream};

fn main() {
    simple_logger::init().unwrap();
    let server_handle=run_server(None);
    server_handle.join();
}

fn run_server(timeout: Option<Duration>)->thread::JoinHandle<()>{
    let handle=thread::spawn(move||{
        let num_cpus=num_cpus::get_physical();
        let pool=Builder::new().num_threads(num_cpus).thread_name(String::from("threadpool")).build();

        const SERVER: Token = Token(0);
        let addr = "127.0.0.1:12345".parse().unwrap();
        let server = TcpListener::bind(&addr).unwrap();

        let poll = Poll::new().unwrap();
        poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();
        let mut events = Events::with_capacity(1024);
        loop {
            match poll.poll(&mut events, timeout){
                Ok(size)=>{
                    trace!("event size={}",size);
                    if size<=0{
                        break;
                    }
                },
                Err(e)=>{
                    error!("{}",e);
                    break;
                }
            }
            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        let (stream,_) = server.accept().unwrap();
                        pool.execute(move ||{
                            simple_echo(stream);
                        });
                    },
                    _ => unreachable!(),
                }
            }
        }

        pool.join();
    });

    handle
}

fn simple_echo(mut stream:TcpStream) {
    info!("New accept {:?}", stream.peer_addr());
    let mut buf = String::new();
    if let Err(e) = stream.read_to_string(&mut buf) {
        error!("{}", e);
    }

    thread::sleep_ms(1000); //加上延时是为了验证线程池工作
    info!("server receive data: {}", buf);
    stream.write_all(buf.as_bytes());
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_server(){
        simple_logger::init().unwrap();
        let server_handle=run_server(Some(Duration::new(10,0)));
        thread::sleep_ms(1000);
        let client_handle=run_client(4);
        client_handle.join();
        server_handle.join();
    }

    fn run_client(num: usize)->thread::JoinHandle<()>{
        let handle=thread::spawn(move||{
            let mut ths=Vec::new();
            for id in 0..num{
                let h=thread::spawn(move||{
                    client(id);
                });
                ths.push(h);
            }

            for h in ths{
                h.join().unwrap();
            }
        });

        handle
    }

    fn client(id: usize){
        let mut stream = std::net::TcpStream::connect("127.0.0.1:12345").unwrap();
        let mut data=format!("client data {}",id);
        stream.write_all(data.as_bytes());
        let mut buffer=String::new();
        stream.read_to_string(&mut buffer);
        info!("client {} received data:{}",id,buffer);

        info!("connect {} end!",id);
    }
}


```

#### 三、Rust [threadpool源码](https://github.com/rust-threadpool/rust-threadpool)实现
该线程池实现了对工作线程的创建，线程异常panic处理，工作线程数量可运行时改变，但数量数需要具体指定，并没有实现随任务队列中任务繁忙程度而动态改变等功能。下面代码实现了线程池最基本的功能（对工作线程的管理），列出部分源码如下:
```rust
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Thunk<'a> = Box<FnBox + Send + 'a>;

```

Sentinel主要作用是检测出线程panic后，新建工作线程补充到线程池中。
```rust
struct Sentinel<'a> {
    shared_data: &'a Arc<ThreadPoolSharedData>,
    active: bool,
}

impl<'a> Sentinel<'a> {
    fn new(shared_data: &'a Arc<ThreadPoolSharedData>) -> Sentinel<'a> {
        Sentinel {
            shared_data: shared_data,
            active: true,
        }
    }

    /// Cancel and destroy this sentinel.
    fn cancel(mut self) {
        self.active = false;
    }
}

impl<'a> Drop for Sentinel<'a> {
    fn drop(&mut self) {
        if self.active {
            self.shared_data.active_count.fetch_sub(1, Ordering::SeqCst);
            if thread::panicking() {
                self.shared_data.panic_count.fetch_add(1, Ordering::SeqCst);
            }
            self.shared_data.no_work_notify_all();
            spawn_in_pool(self.shared_data.clone())
        }
    }
}

```

线程池建造者,负责构造线程池
```rust
/// [`ThreadPool`] factory, which can be used in order to configure the properties of the [`ThreadPool`].
#[derive(Clone, Default)]
pub struct Builder {
    num_threads: Option<usize>,
    thread_name: Option<String>,
    thread_stack_size: Option<usize>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            num_threads: None,
            thread_name: None,
            thread_stack_size: None,
        }
    }

    ...

    // Finalize the Builder and build the ThreadPool.
    pub fn build(self) -> ThreadPool {
        let (tx, rx) = channel::<Thunk<'static>>();
        let num_threads = self.num_threads.unwrap_or_else(num_cpus::get);
        let shared_data = Arc::new(ThreadPoolSharedData {
            name: self.thread_name,
            job_receiver: Mutex::new(rx),
            empty_condvar: Condvar::new(),
            empty_trigger: Mutex::new(()),
            join_generation: AtomicUsize::new(0),
            queued_count: AtomicUsize::new(0),
            active_count: AtomicUsize::new(0),
            max_thread_count: AtomicUsize::new(num_threads),
            panic_count: AtomicUsize::new(0),
            stack_size: self.thread_stack_size,
        });

        for _ in 0..num_threads {
            spawn_in_pool(shared_data.clone());
        }

        ThreadPool {
            jobs: tx,
            shared_data: shared_data,
        }
    }
}
```

```rust
struct ThreadPoolSharedData {
    name: Option<String>,
    job_receiver: Mutex<Receiver<Thunk<'static>>>,
    empty_trigger: Mutex<()>,
    empty_condvar: Condvar,
    join_generation: AtomicUsize,
    queued_count: AtomicUsize,
    active_count: AtomicUsize,
    max_thread_count: AtomicUsize,
    panic_count: AtomicUsize,
    stack_size: Option<usize>,
}

impl ThreadPoolSharedData {
    fn has_work(&self) -> bool {
        self.queued_count.load(Ordering::SeqCst) > 0 || self.active_count.load(Ordering::SeqCst) > 0
    }

    /// Notify all observers joining this pool if there is no more work to do.
    fn no_work_notify_all(&self) {
        if !self.has_work() {
            *self.empty_trigger.lock().expect(
                "Unable to notify all joining threads",
            );
            self.empty_condvar.notify_all();
        }
    }
}

```

线程池结构体
```rust
// Abstraction of a thread pool for basic parallelism.
pub struct ThreadPool {
    // How the threadpool communicates with subthreads.
    // This is the only such Sender, so when it is dropped all subthreads will quit.
    jobs: Sender<Thunk<'static>>,
    shared_data: Arc<ThreadPoolSharedData>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> ThreadPool {
        Builder::new().num_threads(num_threads).build()
    }

    // Executes the function `job` on a thread in the pool.
    pub fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.shared_data.queued_count.fetch_add(1, Ordering::SeqCst);
        self.jobs.send(Box::new(job)).expect( "ThreadPool::execute unable to send job into queue.");
    }

    /// Returns the number of jobs waiting to executed in the pool.
    pub fn queued_count(&self) -> usize {
        self.shared_data.queued_count.load(Ordering::Relaxed)
    }

    /// **Deprecated: Use [`ThreadPool::set_num_threads`](#method.set_num_threads)**
    #[deprecated(since = "1.3.0", note = "use ThreadPool::set_num_threads")]
    pub fn set_threads(&mut self, num_threads: usize) {
        self.set_num_threads(num_threads)
    }

    /// Sets the number of worker-threads to use as `num_threads`.  Can be used to change the threadpool size during runtime. Will not abort already running or waiting threads.
    pub fn set_num_threads(&mut self, num_threads: usize) {
        assert!(num_threads >= 1);
        let prev_num_threads = self.shared_data.max_thread_count.swap(
            num_threads,
            Ordering::Release,
        );
        if let Some(num_spawn) = num_threads.checked_sub(prev_num_threads) {
            // Spawn new threads
            for _ in 0..num_spawn {
                spawn_in_pool(self.shared_data.clone());
            }
        }
    }

    /// Block the current thread until all jobs in the pool have been executed.
    pub fn join(&self) {
        // fast path requires no mutex
        if self.shared_data.has_work() == false {
            return ();
        }

        let generation = self.shared_data.join_generation.load(Ordering::SeqCst);
        let mut lock = self.shared_data.empty_trigger.lock().unwrap();

        while generation == self.shared_data.join_generation.load(Ordering::Relaxed) &&
                self.shared_data.has_work() {
            lock = self.shared_data.empty_condvar.wait(lock).unwrap();
        }

        // increase generation if we are the first thread to come out of the loop
        self.shared_data.join_generation.compare_and_swap(generation, generation.wrapping_add(1), Ordering::SeqCst);
    }
}
```
创建线程，实现线程增减，如果运行中工作线程panic，则新建工作线程补充到线程池中。如果是减少当前的工作线程数量，则要等到工作线程运行到自动结束。不会强制终结目前正在运行的工作线程。
```rust
fn spawn_in_pool(shared_data: Arc<ThreadPoolSharedData>) {
    let mut builder = thread::Builder::new();
    if let Some(ref name) = shared_data.name {
        builder = builder.name(name.clone());
    }
    if let Some(ref stack_size) = shared_data.stack_size {
        builder = builder.stack_size(stack_size.to_owned());
    }
    builder
        .spawn(move || {
            // Will spawn a new thread on panic unless it is cancelled.
            let sentinel = Sentinel::new(&shared_data);

            loop {
                // Shutdown this thread if the pool has become smaller
                let thread_counter_val = shared_data.active_count.load(Ordering::Acquire);
                let max_thread_count_val = shared_data.max_thread_count.load(Ordering::Relaxed);
                if thread_counter_val >= max_thread_count_val {
                    break;
                }
                let message = {
                    // Only lock jobs for the time it takes to get a job, not run it.
                    let lock = shared_data.job_receiver.lock().expect(
                        "Worker thread unable to lock job_receiver",
                    );
                    lock.recv()
                };

                let job = match message {
                    Ok(job) => job,
                    // The ThreadPool was dropped.
                    Err(..) => break,
                };
                // Do not allow IR around the job execution
                shared_data.active_count.fetch_add(1, Ordering::SeqCst);
                shared_data.queued_count.fetch_sub(1, Ordering::SeqCst);

                job.call_box();

                shared_data.active_count.fetch_sub(1, Ordering::SeqCst);
                shared_data.no_work_notify_all();
            }

            sentinel.cancel();
        })
        .unwrap();
}

```

#### 四、其他线程池的实现
线程池有很多实现形式，用Rust语言实现的一个较为复杂的线程池可以参考[tokio_threadpool](https://crates.io/crates/tokio-threadpool)，是一个基于work-stealing算法的线程池。