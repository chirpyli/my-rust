#### RefCell
Rust在编译阶段会进行严格的借用规则检查，规则如下：
>- 在任意给定时间，要么只能有一个可变引用，要么只能有多个不可变引用。
>- 引用必须总是有效。

即在编译阶段，当有一个不可变值时，不能可变的借用它。如下代码所示：
```rust
fn main() {
    let x = 5;
    let y = &mut x;
}
```
会产生编译错误：
```rust
error[E0596]: cannot borrow immutable local variable `x` as mutable
  --> src/main.rs:32:18
   |
31 |     let x = 5;
   |         - consider changing this to `mut x`
32 |     let y = &mut x;
   |                  ^ cannot borrow mutably
```
但是在实际的编程场景中可能会需要在有不可变引用时改变数据的情况，这时可以考虑Rust中的内部可变性。其借用规则检查由编译期推迟到运行期。对应的，在编译期借用规则检查不通过，则会产生编译错误；而运行期借用规则检查不通过，则会```panic```，且有运行期的代价。

所以实际代码中使用```RefCell<T>```的情况是当你确定你的代码遵循借用规则，而编译器不能理解和确定的时候。代码仍然要符合借用规则，只不过规则检查放到了运行期。

#### RefCell代码实例1：
```rust
use std::cell::RefCell;

fn main() {
    let x = RefCell::new(5u8);
    assert_eq!(5, *x.borrow());
    {
        let mut y = x.borrow_mut();
        *y = 10;
        assert_eq!(10, *x.borrow());
        let z = x.borrow();     //编译时会通过，但运行时panic!
    }
}
```
运行结果：
```rust
thread 'main' panicked at 'already mutably borrowed: BorrowError', libcore/result.rs:983
:5
note: Run with `RUST_BACKTRACE=1` for a backtrace.
```
可以看到在运行时进行了借用检查，并且panic!

#### RefCell代码实例2：
```rust
#[derive(Debug, Default)]
struct Data {
    a: u8,
    b: RefCell<u8>,
}

impl Data {
    // 编译通过
    pub fn value_b(&self) -> u8 {
        let mut cache = self.b.borrow_mut();
        if *cache != 0 {
            return *cache;
        }
        *cache = 100;
        *cache
    }

    //编译错误：cannot mutably borrow field of immutable binding
    pub fn value_a(&self) -> u8 {
        if self.a != 0 {
            return self.a;
        }

        self.a = 100;
        self.a
    }
}

fn main() {
    let value = Data::default();
    println!("{:?}", value);
    value.value_b();
    println!("{:?}", value);
}

```
把```value_a```注释掉运行结果如下：
```rust
Data { a: 0, b: RefCell { value: 0 } }
Data { a: 0, b: RefCell { value: 100 } }
```
很多时候我们只能获取一个不可变引用，然而又需要改变所引用数据，这时用```RefCell<T>```是解决办法之一。


#### 内部可变性

内部可变性（Interior mutability）是Rust中的一个设计模式，它允许你即使在有不可变引用时改变数据，这通常是借用规则所不允许的。为此，该模式在数据结构中使用unsafe代码来模糊Rust通常的可变性和借用规则。当可以确保代码在**运行时**会遵守借用规则，即使编译器不能保证的情况，可以选择使用那些运用内部可变性模式的类型。所涉及的 unsafe 代码将被封装进安全的 API 中，而外部类型仍然是不可变的。
