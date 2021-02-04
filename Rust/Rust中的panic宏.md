`panic!`会立即终止程序，同时rust中的`Option`和`Result`出现`None`和`Err`时都会触发`panic`。
>如果主线程panic，则整个程序都会终止，如果不是主线程panic则只会终止子线程，其他线程不会异常终止.

panic宏源码如下：
```rust
#[macro_export]
#[stable(feature = "rust1", since = "1.0.0")]
#[allow_internal_unstable]
macro_rules! panic {
    () => ({
        panic!("explicit panic")
    });
    ($msg:expr) => ({
        $crate::rt::begin_panic($msg, &(file!(), line!(), __rust_unstable_column!()))
    });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::rt::begin_panic_fmt(&format_args!($fmt, $($arg)+),
                                    &(file!(), line!(), __rust_unstable_column!()))
    });
}

```


有时我们希望将panic信息输出到日志中，标准库中的std::panic可以满足需求。可以设置一个类似于windows中的钩子函数，在panic发出后，在panic运行时之前，触发钩子函数去处理这个panic信息。panic信息被保存在PanicInfo结构体中。
```rust
///Registers a custom panic hook, replacing any that was previously registered.
pub fn set_hook(hook: Box<Fn(&PanicInfo) + Sync + Send + 'static>)
```
>The panic hook is invoked when a thread panics, but before the panic runtime is invoked. As such, the hook will run with both the aborting and unwinding runtimes. The default hook prints a message to standard error and generates a backtrace if requested, but this behavior can be customized with the set_hook and take_hook functions.

```rust
#[macro_use]
extern crate log;
extern crate simple_logger;
use std::thread;
use std::boxed::Box;
use std::panic;

fn main() {
    simple_logger::init().unwrap();
    panic::set_hook(Box::new(|panic_info|{
        error!("panic info: {:?},panic occurred in {:?}",panic_info.payload().downcast_ref::<&str>(),panic_info.location());
    }));

    thread::spawn(||{
        panic!("child thread panic test!");
    });

    loop{
    }
}
```

[downcast_ref的源码](https://doc.rust-lang.org/src/core/any.rs.html#88-111)
```rust
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe {
                Some(&*(self as *const Any as *const T))
            }
        } else {
            None
        }
    }
```
但上面的代码如果是`Result`或者`Option`的`unwrap`触发的`panic`，则输出信息中`payload`项会是`None`，原因可以看上面`downcast_ref`的源码，进入`downcast_ref`后会有个判断条件如果不是类型`T`则就会返回`None`。如果我们想处理`unwrap`触发的`panic`，可以使用下面的示例代码，但是是有条件的，会用到`unsafe`代码段。
```rust
#[macro_use]
extern crate log;
extern crate simple_logger;
use std::thread;
use std::boxed::Box;
use std::panic;
use std::any::Any;

#[derive(Debug)]
struct PanicErr{
    info:String,
    value:i32,
}

fn main() {
    simple_logger::init().unwrap();
    panic::set_hook(Box::new(|panic_info|{
        unsafe {
            let s=&*(panic_info.payload() as *const Any as *const &str);
            error!("panic info: {},occurred in {:?}",s,panic_info.location());
        }
    }));

    thread::spawn(||{
        let e:Result<(),bool>=Err(false);
        e.unwrap();
    });

    let a:Result<(),PanicErr>=Err(PanicErr{info:"error info".to_string(),value:10});
    a.unwrap();

    loop{
    }
}

```

>参考文档：[std::panic](https://doc.rust-lang.org/std/macro.panic.html)