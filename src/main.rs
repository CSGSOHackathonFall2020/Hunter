extern crate amd64;
extern crate asm_syntax;
extern crate memmap;

use amd64::{Register, Assembler};
use asm_syntax::Immediate;
use memmap::MmapMut;

use std::fs::File;
use std::io::Read;
use std::slice::Iter;
use std::sync::atomic::{AtomicUsize, Ordering};

enum Instruction {
    Increment,
    Decrement,
    Forward,
    Back,
    Print,
    Read,
    Loop(Vec<Instruction>)
}

fn main() {
    let mut buf = String::new();
    let mut file = File::open("rot13.bf").unwrap();
    file.read_to_string(&mut buf).unwrap();

    let program = parse(&mut buf.chars(), false);
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

fn parse<'a>(code: &mut std::str::Chars<'a>, loopp: bool) -> Vec<Instruction> {
    let mut program = Vec::new();
    while let Some(c) = code.next() {
        match c {
            '>' => program.push(Instruction::Forward),
            '<' => program.push(Instruction::Back),
            '+' => program.push(Instruction::Increment),
            '-' => program.push(Instruction::Decrement),
            '.' => program.push(Instruction::Print),
            ',' => program.push(Instruction::Read),
            '[' => program.push(Instruction::Loop(parse(code, true))),
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
            Instruction::Increment => asm.add_addr_u8(Register::RDI, 1),
            Instruction::Decrement => asm.sub_addr_u8(Register::RDI, 1),
            Instruction::Forward => asm.add_reg_u8(Register::RDI, 1),
            Instruction::Back => asm.sub_reg_u8(Register::RDI, 1),
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
            Instruction::Loop(p) => {
                let loopl = make_label();
                let donel = make_label();
                asm.label(loopl.clone());
                asm.mov_reg_addr(Register::R9, Register::RDI, None);
                asm.and_reg_imm(Register::R9, 0xff);
                asm.test(Register::R9, Register::R9);
                asm.jz(donel.clone());
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
