列举一些学习Rust的好资料，方便平常学习与查阅。大部分文档在[官网](https://www.rust-lang.org/learn)`Grow with Rust`一节都有列出，另一部分是平常学习时涉及到的文档资料。


---

### [The Rust Programming Language](https://doc.rust-lang.org/book/title-page.html)
这本书当然是要第一本阅读的了，入门首选。

>Rust对单元测试的支持是非常友好的，可参考[Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)这一章。

### [Rust 程序设计语言 简体中文版](https://kaisery.github.io/trpl-zh-cn/title-page.html)

### [Rust by Example](https://doc.rust-lang.org/stable/rust-by-example/)
通过代码示例学习Rust。

### [The Rust Reference](https://doc.rust-lang.org/nightly/reference/introduction.html#introduction)

### [The Cargo Book](https://doc.rust-lang.org/cargo/index.html)
Rust包管理工具，提供了编译、安装等功能。`cargo build`编译真的很方便，就是编译大项目时时间稍微有点长。

### [The Edition Guide](https://doc.rust-lang.org/edition-guide/introduction.html)
这一本主要是Rust版本说明文档，相对2015版，2018版本的变化。

当前Rust版本已经到了2018，与之前的2015在很多地方都有不同，不过都能平滑过渡，还给提供了版本修复`cargo fix`工具。Rust还是很赞的，可以在文档[The Edition Guide](https://doc.rust-lang.org/edition-guide/introduction.html)，学习2015到2018的变化。比较突出的变化是异步支持`async`、`await`关键字，很多涉及到异步的库都会要求Rust版本为2018版本，还有在使用Trait对象时需要添加`dyn`等等变化。如果你原先是老版本代码，到新版本编译时编译器都会给予提示，方便你从老版本到新版本。总体来讲变化并不大，更多的都是细节性的。

有变化的关键字含义：
- `async` - return a Future instead of blocking the current thread
- `await` - suspend execution until the result of a Future is ready
- `dyn` - dynamic dispatch to a trait object

### [The Rustonomicon(The Dark Arts of Unsafe Rust)](https://doc.rust-lang.org/stable/nomicon/)
这本是Rust进阶读物，比较有难度，比较底层一些。中文翻译：[《Rust 高级编程》](https://learnku.com/docs/nomicon/2018)



### [The Rust Standard Library](https://doc.rust-lang.org/std/index.html)
Rust标准库，常用。

### [crates.io](https://crates.io/)
在这里可以查找丰富Rust第三方库。

### [Keywords](https://doc.rust-lang.org/book/appendix-01-keywords.html)
Rust关键字

### [Futures Explained in 200 Lines of Rust](https://cfsamson.github.io/books-futures-explained/introduction.html)
学习异步的好文档，中文翻译[200行代码讲透RUST FUTURES的问题](https://stevenbai.top/rust/futures_explained_in_200_lines_of_rust2/).

### [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/)
Rust异步编程


### Rust常用的库

- [grpc-rs](https://github.com/tikv/grpc-rs) : tikv团队的grpc实现，封装的C实现的grpc。
- [tonic](https://github.com/hyperium/tonic): Rust原生实现的grpc。
- [mio](https://github.com/tokio-rs/mio) : I/O库，简单封装了epoll（对Linux操作系统来讲）。
- [tokio](https://github.com/tokio-rs/tokio):实现了异步、非阻塞I/O、事件驱动，底层是mio。

>mio从之前的v0.6现已到v0.7，而tokio也从v0.1到v0.2，尤其是tokio，之前v0.1变动比较大，所以实际项目中没有采用这个库，而是使用了原始的mio，真的很原始，写代码调试代码很累，代码比较离散，现在的v0.2，后面再有需要可以考虑使用了v0.2版本了。

>gRPC的Rust实现，目前看有大概四五个版本实现，上面的两个版本实现个人认为是相对比较好的。

### Rust开源项目
- [libra](https://github.com/libra/libra) ：Facebook区块链项目
- [tikv](https://github.com/tikv/tikv)：国内pingcap分布式数据库项目
- [parity](https://github.com/paritytech/parity): 以太坊Rust实现
- [parity-bitcoin](https://github.com/paritytech/parity-bitcoin)： 比特币的Rust实现


