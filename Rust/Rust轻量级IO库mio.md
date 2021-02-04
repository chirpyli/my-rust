mio是rust实现的一个轻量级的I/O库。其实现基本上就是对不同操作系统底层相关API的封装，抽象出统一的接口供上层使用。Linux下为[epoll](http://man7.org/linux/man-pages/man7/epoll.7.html)，Windows下为IOCP，OS X下为[kqueue](https://www.freebsd.org/cgi/man.cgi?query=kqueue&sektion=2)。



### 一、关于mio
#### 1、重要特性
- 非阻塞TCP，UDP
- I/O事件通知epoll,kqeue,IOCP实现
- 运行时零分配
- 平台可扩展

#### 2、基础用法
其使用方法与Linux中epoll差不多，mio底层封装了epoll，使用步骤思路：
1. 创建Poll
2. 注册事件
3. 事件循环等待与处理事件

mio提供可跨平台的sytem selector访问，不同平台如下表，都可调用相同的API。不同平台使用的API开销不尽相同。由于mio是基于readiness(就绪状态)的API，与Linux epoll相似，可以看到很多API在Linux上都可以一对一映射。相比之下，Windows IOCP是基于完成（completion-based）而非基于就绪的API，所以两者间会有较多桥接。 同时mio提供自身版本的TcpListener、TcpStream、UdpSocket，这些API封装了底层平台相关API，并设为非阻塞且实现Evented trait。


|OS	|Selector|
|--  | -- |
|Linux|	epoll
|OS X, iOS	|kqueue
|Windows	|IOCP
|FreeBSD	|kqueue
|Android	|epoll

> mio实现的是一个单线程事件循环，并没有实现线程池及多线程事件循环，如果需要线程池及多线程事件循环等需要自己实现。

### 二、源码分析
先给出mio的源码目录结构，只列出了关键的部分，如下所示：
```rust
mio代码目录结构
mio
|---->test
|---->src
|-------->deprecated			//事件循环代码
|-------------->event_loop.rs		//EventLoop的实现，内部封装了Poll		【1】
|-------------->handler.rs			//供上层实现的接口
|-------->net
|------------>mod.rs
|------------>tcp.rs
|------------>udp.rs
|-------->sys						//不同系统下的实现
|------------>mod.rs			
|------------>fuchsia
|------------>unix				//Linux下封装的epoll
|------------------>mod.rs
|------------------>epoll.rs						【3】
|------------------>awakener.rs
|------------>windows			//windows下封装的iocp
|-------->lib.rs	
|-------->poll.rs			//定义Poll			【2】
|-------->channel.rs		【4】
|-------->event_imp.rs
|-------->timer.rs		【5】
|-------->......
```
对涉及不同操作系统的部分代码，以Linux操作系统为例。在Linux操作系统中，mio封装了epoll。后面会给出相应的代码。

#### 【1】Eventloop代码分析
结合前面的代码示例给出相应的关键代码如下：
`EventLoop`事件循环定义，可以看到里面封装了`Poll`，以Linux系统举例，`Poll`又封装了`epoll`。在使用`Poll`或Linux中`epoll`时，最重要的代码是`epoll_wait()`等待事件`Event`并针对每个`Event`进行不同的处理。这里`EventLoop`将`epoll_create()`、`epoll_wait()`、`epoll_ctl()`进行进一步的封装，将对`Event`的处理抽象成`Handler`，供上层实现具体的逻辑处理。
```rust
// Single threaded IO event loop.		//这里是单线程事件循环，更多的时候我们需要加线程池，以此为基础，再进行一次封装，供上层使用
pub struct EventLoop<H: Handler> {
    run: bool,
    poll: Poll,		
    events: Events,		//对应epoll中的epoll_event
    timer: Timer<H::Timeout>,
    notify_tx: channel::SyncSender<H::Message>,
    notify_rx: channel::Receiver<H::Message>,
    config: Config,
}
```
抽象出接口供上层应用实现不同事件的逻辑处理。这里有点类似于回调函数，上层用户需要在此实现业务逻辑代码，实际运行时需要将函数指针传递给底层事件循环，底层事件循环运行时会调用用户传递过来的函数。在Rust中，可能描述的不是很精准，不过可以这样理解。
```rust
pub trait Handler: Sized {
    type Timeout;
    type Message;

    /// Invoked when the socket represented by `token` is ready to be operated
    /// on. `events` indicates the specific operations that are
    /// ready to be performed.
    /// This function will only be invoked a single time per socket per event
    /// loop tick.
    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: Ready) {
    }		//【1】

    /// Invoked when a message has been received via the event loop's channel.
    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
    }		//【2】

    /// Invoked when a timeout has completed.
    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Self::Timeout) {
    }		//【3】

    /// Invoked when `EventLoop` has been interrupted by a signal interrupt.
    fn interrupted(&mut self, event_loop: &mut EventLoop<Self>) {
    }		//【4】

    /// Invoked at the end of an event loop tick.
    fn tick(&mut self, event_loop: &mut EventLoop<Self>) {
    }		//【5】
}
```
这里把`Poll`进行了封装，主要实现了`Eventloop::new()`---->`Poll::new()`---->`epoll_create()`，`Eventloop::run()`--->`Selecter::select()`---->`epoll_wait()`，还有`register()`、`reregister()`、`deregister()`等等......
```rust
impl<H: Handler> EventLoop<H> {
    /// Constructs a new `EventLoop` using the default configuration values.
    /// The `EventLoop` will not be running.
    pub fn new() -> io::Result<EventLoop<H>> {
        EventLoop::configured(Config::default())
    }

    fn configured(config: Config) -> io::Result<EventLoop<H>> {
        // Create the IO poller
        let poll = Poll::new()?;		//Linux内部调用epoll_create()

        let timer = timer::Builder::default()
            .tick_duration(config.timer_tick)
            .num_slots(config.timer_wheel_size)
            .capacity(config.timer_capacity)
            .build();

        // Create cross thread notification queue
        let (tx, rx) = channel::sync_channel(config.notify_capacity);  //这里创建的是同步管道,可配置同步管道内部的buffer queue bound size.

        // Register the notification wakeup FD with the IO poller
        poll.register(&rx, NOTIFY, Ready::readable(), PollOpt::edge() | PollOpt::oneshot())?;	//NOTIFY和TIMER由mio实现
        poll.register(&timer, TIMER, Ready::readable(), PollOpt::edge())?;

        Ok(EventLoop {
            run: true,
            poll: poll,
            timer: timer,
            notify_tx: tx,
            notify_rx: rx,
            config: config,
            events: Events::with_capacity(1024),
        })
    }

    /// Keep spinning the event loop indefinitely, and notify the handler whenever
    /// any of the registered handles are ready.
    pub fn run(&mut self, handler: &mut H) -> io::Result<()> {
        self.run = true;

        while self.run {
            // Execute ticks as long as the event loop is running
            self.run_once(handler, None)?;	//Linux下调用epoll_wait()
        }

        Ok(())
    }

    pub fn run_once(&mut self, handler: &mut H, timeout: Option<Duration>) -> io::Result<()> {
        trace!("event loop tick");

        // Check the registered IO handles for any new events. Each poll
        // is for one second, so a shutdown request can last as long as
        // one second before it takes effect.
        let events = match self.io_poll(timeout) {
            Ok(e) => e,
            Err(err) => {
                if err.kind() == io::ErrorKind::Interrupted {
                    handler.interrupted(self);		//调用Handler::interrupted() 【4】
                    0
                } else {
                    return Err(err);
                }
            }
        };

        self.io_process(handler, events);	//处理就绪的事件，handler为如何处理各种事件的实例
        handler.tick(self);	//一轮事件处理后，最后调用Handler::tick()	调用【5】
        Ok(())
    }

    #[inline]
    fn io_poll(&mut self, timeout: Option<Duration>) -> io::Result<usize> {
        self.poll.poll(&mut self.events, timeout)
    }

    // Process IO events that have been previously polled
    fn io_process(&mut self, handler: &mut H, cnt: usize) {
        let mut i = 0;

        trace!("io_process(..); cnt={}; len={}", cnt, self.events.len());

        // Iterate over the notifications. Each event provides the token
        // it was registered with (which usually represents, at least, the
        // handle that the event is about) as well as information about
        // what kind of event occurred (readable, writable, signal, etc.)
        while i < cnt {		//遍历所有就绪的事件，进行处理
            let evt = self.events.get(i).unwrap();

            trace!("event={:?}; idx={:?}", evt, i);

			// mio在epoll之上，增加了NOTIFY和TIMER
            match evt.token() {
                NOTIFY => self.notify(handler),			//channel处理 ，这个epoll中是没有的，mio实现
                TIMER => self.timer_process(handler),	//Timer处理， 这个epoll中也是没有的，mio实现
                _ => self.io_event(handler, evt)		//IO事件的处理， 这个epoll有
            }

            i += 1;
        }
    }

    fn io_event(&mut self, handler: &mut H, evt: Event) {
        handler.ready(self, evt.token(), evt.readiness());	//调用Handler::ready() 【1】
    }

    fn notify(&mut self, handler: &mut H) {
        for _ in 0..self.config.messages_per_tick {
            match self.notify_rx.try_recv() {	//从channel中接收数据，内部实现是std::sync::mpsc::sync_channel()
                Ok(msg) => handler.notify(self, msg),	//调用Handler::notify()	【2】
                _ => break,
            }
        }

        // Re-register
        let _ = self.poll.reregister(&self.notify_rx, NOTIFY, Ready::readable(), PollOpt::edge() | PollOpt::oneshot());	//PollOpt::oneshot(),必须重新reregister.
    }

    fn timer_process(&mut self, handler: &mut H) {
        while let Some(t) = self.timer.poll() {
            handler.timeout(self, t);	//调用Handler::timeout() 【3】
        }
    }

    /// Registers an IO handle with the event loop.
    pub fn register<E: ?Sized>(&mut self, io: &E, token: Token, interest: Ready, opt: PollOpt) -> io::Result<()>
        where E: Evented
    {
        self.poll.register(io, token, interest, opt)
    }

    /// Re-Registers an IO handle with the event loop.
    pub fn reregister<E: ?Sized>(&mut self, io: &E, token: Token, interest: Ready, opt: PollOpt) -> io::Result<()>
        where E: Evented
    {
        self.poll.reregister(io, token, interest, opt)
    }

    /// Deregisters an IO handle with the event loop.
    pub fn deregister<E: ?Sized>(&mut self, io: &E) -> io::Result<()> where E: Evented {
        self.poll.deregister(io)
    }

    /// Returns a sender that allows sending messages to the event loop in a
    /// thread-safe way, waking up the event loop if needed.
    pub fn channel(&self) -> Sender<H::Message> {
        Sender::new(self.notify_tx.clone())
    }

    /// Schedules a timeout after the requested time interval. When the
    /// duration has been reached,
    pub fn timeout(&mut self, token: H::Timeout, delay: Duration) -> timer::Result<Timeout> {
        self.timer.set_timeout(delay, token)
    }

    /// If the supplied timeout has not been triggered, cancel it such that it
    /// will not be triggered in the future.
    pub fn clear_timeout(&mut self, timeout: &Timeout) -> bool {
        self.timer.cancel_timeout(&timeout).is_some()
    }

    /// Tells the event loop to exit after it is done handling all events in the current iteration.
    pub fn shutdown(&mut self) {
        self.run = false;
    }

    /// Indicates whether the event loop is currently running. If it's not it has either
    /// stopped or is scheduled to stop on the next tick.
    pub fn is_running(&self) -> bool {
        self.run
    }
}
```
#### 【2】Poll代码分析
`Poll`屏蔽了不同系统的实现，给出了统一的抽象。`Poll`的实现代码这里只能列出较为重要的部分代码，有一部分代码省略掉了，详细代码可查看[mio/src/poll.rs](https://github.com/tokio-rs/mio/blob/master/src/poll.rs)：
```rust
pub struct Poll {
    // Platform specific IO selector
    selector: sys::Selector,	

    // Custom readiness queue
    // The second readiness queue is implemented in user space by `ReadinessQueue`. It provides a way to implement purely user space `Evented` types.
    readiness_queue: ReadinessQueue,	//区别于系统就绪队列（sys::Selector），这是上层自己实现的就绪队列

    // Use an atomic to first check if a full lock will be required. This is a
    // fast-path check for single threaded cases avoiding the extra syscall
    lock_state: AtomicUsize,

    // Sequences concurrent calls to `Poll::poll`
    lock: Mutex<()>,

    // Wakeup the next waiter
    condvar: Condvar,
}

impl Poll {
    /// Return a new `Poll` handle.
    pub fn new() -> io::Result<Poll> {
        is_send::<Poll>();
        is_sync::<Poll>();

        let poll = Poll {
            selector: sys::Selector::new()?,
            readiness_queue: ReadinessQueue::new()?,
            lock_state: AtomicUsize::new(0),
            lock: Mutex::new(()),
            condvar: Condvar::new(),
        };

        // Register the notification wakeup FD with the IO poller
        poll.readiness_queue.inner.awakener.register(&poll, AWAKEN, Ready::readable(), PollOpt::edge())?;

        Ok(poll)
    }
	
    /// Wait for readiness events
    ///
    /// Blocks the current thread and waits for readiness events for any of the
    /// `Evented` handles that have been registered with this `Poll` instance.
    /// The function will block until either at least one readiness event has
    /// been received or `timeout` has elapsed. A `timeout` of `None` means that
    /// `poll` will block until a readiness event has been received.
    pub fn poll(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<usize> {
        self.poll1(events, timeout, false)		//Poll::poll()非常最重要的一个方法， poll()-->poll1()-->poll2()
    }
    
    fn poll1(&self, events: &mut Events, mut timeout: Option<Duration>, interruptible: bool) -> io::Result<usize> {
        let zero = Some(Duration::from_millis(0));

        let mut curr = self.lock_state.compare_and_swap(0, 1, SeqCst);

        if 0 != curr { ... }	//{ ... }代表中间有很多代码被省略掉了.

        let ret = self.poll2(events, timeout, interruptible);

        // Release the lock
        if 1 != self.lock_state.fetch_and(!1, Release) { ... }	//{ ... }代表中间有很多代码被省略掉了.

        ret
    }

    #[inline]
    fn poll2(&self, events: &mut Events, mut timeout: Option<Duration>, interruptible: bool) -> io::Result<usize> {
        // Compute the timeout value passed to the system selector. If the
        // readiness queue has pending nodes, we still want to poll the system
        // selector for new events, but we don't want to block the thread to
        // wait for new events.
        if timeout == Some(Duration::from_millis(0)) {
            // If blocking is not requested, then there is no need to prepare
            // the queue for sleep
            //
            // The sleep_marker should be removed by readiness_queue.poll().
        } else if self.readiness_queue.prepare_for_sleep() {
            // The readiness queue is empty. The call to `prepare_for_sleep`
            // inserts `sleep_marker` into the queue. This signals to any
            // threads setting readiness that the `Poll::poll` is going to
            // sleep, so the awakener should be used.
        } else {
            // The readiness queue is not empty, so do not block the thread.
            timeout = Some(Duration::from_millis(0));
        }

		//poll系统就绪队列
        loop {
            let now = Instant::now();
            // First get selector events
            let res = self.selector.select(&mut events.inner, AWAKEN, timeout);	//Linux下调用epoll_wait(),就绪事件放入events中
            match res {
                Ok(true) => {
                    // Some awakeners require reading from a FD.
                    self.readiness_queue.inner.awakener.cleanup();
                    break;
                }
                Ok(false) => break,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted && !interruptible => {
                    // Interrupted by a signal; update timeout if necessary and retry
                    if let Some(to) = timeout {
                        let elapsed = now.elapsed();
                        if elapsed >= to {
                            break;
                        } else {
                            timeout = Some(to - elapsed);
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }

        // Poll custom event queue
        self.readiness_queue.poll(&mut events.inner);	//Poll用户就绪队列

        // Return number of polled events
        Ok(events.inner.len())
    }

    /// Register an `Evented` handle with the `Poll` instance.
    pub fn register<E: ?Sized>(&self, handle: &E, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()>
        where E: Evented {
        validate_args(token)?;

        // Register interests for this socket
        handle.register(self, token, interest, opts)?;

        Ok(())
    }

    /// Re-register an `Evented` handle with the `Poll` instance.
    pub fn reregister<E: ?Sized>(&self, handle: &E, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()>
        where E: Evented {
        validate_args(token)?;

        // Register interests for this socket
        handle.reregister(self, token, interest, opts)?;

        Ok(())
    }

    /// Deregister an `Evented` handle with the `Poll` instance.
    pub fn deregister<E: ?Sized>(&self, handle: &E) -> io::Result<()>
        where E: Evented {
        // Deregister interests for this socket
        handle.deregister(self)?;

        Ok(())
    }
}
```

#### 【3】Selector代码分析
下面这段代码出自[mio/src/sys/unix/epoll.rs](https://github.com/tokio-rs/mio/blob/v0.6.16/src/sys/unix/epoll.rs)是对底层Linux系统epoll的封装抽象，可以看到`Selector::new()`内部实际上调用了`epoll_create()`，`Selector::select()`内部实际上调用了`epoll_wait()`，`register()`、`reregister()`、`deregister()`实内部实际上调用了`epoll_ctl()`。如果你非常熟悉`epoll`，就会感觉下面的代码很熟悉，详细代码如下：
```rust
pub struct Selector {
    id: usize,
    epfd: RawFd,
}

impl Selector {
    pub fn new() -> io::Result<Selector> {
        let epfd = unsafe {
            // Emulate `epoll_create` by using `epoll_create1` if it's available
            // and otherwise falling back to `epoll_create` followed by a call to
            // set the CLOEXEC flag.
            dlsym!(fn epoll_create1(c_int) -> c_int);

            match epoll_create1.get() {
                Some(epoll_create1_fn) => {
                    cvt(epoll_create1_fn(libc::EPOLL_CLOEXEC))?
                }
                None => {
                    let fd = cvt(libc::epoll_create(1024))?;
                    drop(set_cloexec(fd));
                    fd
                }
            }
        };

        // offset by 1 to avoid choosing 0 as the id of a selector
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed) + 1;

        Ok(Selector {
            id: id,
            epfd: epfd,
        })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    /// Wait for events from the OS
    pub fn select(&self, evts: &mut Events, awakener: Token, timeout: Option<Duration>) -> io::Result<bool> {
        let timeout_ms = timeout
            .map(|to| cmp::min(millis(to), i32::MAX as u64) as i32)
            .unwrap_or(-1);

        // Wait for epoll events for at most timeout_ms milliseconds
        evts.clear();
        unsafe {
            let cnt = cvt(libc::epoll_wait(self.epfd,
                                           evts.events.as_mut_ptr(),
                                           evts.events.capacity() as i32,
                                           timeout_ms))?;
            let cnt = cnt as usize;
            evts.events.set_len(cnt);

            for i in 0..cnt {
                if evts.events[i].u64 as usize == awakener.into() {
                    evts.events.remove(i);
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Register event interests for the given IO handle with the OS
    pub fn register(&self, fd: RawFd, token: Token, interests: Ready, opts: PollOpt) -> io::Result<()> {
        let mut info = libc::epoll_event {
            events: ioevent_to_epoll(interests, opts),
            u64: usize::from(token) as u64
        };

        unsafe {
            cvt(libc::epoll_ctl(self.epfd, libc::EPOLL_CTL_ADD, fd, &mut info))?;
            Ok(())
        }
    }

    /// Register event interests for the given IO handle with the OS
    pub fn reregister(&self, fd: RawFd, token: Token, interests: Ready, opts: PollOpt) -> io::Result<()> {
        let mut info = libc::epoll_event {
            events: ioevent_to_epoll(interests, opts),
            u64: usize::from(token) as u64
        };

        unsafe {
            cvt(libc::epoll_ctl(self.epfd, libc::EPOLL_CTL_MOD, fd, &mut info))?;
            Ok(())
        }
    }

    /// Deregister event interests for the given IO handle with the OS
    pub fn deregister(&self, fd: RawFd) -> io::Result<()> {
        // The &info argument should be ignored by the system,
        // but linux < 2.6.9 required it to be not null.
        // For compatibility, we provide a dummy EpollEvent.
        let mut info = libc::epoll_event {
            events: 0,
            u64: 0,
        };

        unsafe {
            cvt(libc::epoll_ctl(self.epfd, libc::EPOLL_CTL_DEL, fd, &mut info))?;
            Ok(())
        }
    }
}
```

#### 【4】Notify channel代码分析
这个涉及的代码比较多，比较杂，也较为难以理解。
```rust
// `ReadinessQueue` is backed by a MPSC queue that supports reuse of linked
// list nodes. This significantly reduces the number of required allocations.
// Each `Registration` / `SetReadiness` pair allocates a single readiness node
// that is used for the lifetime of the registration.
//
// The readiness node also includes a single atomic variable, `state` that
// tracks most of the state associated with the registration. This includes the
// current readiness, interest, poll options, and internal state. When the node
// state is mutated, it is queued in the MPSC channel. A call to
// `ReadinessQueue::poll` will dequeue and process nodes. The node state can
// still be mutated while it is queued in the channel for processing.
// Intermediate state values do not matter as long as the final state is
// included in the call to `poll`. This is the eventually consistent nature of
// the readiness queue.
//
// The readiness node is ref counted using the `ref_count` field. On creation,
// the ref_count is initialized to 3: one `Registration` handle, one
// `SetReadiness` handle, and one for the readiness queue. Since the readiness queue
// doesn't *always* hold a handle to the node, we don't use the Arc type for
// managing ref counts (this is to avoid constantly incrementing and
// decrementing the ref count when pushing & popping from the queue). When the
// `Registration` handle is dropped, the `dropped` flag is set on the node, then
// the node is pushed into the registration queue. When Poll::poll pops the
// node, it sees the drop flag is set, and decrements it's ref count.
//
// The MPSC queue is a modified version of the intrusive MPSC node based queue
// described by 1024cores [1].
#[derive(Clone)]
struct ReadinessQueue {
    inner: Arc<ReadinessQueueInner>,
}

struct ReadinessQueueInner {
    // Used to wake up `Poll` when readiness is set in another thread.
    awakener: sys::Awakener,

    // Head of the MPSC queue used to signal readiness to `Poll::poll`.
    head_readiness: AtomicPtr<ReadinessNode>,

    // Tail of the readiness queue.
    //
    // Only accessed by Poll::poll. Coordination will be handled by the poll fn
    tail_readiness: UnsafeCell<*mut ReadinessNode>,

    // Fake readiness node used to punctuate the end of the readiness queue.
    // Before attempting to read from the queue, this node is inserted in order
    // to partition the queue between nodes that are "owned" by the dequeue end
    // and nodes that will be pushed on by producers.
    end_marker: Box<ReadinessNode>,

    // Similar to `end_marker`, but this node signals to producers that `Poll`
    // has gone to sleep and must be woken up.
    sleep_marker: Box<ReadinessNode>,

    // Similar to `end_marker`, but the node signals that the queue is closed.
    // This happens when `ReadyQueue` is dropped and signals to producers that
    // the nodes should no longer be pushed into the queue.
    closed_marker: Box<ReadinessNode>,
}
```

```rust
/// Node shared by a `Registration` / `SetReadiness` pair as well as the node
/// queued into the MPSC channel.
struct ReadinessNode {
    // Node state, see struct docs for `ReadinessState`
    //
    // This variable is the primary point of coordination between all the
    // various threads concurrently accessing the node.
    state: AtomicState,

    // The registration token cannot fit into the `state` variable, so it is
    // broken out here. In order to atomically update both the state and token
    // we have to jump through a few hoops.
    //
    // First, `state` includes `token_read_pos` and `token_write_pos`. These can
    // either be 0, 1, or 2 which represent a token slot. `token_write_pos` is
    // the token slot that contains the most up to date registration token.
    // `token_read_pos` is the token slot that `poll` is currently reading from.
    //
    // When a call to `update` includes a different token than the one currently
    // associated with the registration (token_write_pos), first an unused token
    // slot is found. The unused slot is the one not represented by
    // `token_read_pos` OR `token_write_pos`. The new token is written to this
    // slot, then `state` is updated with the new `token_write_pos` value. This
    // requires that there is only a *single* concurrent call to `update`.
    //
    // When `poll` reads a node state, it checks that `token_read_pos` matches
    // `token_write_pos`. If they do not match, then it atomically updates
    // `state` such that `token_read_pos` is set to `token_write_pos`. It will
    // then read the token at the newly updated `token_read_pos`.
    token_0: UnsafeCell<Token>,
    token_1: UnsafeCell<Token>,
    token_2: UnsafeCell<Token>,

    // Used when the node is queued in the readiness linked list. Accessing
    // this field requires winning the "queue" lock
    next_readiness: AtomicPtr<ReadinessNode>,

    // Ensures that there is only one concurrent call to `update`.
    //
    // Each call to `update` will attempt to swap `update_lock` from `false` to
    // `true`. If the CAS succeeds, the thread has obtained the update lock. If
    // the CAS fails, then the `update` call returns immediately and the update
    // is discarded.
    update_lock: AtomicBool,

    // Pointer to Arc<ReadinessQueueInner>
    readiness_queue: AtomicPtr<()>,

    // Tracks the number of `ReadyRef` pointers
    ref_count: AtomicUsize,
}
```

```rust
/// Handle to a user space `Poll` registration.
///
/// `Registration` allows implementing [`Evented`] for types that cannot work
/// with the [system selector]. A `Registration` is always paired with a
/// `SetReadiness`, which allows updating the registration's readiness state.
/// When [`set_readiness`] is called and the `Registration` is associated with a
/// [`Poll`] instance, a readiness event will be created and eventually returned
/// by [`poll`].
pub struct Registration {
    inner: RegistrationInner,
}
```

```rust
/// Updates the readiness state of the associated `Registration`.
#[derive(Clone)]
pub struct SetReadiness {
    inner: RegistrationInner,
}
```
未完，待续......

>参考文档：[Intrusive MPSC node-based queue](http://www.1024cores.net/home/lock-free-algorithms/queues/intrusive-mpsc-node-based-queue)

#### 【5】Timer定时器代码分析
```rust
pub struct Timer<T> {
    // Size of each tick in milliseconds
    tick_ms: u64,
    // Slab of timeout entries
    entries: Slab<Entry<T>>,
    // Timeout wheel. Each tick, the timer will look at the next slot for
    // timeouts that match the current tick.
    wheel: Vec<WheelEntry>,
    // Tick 0's time instant
    start: Instant,
    // The current tick
    tick: Tick,
    // The next entry to possibly timeout
    next: Token,
    // Masks the target tick to get the slot
    mask: u64,
    // Set on registration with Poll
    inner: LazyCell<Inner>,
}
```
未完，待续......
### 三、mio用法示例
下面的2个示例都很简单，其实直接看mio的[测试代码mio/test/](https://github.com/tokio-rs/mio/tree/master/test)就好了，不用看下面的2个示例。
#### 1、代码示例1
直接使用`Poll`示例如下：
```rust
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate mio;

use mio::*;
use mio::tcp::{TcpListener, TcpStream};
use std::io::{Read,Write};

fn main() {
    simple_logger::init().unwrap();

    // Setup some tokens to allow us to identify which event is for which socket.
    const SERVER: Token = Token(0);
    const CLIENT: Token = Token(1);

    let addr = "127.0.0.1:12345".parse().unwrap();

    // Setup the server socket
    let server = TcpListener::bind(&addr).unwrap();

    // Create a poll instance
    let poll = Poll::new().unwrap();

    // Start listening for incoming connections
    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    // Setup the client socket
    let sock = TcpStream::connect(&addr).unwrap();

    // Register the socket
    poll.register(&sock, CLIENT, Ready::readable(), PollOpt::edge()).unwrap();

    // Create storage for events
    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    // Accept and drop the socket immediately, this will close
                    // the socket and notify the client of the EOF.
                    let (stream,addr) = server.accept().unwrap();
                    info!("Listener accept {:?}",addr);
                },
                CLIENT => {
                    // The server just shuts down the socket, let's just exit
                    // from our event loop.
                    info!("client response.");
                    return;
                },
                _ => unreachable!(),
            }
        }
    }
}
```
通过上面的代码示例1，我们可以看到其用法与`epoll`非常相似。
#### 2、代码示例2
上面的代码编程时较为麻烦，下面使用事件循环`EventLoop`的方式，代码能看起来更清晰一些（相对的）：
```rust
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate mio;

use mio::*;
use mio::timer::{Timeout};
use mio::deprecated::{EventLoop, Handler, Sender, EventLoopBuilder};
use std::thread;
use std::time::Duration;

fn main() {
    simple_logger::init().unwrap();

    let mut event_loop=EventLoop::new().unwrap();
    let channel_sender=event_loop.channel();

    thread::spawn(move ||{
        channel_sender.send(IoMessage::Notify);
        thread::sleep_ms(5*1000);
        channel_sender.send(IoMessage::End);
    });

    let timeout = event_loop.timeout(Token(123), Duration::from_millis(3000)).unwrap();

    let mut handler=MioHandler::new();
    let _ = event_loop.run(&mut handler).unwrap();
}

pub enum IoMessage{
    Notify,
    End,
}

pub struct MioHandler{
}

impl MioHandler{
    pub fn new()->Self{
        MioHandler{}
    }
}

impl Handler for MioHandler {
    type Timeout = Token;
    type Message = IoMessage;

    /// Invoked when the socket represented by `token` is ready to be operated on.
    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: Ready) {
    }

    /// Invoked when a message has been received via the event loop's channel.
    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
        match msg {
            IoMessage::Notify=>info!("channel notify"),
            IoMessage::End=>{
                info!("shutdown eventloop.");
                event_loop.shutdown();
            }
        }
    }

    /// Invoked when a timeout has completed.
    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Self::Timeout) {
        match timeout{
            Token(123)=>info!("time out."),
            Token(_)=>{},
        }
    }

    /// Invoked when `EventLoop` has been interrupted by a signal interrupt.
    fn interrupted(&mut self, event_loop: &mut EventLoop<Self>) {
    }

    /// Invoked at the end of an event loop tick.
    fn tick(&mut self, event_loop: &mut EventLoop<Self>) {
    }
}
```
这个示例说明了超时及channel，围绕`EventLoop`编程，其实与上一个例子没有什么不同，只是`EventLoop`对`Poll`做了封装。


>参考文档：     
[【譯】Tokio 內部機制：從頭理解 Rust 非同步 I/O 框架](https://blog.techbridge.cc/2018/01/05/tokio-internal/)        
[使用mio开发web framework - base](https://www.jianshu.com/p/aad78343249a)       
[My Basic Understanding of mio and Asynchronous IO](https://hermanradtke.com/2015/07/12/my-basic-understanding-of-mio-and-async-io.html)        
[MIO for Rust](https://legacy.gitbook.com/book/wycats/mio-book/details)     
[mio-github](https://github.com/carllerche/mio)     