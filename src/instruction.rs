#[derive(Debug, PartialEq)]
pub enum Instruction {
    Increment(u8),
    Decrement(u8),
    Forward(u32),
    Back(u32),
    Print,
    Read,
    Loop(Vec<Instruction>),
    SetToZero,
    Add(i32, u8),
    Sub(i32, u8),
}

impl Instruction {
    pub fn incp(&self) -> bool {
        match self {
            Instruction::Increment(_) => true,
            _ => false,
        }
    }

    pub fn decp(&self) -> bool {
        match self {
            Instruction::Decrement(_) => true,
            _ => false,
        }
    }

    pub fn forwardp(&self) -> bool {
        match self {
            Instruction::Forward(_) => true,
            _ => false,
        }
    }

    pub fn backp(&self) -> bool {
        match self {
            Instruction::Back(_) => true,
            _ => false,
        }
    }

    pub fn add_count(&self) -> u8 {
        match self {
            Instruction::Increment(i) | Instruction::Decrement(i) => *i,
            _ => unreachable!(),
        }
    }

    pub fn move_count(&self) -> i32 {
        match self {
            Instruction::Forward(i) | Instruction::Back(i) => *i as i32,
            _ => unreachable!(),
        }
    }
}
