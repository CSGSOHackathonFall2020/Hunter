extern crate amd64;
extern crate asm_syntax;
extern crate memmap;

use amd64::{Register, Assembler};
use asm_syntax::{Displacement, Immediate};
use memmap::MmapMut;

use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
use std::num::{NonZeroI8, NonZeroI16, NonZeroI32};
use std::slice::Iter;
use std::str::Chars;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, PartialEq)]
enum Instruction {
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
    fn incp(&self) -> bool {
        match self {
            Instruction::Increment(_) => true,
            _ => false,
        }
    }

    fn decp(&self) -> bool {
        match self {
            Instruction::Decrement(_) => true,
            _ => false,
        }
    }

    fn forwardp(&self) -> bool {
        match self {
            Instruction::Forward(_) => true,
            _ => false,
        }
    }

    fn backp(&self) -> bool {
        match self {
            Instruction::Back(_) => true,
            _ => false,
        }
    }

    fn add_count(&self) -> u8 {
        match self {
            Instruction::Increment(i) | Instruction::Decrement(i) => *i,
            _ => unreachable!(),
        }
    }

    fn move_count(&self) -> i32 {
        match self {
            Instruction::Forward(i) | Instruction::Back(i) => *i as i32,
            _ => unreachable!(),
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <Directory>", args[0]);
        std::process::exit(1);
    }

    let mut buf = String::new();
    let mut file = File::open(&args[1]).unwrap();
    file.read_to_string(&mut buf).unwrap();

    let program = parse(&mut buf.chars().peekable(), false);
    let code = compile(&program);

    let mut m = MmapMut::map_anon(code.len()).unwrap();
    for (i, c) in code.iter().enumerate() {
        m[i] = *c;
    }
    let m = m.make_exec().unwrap();
    let myfunc = unsafe { std::mem::transmute::<*const u8, fn(*mut u8)>(m.as_ptr()) };
    // 10 MB
    let mut data = vec![0; 10*1024*1024];
    let d = data.as_mut_ptr();
    myfunc(d);
}

fn parse<'a>(code: &mut Peekable<Chars<'a>>, loopp: bool) -> Vec<Instruction> {
    let mut program = Vec::new();
    while let Some(c) = code.next() {
        match c {
            '>' => {
                let mut i = 1;
                while let Some('>') = code.peek() {
                    code.next();
                    i += 1;
                }
                program.push(Instruction::Forward(i));
            }
            '<' => {
                let mut i = 1;
                while let Some('<') = code.peek() {
                    code.next();
                    i += 1;
                }
                program.push(Instruction::Back(i));
            }
            '+' => {
                let mut i = 1;
                while let Some('+') = code.peek() {
                    code.next();
                    i += 1;
                }
                program.push(Instruction::Increment(i));
            }
            '-' => {
                let mut i = 1;
                while let Some('-') = code.peek() {
                    code.next();
                    i += 1;
                }
                program.push(Instruction::Decrement(i));
            }
            '.' => program.push(Instruction::Print),
            ',' => program.push(Instruction::Read),
            '[' => {
                let c = parse(code, true);
                if c.len() == 0 {
                    continue;
                } else {
                    program.push(optimize_loop(c));
                }
            }
            ']' => if loopp {
                return program;
            } else {
                // TODO
                panic!("End of loop but not in loop");
            },
            _ => (),
        }
    }

    if loopp {
        // TODO
        panic!("Missing end of loop");
    }

    program
}

fn optimize_loop(c: Vec<Instruction>) -> Instruction {
    if c.len() == 1 && c[0] == Instruction::Decrement(1) {
        return Instruction::SetToZero;
    } else if c.len() == 4 {
        // ->+<
        if c[0].decp() && c[1].forwardp() && c[2].incp() && c[3].backp() {
            if c[0].add_count() == 1 && c[1].move_count() == c[3].move_count() {
                return Instruction::Add(c[1].move_count(), c[2].add_count());
            }
        // >+<-
        } else if c[0].forwardp() && c[1].incp() && c[2].backp() && c[3].decp() {
            if c[3].add_count() == 1 && c[0].move_count() == c[2].move_count() {
                return Instruction::Add(c[0].move_count(), c[1].add_count());
            }
        // -<+>
            /*
        } else if c[0].decp() && c[1].backp() && c[2].incp() && c[3].forwardp() {
            if c[0].add_count() == 1 && c[1].move_count() == c[3].move_count() {
                println!("in {} {}", -c[1].move_count(), c[2].add_count());
                return Instruction::Add(-c[1].move_count(), c[2].add_count());
            }
            */
        // <+>-
        } else if c[0].backp() && c[1].incp() && c[2].forwardp() && c[3].decp() {
            if c[3].add_count() == 1 && c[0].move_count() == c[2].move_count() {
                return Instruction::Add(-c[0].move_count(), c[1].add_count());
            }
        // ->-<
        } else if c[0].decp() && c[1].forwardp() && c[2].decp() && c[3].backp() {
            if c[0].add_count() == 1 && c[1].move_count() == c[3].move_count() {
                return Instruction::Sub(c[1].move_count(), c[2].add_count());
            }
        // >-<-
        } else if c[0].forwardp() && c[1].decp() && c[2].backp() && c[3].decp() {
            if c[3].add_count() == 1 && c[0].move_count() == c[2].move_count() {
                return Instruction::Sub(c[0].move_count(), c[1].add_count());
            }
        // -<->
        } else if c[0].decp() && c[1].backp() && c[2].decp() && c[3].forwardp() {
            if c[0].add_count() == 1 && c[1].move_count() == c[3].move_count() {
                return Instruction::Sub(-c[1].move_count(), c[2].add_count());
            }
        // <->-
        } else if c[0].backp() && c[1].decp() && c[2].forwardp() && c[3].decp() {
            if c[3].add_count() == 1 && c[0].move_count() == c[2].move_count() {
                return Instruction::Sub(-c[0].move_count(), c[1].add_count());
            }
        }
    }

    Instruction::Loop(c)
}

