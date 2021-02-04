在单循环中使用`break`跳出循环，但如果遇到双重循环或者更多重循环时怎么办呢？与其他语言类似，Rust使用标记标识跳出指定循环。如下所示：
```rust
fn main() {
    let a = vec![1;5];
    let b = vec![2;6];
    'outer: for i in a {
        println!("{}", i);
        'inner: for j in b.iter() {
            print!("{}", j);
            break 'outer;   // 跳出外层循环，如果不加标记，默认跳出最内层循环
        }
    }
}
```

>可参考：[Nesting and labels](https://doc.rust-lang.org/rust-by-example/flow_control/loop/nested.html)