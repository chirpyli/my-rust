use std::pin::Pin;

pub async fn run_pinning() {
    let mut t1 = Test::new("t1");
    let mut t2 = Test::new("t2");
    t1.init();
    t2.init();
    info!("t1: <{},{}>", t1.a(), t1.b());
    info!("t2: <{},{}>", t2.a(), t2.b());

    std::mem::swap(&mut t1, &mut t2);
    info!("t1: <{},{}>", t1.a(), t1.b());

}

#[derive(Debug)]
struct Test {
    a: String,
    b: *const String,
}

impl Test {
    fn new(txt: &str) -> Self {
        Test {
            a: String::from(txt),
            b: std::ptr::null(),
        }
    }

    fn init(&mut self) {
        let self_ref: *const String = &self.a;
        info!("{} pointer: {:?}", self.a(), self_ref);
        self.b = self_ref;
    }

    fn a(&self) -> &str {
        &self.a
    }

    fn b(&self) -> &String {
        unsafe {
            &*(self.b)
        }
    }
}