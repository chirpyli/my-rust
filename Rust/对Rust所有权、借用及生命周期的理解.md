Rust的内存管理中涉及所有权、借用与生命周期这三个概念，下面是个人的一点粗浅理解。

#### 一、从内存安全的角度理解Rust中的所有权、借用、生命周期
要理解这三个概念，你首要想的是这么做的出发点是什么——内存安全，这是Rust非常强调的一点。可以这么理解，所有权、借用与生命周期很大程度上是为内存安全而设计的。

所有权，从内存安全的角度思考，如果一个实例有多个所有者，这个实例就很可能不安全，多个所有者都可能操作这个实例产生竞争，解决的办法是让他只有一个所有者，这样就无论如何也无法产生竞争(Data race)。新问题来了，如果其他人想访问这个实例怎么办？借用。

借用，有点类似与引用，可以理解为我不去获取这个实例的所有权，我只借用一下，用完后就还回去，只使用而不占有。有两种借用，可变借用与不可变借用。后面有规则说明。

生命周期，对生命周期的理解可以暂时这么认为：<font color=blue>它的目的是避免悬垂指针的出现或者是保证引用的有效性。</font> 如果你进行一个借用，而那个被借用的实例超出作用域或者已经被释放了（即你的生命周期比你借用的实例的生命周期长），此时，你指向的是一个无意义的地址，可能会因此产生严重错误，而且这种错误又难发现，你必须对所借用的实例的生命周期非常清晰，才不会产生错误。在C++中，没有显式的生命周期概念，程序员在处理这种情况时必须十分小心且需对程序十分清晰，否则就有可能出现悬垂指针。在Rust中，通过生命周期，明确指出各个对象的生命周期，以保证你的生命周期与你借用的实例的生命周期处在一个交集中，你的生命周期不会超过你借用的实例的生命周期。尽管在Rust中也比较难以处理，但是Rust编译器会显示的指明你可能产生的生命周期错误，强制你处理生命周期问题，从而避免可能引发的错误。

可以说，编写Rust程序，编译器会根据所有权、借用、生命周等规则对代码进行检查，如果不合乎Rust的规则，就不会编译通过(尽管你认为这样的代码目前没有Bug，但问题是编译器认为这样有可能产生Bug)，减少了未来程序出现内存问题的几率。

#### 二、所有权
一般程序管理计算机内存的方式主要有两种，一种是垃圾回收机制（GC），一种是程序员自己分配和释放内存，这两种方式各自优缺点暂且不谈，Rust采用的是第三种方式：通过所有权系统管理内存，编译器在编译时会根据一系列的规则进行检查。在运行时，所有权系统的任何功能都不会减慢程序。

>所有权规则：
>1. Rust 中的每一个值都有一个被称为其所有者（owner）的变量。
>2. 值有且只有一个所有者。
>3. 当所有者（变量）离开作用域，这个值将被丢弃。

```rust
{
    let s = String::from("hello"); // 从此处起，s 是有效的

    // 使用 s
}                                  // 此作用域已结束，
                                   // s 不再有效
```
Rust中内存在拥有它的变量离开作用域后就被自动释放。编译器可以识别出变量的生命周期，在编译时就知道何时释放该变量占用的内存。

#### 三、借用&引用
借用与引用是相对比较好理解的，为了保证内存安全，Rust制定了如下的引用规则。强制在编译期间进行如下的规则检查，
>引用规则
>1. 在任意给定时间，要么只能有一个可变引用，要么只能有多个不可变引用。
>2. 引用必须总是有效。

这2条规则的核心要点是要避数据竞争和悬垂指针的出现。

下面用C++和Rust的代码做对比，可以看出Rust编译器对代码进行严格的安全检查，对有可能出现问题的代码显示提出编译错误，尽管代码可能并不会产生数据竞争或悬垂指针。

C++代码编译通过，没有产生编译错误：
```c++
#include <iostream>
using namespace std;

int main(){
  int a = 10;
  int &b = a;
  a = 100;
  b = 200;

  cout<<a<<endl;
  cout<<b<<endl;

  return 0;
}

//编译通过
```
Rust代码编译不通过，尽管代码目前看起来不会产生可能的错误，但编译器进行严格的检查，编译不通过：
```rust
fn main() {
    let mut a = 10;
    let ref b = a;
    let ref mut c = a;
    //编译报错：cannot borrow `a` as immutable because it is also borrowed as mutable
}
```

#### 四、生命周期
>生命周期（lifetimes），它是一类允许我们向编译器提供引用如何相互关联的泛型。Rust 的生命周期功能允许在很多场景下借用值的同时仍然使编译器能够检查这些引用的有效性。

