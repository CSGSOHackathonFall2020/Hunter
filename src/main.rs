extern crate amd64;
extern crate memmap;

use memmap::MmapMut;

use std::fs::File;
use std::io::Read;

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
    let mut file = File::open("t.bf").unwrap();
    file.read_to_string(&mut buf).unwrap();

    let program = parse(&mut buf.chars(), false);

    // TODO
    let code = Vec::new();
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
            '>' => program.push(Instruction::Increment),
            '<' => program.push(Instruction::Decrement),
            '+' => program.push(Instruction::Forward),
            '-' => program.push(Instruction::Back),
            '.' => program.push(Instruction::Print),
            ',' => program.push(Instruction::Read),
            '[' => program.push(Instruction::Loop(parse(code, true))),
            ']' => if loopp {
                return program;
            } else {
                // TODO
                panic!("End of loop but not in loop");
            },
            // TODO
            _ => panic!("invalid character"),
        }
    }

    if loopp {
        // TODO
        panic!("Missing end of loop");
    }

    program
}
