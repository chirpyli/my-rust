# my-rust
personal rust study notes.

### 异步
- [Rust异步之Future](./async/Rust异步之Future.md)
- [Rust异步之自己构造block_on](../Rust异步之自己构造block_on.md)
- [Rust异步之tokio](./async/Rust异步之tokio.md)

### 其他
- [Rust学习资料汇总](./Rust/Rust学习资料汇总.md)
- [RefCell和内部可变性](./Rust/RefCell和内部可变性.md)
- [Rust关联类型与默认类型参数](./Rust/Rust关联类型与默认类型参数.md)
- [Rust写时复制](./Rust/Rust写时复制.md)
- [Rust完全限定语法与消歧义：调用相同名称的方法](./Rust/Rust完全限定语法与消歧义：调用相同名称的方法.md)
- [Rust更换Crates源](./Rust/Rust更换Crates源)
- [Rust生命周期bound用于泛型的引用](./Rust/Rust生命周期bound用于泛型的引用.md)
- [对Rust所有权、借用及生命周期的理解](./Rust/对Rust所有权、借用及生命周期的理解.md)
- [Rust实现的常用密码学库](./Rust/Rust实现的常用密码学库)       
- [Rust中使用Protocol Buffers](./Rust/Rust中使用ProtocolBuffers.md)   
- [Rust中的panic宏](./Rust/Rust中的panic宏.md)      
- [线程池的简单实现（Rust）](./Rust/线程池的简单实现（Rust）.md)        
- [记一次排查内存泄漏的过程](./Rust/记一次排查内存泄漏的过程.md)  
- [Rust轻量级I/O库mio](./Rust/Rust轻量级IO库mio.md)     
- [Rust双重循环break的问题](./Rust双重循环break的问题.md)   
- [log4rs日志库简析](./Rust/log4rs日志库简析.md)
- [Rust关于ParticalEq与Eq](./Rust/cmp/particaleq/README.md)
- [Rust中的Arc与Rc](./Rust/Rust中的Arc与Rc.md)
- [实现一个基于栈的虚拟机](./Rust/vm/%E8%99%9A%E6%8B%9F%E6%9C%BA.md)

### 个人编写的Rust程序

- [lru-simple](https://github.com/chirpyli/data-structure/tree/master/lru/lru-simple)   LRU算法的简单实现
- [craft](./craft/)
- [pgmq-demo](./pgmq-demo/)     Rust中使用PostgreSQL消息队列拓展PGMQ的使用示例
- [amqp-demo](./amqp-demo/)    Rust中使用RabbitMQ的使用示例，这里采用的是[lapin](https://docs.rs/lapin/latest/lapin/index.html)库