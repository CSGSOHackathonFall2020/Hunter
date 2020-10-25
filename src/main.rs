extern crate amd64;
extern crate asm_syntax;
extern crate memmap;

use amd64::{Register, Assembler};
use asm_syntax::Immediate;
use memmap::MmapMut;

use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
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
    myfunc(data.as_mut_ptr());
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
                }
                if c.len() == 1 && c[0] == Instruction::Decrement(1) {
                    program.push(Instruction::SetToZero);
                } else {
                    program.push(Instruction::Loop(c));
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
