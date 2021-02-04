Rust既不能避免一个```trait```与另一个```trait```拥有相同名称的方法，也不能阻止为同一类型同时实现这两个```trait```。甚至可以直接在类型上实现开始已经有的同名方法！当然，当调用这些同名方法时，你必须要告诉Rust我们使用哪一个。

下面示例代码说明了具体用法：
```rust
pub trait Airplane {
    fn speed(&self){
        println!("airplane default speed=800");
    }

    fn state(){
        println!("fly state");
    }
}

pub trait Boat {
    fn speed(&self);
    fn state(){
        println!("boat state");
    }
}

struct Airboat;

impl Airboat{
    fn speed(&self) {
        println!("airboat speed 0~800");
    }
    fn state(){
        println!("airboat state");
    }
}

impl Airplane for Airboat {
    fn speed(&self){
        println!("airboat in airplane state speed=800");
    }
}

impl Boat for Airboat {
    fn speed(&self){
        println!("airboat in boat state speed=60");
    }
}

fn main() {
    let a = Airboat;
    Airboat::state();
    a.speed();  //默认调用自身的实现
    <Airboat as Airplane>::state(); //完全限定语法
    Airplane::speed(&a);    //显示语法指定调用Airplane方法
    <Airboat as Airplane>::speed(&a);
    <Airboat as Boat>::state();
    Boat::speed(&a);
}
```
运行结果：
```
airboat state
airboat speed 0~800
fly state
airboat in airplane state speed=800
airboat in airplane state speed=800
boat state
airboat in boat state speed=60
```

>通常，完全限定语法定义为：```<Type as Trait>::function(receiver_if_method, next_arg, ...)```

在存在调用相同名称方法时，重要的是告诉编译器你调用的是具体那个方法，上面的示例代码给出了调用相同名称时的调用方法。