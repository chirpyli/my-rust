
#### 关于PartialEq和Eq

##### PartialEq
这里的相等关系指的是实现了如下的关系：
- symmetric: a == b implies b == a; 
- transitive: a == b and b == c implies a == c.

相比`Eq`，少了一个`a == a`的约束。

`PartialEq`的定义如下：
```rust
pub trait PartialEq<Rhs: ?Sized = Self> {
    /// This method tests for `self` and `other` values to be equal, and is used
    /// by `==`.
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn eq(&self, other: &Rhs) -> bool;

    /// This method tests for `!=`.
    #[inline]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}
```
标准库文档中是这样讲的：*This trait allows for partial equality, for types that do not have a full equivalence relation. For example, in floating point numbers NaN != NaN, so floating point types implement PartialEq but not Eq.*
讲到一点两者的不同，浮点数只实现了`PartialEq`，并没有实现`Eq`。但实际使用中，因为“相等”是自己可以定义的，是一个抽象的概念，并没有什么本质的区别。下面举个比较不同类型数据的例子：
```rust
use std::cmp::{PartialEq, Eq};

#[derive(Debug, PartialEq)]
enum BookFormat {
    Paperback,
    Hardback,
    Ebook,
}

#[derive(Debug)]
struct Book {
    isbn: i32,
    format: BookFormat,
}

impl Book {
    pub fn new(isbn: i32, format: BookFormat) -> Self {
        Book {
            isbn,
            format,
        }
    }
}

// Implement <Book> == <BookFormat> comparisons
impl PartialEq<BookFormat> for Book {
    fn eq(&self, other: &BookFormat) -> bool {
        self.format == *other
    }
}

// Implement <BookFormat> == <Book> comparisons
impl PartialEq<Book> for BookFormat {
    fn eq(&self, other: &Book) -> bool {
        *self == other.format
    }
}

fn main() {
    println!("ParticalEq or Eq:");
    let bf1 = BookFormat::Paperback;
    let bf2 = BookFormat::Hardback;
    let book1 = Book::new(1, BookFormat::Paperback);
    let book2 = Book::new(2, BookFormat::Paperback);
    let book3 = Book::new(1, BookFormat::Paperback);
    assert!(book1 != bf2);
    assert!(bf1 == book1);
    println!("book1:{:?}, book2:{:?}, book3:{:?}", book1, book2, book3);

    println!("NaN {}", f64::NAN == f64::NAN);
    println!("{}", 0.30000000000000000000000000000000000001 == 0.3);   // 浮点数受限于系统精度，比较浮点数的大小问题时要注意这个问题。
}
```

##### Eq
这里的相等关系，是指实现了如下的关系：
- reflexive: a == a;
- symmetric: a == b implies b == a; and
- transitive: a == b and b == c implies a == c.

上面的浮点数因为没有实现`a==a`，即`f64::NAN == f64::NAN`，所以只实现了`ParticalEq`。

`Eq`的定义如下：
```rust
pub trait Eq: PartialEq<Self> {
    // this method is used solely by #[deriving] to assert
    // that every component of a type implements #[deriving]
    // itself, the current deriving infrastructure means doing this
    // assertion without using a method on this trait is nearly
    // impossible.
    //
    // This should never be implemented by hand.
    #[doc(hidden)]
    #[inline]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn assert_receiver_is_total_eq(&self) {}
}
```
在标准库文档中是这么将`Eq`的：*Because Eq has no extra methods, it is only informing the compiler that this is an equivalence relation rather than a partial equivalence relation.** 意思就是，`Eq`并没有额外的方法，它的作用，只是起到一个标识的作用，强调这里是相等关系而不是部分相等的关系。 

举个例子：
```rust
use std::cmp::{PartialEq, Eq};

#[derive(Debug, PartialEq)]
enum BookFormat {
    Paperback,
    Hardback,
    Ebook,
}

#[derive(Debug, PartialEq)]
struct Book {
    isbn: i32,
    format: BookFormat,
}

impl Book {
    pub fn new(isbn: i32, format: BookFormat) -> Self {
        Book {
            isbn,
            format,
        }
    }
}

impl Eq for Book {}     // 前提是必须实现PartialEq<Book>

fn main() {
    println!("ParticalEq or Eq:");
    let book1 = Book::new(1, BookFormat::Paperback);
    let book2 = Book::new(2, BookFormat::Paperback);
    let book3 = Book::new(1, BookFormat::Paperback);
    assert!(book1 != book2);
    assert!(book1 == book3);
    println!("book1:{:?}, book2:{:?}, book3:{:?}", book1, book2, book3);
}
```