写时复制（Copy on Write）技术是一种程序中的优化策略，多应用于读多写少的场景。主要思想是创建对象的时候不立即进行复制，而是先引用（借用）原有对象进行大量的读操作，只有进行到少量的写操作的时候，才进行复制操作，将原有对象复制后再写入。这样的好处是在读多写少的场景下，减少了复制操作，提高了性能。

Rust中对应这种思想的是智能指针`Cow<T>`，定义如下：
```rust
pub enum Cow<'a, B> 
where
    B: 'a + ToOwned + 'a + ?Sized, 
 {
    Borrowed(&'a B),    //用于包裹引用
    Owned(<B as ToOwned>::Owned),   //用于包裹所有者
}
```
可以看到是一个枚举体，包括两个可选值，一个是“借用”，一个是“所有”。具体含义是：以不可变的方式访问借用内容，在需要可变借用或所有权的时候再克隆一份数据。

下面举个例子说明`Cow<T>`的应用：
```rust
use std::borrow::Cow;

fn abs_all(input: &mut Cow<[i32]>) {
    for i in 0..input.len() {
        let v = input[i];
        if v < 0 {
            input.to_mut()[i] = -v;
        }
    }

    println!("value: {:?}", input);
}

fn main() {
    // 只读，不写，没有发生复制操作
    let a = [0, 1, 2];
    let mut input = Cow::from(&a[..]);
    abs_all(&mut input);
    assert_eq!(input, Cow::Borrowed(a.as_ref()));

    // 写时复制， 在读到-1的时候发生复制
    let b = [0, -1, -2];
    let mut input = Cow::from(&b[..]);
    abs_all(&mut input);
    assert_eq!(input, Cow::Owned(vec![0,1,2]) as Cow<[i32]>);

    // 没有写时复制，因为已经拥有所有权
    let mut input = Cow::from(vec![0, -1, -2]);
    abs_all(&mut input);
    assert_eq!(input, Cow::Owned(vec![0,1,2]) as Cow<[i32]>);
    
    let v = input.into_owned();
    assert_eq!(v, [0, 1, 2]);
}
```
上面这个用例已经讲明了`Cow<T>`的使用，下面我们继续探索一下`Cow<T>`的实现细节。重点关注`to_mut`及`into_owned`的实现。
- `to_mut` ：就是返回数据的可变引用，如果没有数据的所有权，则复制拥有后再返回可变引用；
- `into_owned` ：获取一个拥有所有权的对象（区别与引用），如果当前是借用，则发生复制，创建新的所有权对象，如果已拥有所有权，则转移至新对象。

```rust
impl<B: ?Sized + ToOwned> Cow<'_, B> {
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn to_mut(&mut self) -> &mut <B as ToOwned>::Owned {
        // 如果时借用，则进行复制；如果已拥有所有权，则无需进行复制
        match *self {
            Borrowed(borrowed) => {
                *self = Owned(borrowed.to_owned());
                match *self {
                    Borrowed(..) => unreachable!(),
                    Owned(ref mut owned) => owned,
                }
            }   
            Owned(ref mut owned) => owned,  //这里解释了上个例子中，已拥有所有权的情况，无需再复制
        }
    }
    
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn into_owned(self) -> <B as ToOwned>::Owned {
        // 如果当前是借用，则发生复制，创建新的所有权对象，如果已拥有所有权，则转移至新对象。
        match self {
            Borrowed(borrowed) => borrowed.to_owned(),
            Owned(owned) => owned,
        }
    }
}
```