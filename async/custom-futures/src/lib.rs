
#[macro_use]
extern crate log;


mod timer;

pub use timer::TimerFuture;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