fn compile(program: &[Instruction]) -> Vec<u8> {
    let mut asm = Assembler::new();
    _compile(&mut program.iter(), &mut asm);
    asm.ret();
    return asm.finish();
}

fn _compile(program: &mut Iter<Instruction>, asm: &mut Assembler) {
    for inst in program {
        match inst {
            Instruction::Increment(i) => asm.add_addr_u8(Register::RDI, *i),
            Instruction::Decrement(i) => asm.sub_addr_u8(Register::RDI, *i),
            Instruction::Forward(i) => asm.add_reg_u32(Register::RDI, *i),
            Instruction::Back(i) => asm.sub_reg_u32(Register::RDI, *i),
            Instruction::Print => {
                asm.push_reg(Register::RDI);
                // syscall
                asm.mov_reg_imm(Register::RAX, Immediate::U32(1));
                // buf
                asm.mov_reg_reg(Register::RSI, Register::RDI);
                // fd: stdout
                asm.mov_reg_imm(Register::RDI, Immediate::U32(1));
                // count
                asm.mov_reg_imm(Register::RDX, Immediate::U32(1));
                asm.syscall();
                asm.pop_reg(Register::RDI);
            }
            Instruction::Read => {
                asm.push_reg(Register::RDI);
                // syscall
                asm.mov_reg_imm(Register::RAX, Immediate::U32(0));
                // buf
                asm.mov_reg_reg(Register::RSI, Register::RDI);
                // fd: stdin
                asm.mov_reg_imm(Register::RDI, Immediate::U32(0));
                // count
                asm.mov_reg_imm(Register::RDX, Immediate::U32(1));
                asm.syscall();
                asm.pop_reg(Register::RDI);
            }
            Instruction::SetToZero => asm.mov_addr_imm(Register::RDI, None, Immediate::U8(0)),
            Instruction::Add(i, x) => {
                let i = *i;
                asm.mov_reg_addr(Register::AL, Register::RDI, None);
                asm.sub_addr_reg(Register::RDI, Register::AL, None);
                if *x > 1 {
                    asm.mov_reg_imm(Register::DL, Immediate::U8(*x));
                    asm.mul(Register::DL);
                }
                let disp = if i >= i8::MIN as i32 && i <= i8::MAX as i32 {
                    Displacement::Disp8(NonZeroI8::new(i as i8).unwrap())
                } else if i >= i16::MIN as i32 && i <= i16::MAX as i32 {
                    Displacement::Disp16(NonZeroI16::new(i as i16).unwrap())
                } else {
                    Displacement::Disp32(NonZeroI32::new(i).unwrap())
                };
                asm.add_addr_reg(Register::RDI, Register::AL, Some(disp));
            }
            Instruction::Sub(i, x) => {
                let i = *i;
                asm.mov_reg_addr(Register::AL, Register::RDI, None);
                asm.sub_addr_reg(Register::RDI, Register::AL, None);
                if *x > 1 {
                    asm.mov_reg_imm(Register::DL, Immediate::U8(*x));
                    asm.mul(Register::DL);
                }
                let disp = if i >= i8::MIN as i32 && i <= i8::MAX as i32 {
                    Displacement::Disp8(NonZeroI8::new(i as i8).unwrap())
                } else if i >= i16::MIN as i32 && i <= i16::MAX as i32 {
                    Displacement::Disp16(NonZeroI16::new(i as i16).unwrap())
                } else {
                    Displacement::Disp32(NonZeroI32::new(i).unwrap())
                };
                asm.sub_addr_reg(Register::RDI, Register::AL, Some(disp));
            }
            Instruction::Loop(p) => {
                let loopl = make_label();
                let donel = make_label();
                // Initial loop test
                asm.mov_reg_addr(Register::R9, Register::RDI, None);
                asm.and_reg_imm(Register::R9, 0xff);
                asm.test(Register::R9, Register::R9);
                asm.jz(donel.clone());

                asm.label(loopl.clone());
                _compile(&mut p.iter(), asm);
                asm.mov_reg_addr(Register::R9, Register::RDI, None);
                asm.and_reg_imm(Register::R9, 0xff);
                asm.test(Register::R9, Register::R9);
                asm.jnz(loopl);
                asm.label(donel);
            }
        }
    }
}

fn make_label() -> String {
    static LABEL: AtomicUsize = AtomicUsize::new(0);
    LABEL.fetch_add(1, Ordering::SeqCst).to_string()
}
