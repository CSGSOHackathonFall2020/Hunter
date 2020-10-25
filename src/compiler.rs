use instruction::Instruction;

use amd64::{Register, Assembler};
use asm_syntax::{Displacement, Immediate};

use std::slice::Iter;
use std::num::{NonZeroI8, NonZeroI16, NonZeroI32};
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn compile(program: &[Instruction]) -> Vec<u8> {
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
