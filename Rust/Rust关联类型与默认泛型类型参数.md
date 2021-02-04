### 一、关联类型（associated types）
我们阅读Rust程序的时候，有时候会出现如下的代码：
```rust
trait Iterator {
    type Item; 
    fn next(&mut self) -> Option<Self::Item>;
}
```
 下面是上面代码的注解：Iterator trait 有一个关联类型 ```Item```。```Item```是一个占位类型，同时 next 方法会返回 ```Option<Self::Item>```类型的值。这个 ```trait```的实现者会指定 ```Item```的具体类型。

这里的```type```用法就是关联类型。


>关联类型（associated types）是一个将类型占位符与 trait 相关联的方式，这样 trait 的方法签名中就可以使用这些占位符类型。trait 的实现者会针对特定的实现在这个类型的位置指定相应的具体类型。如此可以定义一个使用多种类型的 trait，直到实现此 trait 时都无需知道这些类型具体是什么。


使用关联类型的代码示例如下:
```rust
pub trait Watch {
    type Item;
    fn inner(&self) -> Option<Self::Item>;
}

struct A {
    data: i32,
}

impl Watch for A {
    type Item = i32;
    fn inner(&self) -> Option<Self::Item> {
        Some(self.data)
    }
}

struct B {
    data: String,
}

impl Watch for B {
    type Item = String;
    fn inner(&self) -> Option<Self::Item> {
        Some(self.data.clone())
    }
}

fn main() {
    let a = A{data: 10};
    let b = B{data: String::from("B")};
    assert_eq!(Some(10), a.inner());
    assert_eq!(Some(String::from("B")), b.inner());
}
```

###  二、默认泛型类型参数
我们还会碰到如下类型的代码：
```rust
#[lang = "add"]
pub trait Add<RHS = Self> {
    type Output;
    
    #[must_use]
    fn add(self, rhs: RHS) -> Self::Output;
}
```
这里```Add<RHS = Self>```是默认泛型类型参数，表示如果不显示指定泛型类型，就默认泛型类型为```Self```。

>当使用泛型类型参数时，可以为泛型指定一个默认的具体类型。如果默认类型就足够的话，这消除了为具体类型实现 trait 的需要。为泛型类型指定默认类型的语法是在声明泛型类型时使用 <PlaceholderType=ConcreteType>。

使用默认泛型类型参数的示例代码如下：
```rust
pub trait Watch<Inner=String> {
    type Item;
    fn inner(&self) -> Option<Self::Item>;
    fn info(&self) -> Inner;
}

struct A {
    data: i32,
}

impl Watch<i32> for A {
    type Item = i32;
    fn inner(&self) -> Option<Self::Item> {
        Some(self.data)
    }
    fn info(&self) -> i32 {
        println!("A inner is {}", self.data);
        self.data
    }
}

struct B {
    data: String,
}

impl Watch for B {
    type Item = String;
    fn inner(&self) -> Option<Self::Item> {
        Some(self.data.clone())
    }
    fn info(&self) -> String {
        println!("B inner is {}", self.data);
        self.data.clone()
    }
}

fn main() {
    let a = A{data: 10};
    let b = B{data: String::from("B")};
    assert_eq!(10, a.info());
    assert_eq!(Some(String::from("B")), b.inner());
}
```