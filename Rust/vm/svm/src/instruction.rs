pub const IR_OFFSET: u8 = 16;

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum OpCode {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
    Print = 4,
    Jmp,
    If,
    ReadLine,
    Return,
    Call,
    Exit,
}

pub fn is_opcode(opcode: u8) -> bool {
    let min = OpCode::Add as u8;
    let max = OpCode::Exit as u8;
    opcode >= min && opcode <= max
}

impl From<u8> for OpCode {
    fn from(v: u8) -> Self {
        match v {
            0 => OpCode::Add,
            1 => OpCode::Sub,
            2 => OpCode::Mul,
            3 => OpCode::Div,
            4 => OpCode::Print,
            5 => OpCode::Jmp,
            6 => OpCode::If,
            7 => OpCode::ReadLine,
            8 => OpCode::Return,
            9 => OpCode::Call,
            10 => OpCode::Exit,
            _ => panic!("invalid opcode."),
        }
    }
}