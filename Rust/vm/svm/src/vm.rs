use std::vec::Vec;
use std::collections::HashMap;
use crate::instruction::*;
use std::borrow::BorrowMut;
use std::io::Read;

// 基于栈的虚拟机需要一个栈，所有的操作都是基于栈的。
static mut VmStack: Vec<u8> = Vec::new();
static mut PointerCounter: usize = 0;

pub struct Vm {
    dispatch_table: HashMap<OpCode, Box<FnMut()>>,
    codes: Vec<u8>,
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            dispatch_table: HashMap::new(),
            codes: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        // let add = || {
        //     let value = self.pop() + self.pop();
        //     self.push(value);
        // };

        // let sub = || {
        //     let second = self.pop();
        //     let value = self.pop() - second;
        //     self.push(value);
        // };
        self.dispatch_table.insert(OpCode::Add, Box::new(|| {
            let value = pop() + pop();
            push(value);
        }));

        self.dispatch_table.insert(OpCode::Sub, Box::new(|| {
            let second = pop();
            let value = pop() - second;
            push(value);
        }));

        self.dispatch_table.insert(OpCode::Mul, Box::new(|| {
            let value = pop() * pop();
            push(value)
        }));

        self.dispatch_table.insert(OpCode::Div, Box::new(|| {
            let second = pop();
            let value = pop() / second;
            push(value);
        }));

        self.dispatch_table.insert(OpCode::Print, Box::new(|| {
            print();
        }));

        self.dispatch_table.insert(OpCode::Jmp, Box::new(|| {
            let addr = pop() as usize;
            // if addr >= self.codes.len() {
            //     panic!("invalid point counter addr.");
            // }
            unsafe {
                PointerCounter = addr;        // fixme: 这里缺少错误处理
            }
        }));
    }

    pub fn import_codes(&mut self, codes: &[u8]) {
        self.codes = codes.to_vec();
    }

    pub fn run(&mut self) {
        let opcodes_len = self.codes.len();
        unsafe {
            while PointerCounter < opcodes_len {
                let opcode = self.codes[PointerCounter];
                if is_opcode(opcode) {      // 如果是操作码，解析操作码并执行
                    let opcode = OpCode::from(opcode);
                    if let Some(action) = self.dispatch_table.get_mut(&opcode) {
                        action();
                    } else {
                        panic!("opcode not found.");
                    }
                } else {        // 如果不是操作码就是操作数，压栈处理
                    let value = opcode - IR_OFFSET;     //减掉指令偏移量
                    push(value);
                }
                PointerCounter = PointerCounter + 1;
            }
        }
    }
}

fn pop() -> u8 {
    unsafe {
        let top = VmStack.pop().unwrap();
        top
    }
}

fn push(value: u8) {
    unsafe {
        VmStack.push(value);
    }
}

fn print() {
    let terminal = console::Term::stdout();

    let top = unsafe {
        VmStack.pop().unwrap()
    };

    let s = format!("{}", top);
    terminal.write_line(s.as_str());
}

fn read_line() {
    let mut term = console::Term::buffered_stdout();
    // let input = term.read_line().unwrap();
    let mut input = Vec::new();
    term.read(input.as_mut_slice()).unwrap();
    for i in input {
        push(i);
    }
}

fn fi() {
    let f = pop();
    let t = pop();
    let condition = pop();
    if condition != 0 {
        push(t);
    } else {
        push(f);
    }
}

