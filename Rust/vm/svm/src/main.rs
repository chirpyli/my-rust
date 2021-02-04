#[macro_use]
extern crate log;

mod instruction;
mod vm;

use console;
use std::io::Read;
use std::borrow::BorrowMut;

fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();
    info!("simple stack vm impl.");

    // let mut terminal = console::Term::stdout();
    // terminal.write_line("terminal print test.");
    //
    // std::thread::sleep(std::time::Duration::from_secs(3));
    // terminal.clear_line().unwrap();
    //
    // let s = terminal.read_line().unwrap();
    // info!("read from terminal: {}", s);

    let mut svm = vm::Vm::new();
    svm.init();

    // print(1+2)编译成字节码的结果[17,18,0,4]， 这里需要编写一个编译器，将print(1+2)语句翻译为字节码
    let codes = vec![17u8, 18, 0, 4];
    svm.import_codes(codes.as_slice());
    svm.run();
}
