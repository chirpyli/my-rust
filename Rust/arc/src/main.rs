use std::sync::Arc;
use std::sync::Mutex;
use std::rc::Rc;

fn main() {
    println!("Arc or Rc!");

    let r0 = Rc::new(10);
    let r1 = r0.clone();
    println!("rc count: {}. r0 ptr: {:?}, r1 ptr: {:?}", Rc::strong_count(&r0), Rc::as_ptr(&r0), Rc::as_ptr(&r1));

    let ar0 = Arc::new(10);
    let ar1 = ar0.clone();
    let armut0 = Arc::new(Mutex::new(100));     // 要想写操作，必须加锁，如果不加锁的话，也可以实现写操作，但不是写的同一个地址，线程间的数据会不一致。
    let armut1 = armut0.clone();
    let handle = std::thread::spawn(move || {
        let mut lock = armut1.lock().unwrap();
        *lock = *ar1;
        println!("ar1 ptr: {:?}. mute change to {}", Arc::as_ptr(&ar1), *lock);
    });
    handle.join();
    println!("arc count: {}. arc0 ptr: {:?}. arcmut0 value: {}", Arc::strong_count(&ar0), Arc::as_ptr(&ar0), *armut0.lock().unwrap());
}