##### 【1】c++与rust的不同
生命周期的主要目标是避免悬垂引用，它会导致程序引用了非预期引用的数据。Rust编译器会强制进行生命周期的检查，如果不符合规则，就编译报错，强制你编写符合生命周期规则的代码。下面对比Rust和C++的代码说明Rust增加生命周期的动机与好处。

Rust代码，如果不符合生命周期规则，就编译错误。
```rust
fn main() {
    let r;
    {
        let x = 5;
        r = &x;
    }

    println!("r: {}", r);
}
```
编译产生如下错误：
```rust
error[E0597]: `x` does not live long enough

  --> src/main.rs:76:18
   |
76 |             r = &x;
   |                  ^ borrowed value does not live long enough
77 |         }
   |         - `x` dropped here while still borrowed
...
80 |     }
   |     - borrowed value needs to live until here
```


C++代码，即使会产生悬垂指针，编译还是会通过，并且正常运行，这样悬垂指针可能会导致非常隐晦的bug，造成排查bug的困难。
```c++
#include <iostream>
using namespace std;

int main(){
  int *r = new int(5);
  int *x = r;
  delete r;

  cout<< *r <<endl;
  cout<< *x <<endl;

  return 0;
}
```
输出结果：
```
0
2608    //这种情况可能会产生较为隐晦的Bug
```
编译通过，但有可能会产生Bug，也有可能不会引发Bug，但是一旦由此产生Bug，可能比较难找。

通过上面的代码，我们可以看到增加生命周期对内存安全的好处。一定程度上，生命周期是Rust强调内存安全的产物。下面再次通过代码说明为什么需要生命周期。

##### 【2】为什么需要生命周期？
简单的理解是当编译器判断不出引用的有效性，不能够判断出引用内存是否安全的时候，就需要程序员通过在Rust代码中明确的生命周期注解为编译器指明每个引用的生命周期，告诉编译器足够的信息，使编译器能够判断这段引用是否有效，符不符合规则要求，不会出现垂悬引用这种不安全的操作。

下面这段代码会产生垂悬引用：
```rust
fn main() {
    let a = 10;
    let m;
    {
        let b = 100;

        m = max_num(&a, &b);
        assert_eq!(100, *m);
    }
    println!("max num is {}", m);
}

fn max_num(a: &i32, b: &i32) -> &i32 {
    if *a > *b {
        return *a;
    }
    *b
}
```
在Rust中，上面的代码会产生编译错误：
```
error[E0106]: missing lifetime specifier
  --> src/main.rs:23:33
   |
23 | fn max_num(a: &i32, b: &i32) -> &i32 {
   |                                 ^ expected lifetime parameter
   |
   = help: this function's return type contains a borrowed value, but the signature does
 not say whether it is borrowed from `a` or `b`
```
编译器提示需要添加生命周期注解，因为编译器目前不能判断最后是返回a的引用还是b的。所以编译器无法通过作用域来确定返回的引用是否总是有效。如果返回的是a的引用，那么上面的不会产生垂悬引用，如果返回的是b的引用，那么就会产生垂悬引用，产生安全隐患。

怎么办呢？通过增加生命周期注解告诉编译器更多的信息，使它能够判断出引用是否有效。
```rust
// 增加生命周期注解
fn max_num<'a>(a: &'a i32, b: &'a i32) -> &'a i32 {
    if *a > *b {
        return a;
    }
    b
}
```
通过增加生命周期注解，编译器就能够判断引用的有效性：
```rust
error[E0597]: `b` does not live long enough
  --> src/main.rs:16:26
   |
16 |         m = max_num(&a, &b);
   |                          ^ borrowed value does not live long enough
17 |         assert_eq!(100, *m);
18 |     }

   |     - `b` dropped here while still borrowed
19 |     println!("max num is {}", m);
20 | }
   | - borrowed value needs to live until here
```
编译器告诉我们，b不满足生命周期注解的要求，因为我们的生命周期注解中说明了返回的引用的生命周期在a和b中较短的那个生命周期结束之前保持有效。具体分析，这里m的生命周期是a和b生命周期的交集。

根据编译器提示信息，更改为以下代码，编译成功：
```rust
fn main() {
    // let inner = Inner{data:"inner data."};
    let a = 1000;
    let b = 100;    //将b的生命周期延长至与a，m相同
    let m;
    {
        m = max_num(&a, &b);
        assert_eq!(100, *m);
    }
    println!("max num is {}", m);
}
```
##### 【3】生命周期注解语法
生命周期注解并不改变任何引用的生命周期的长短。注解语法如下示例：
```rust
&i32        // a reference
&'a i32     // a reference with an explicit lifetime
&'a mut i32 // a mutable reference with an explicit lifetime
```