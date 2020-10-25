extern crate amd64;
extern crate asm_syntax;
extern crate memmap;

mod compiler;
mod instruction;
mod parser;

use memmap::MmapMut;

use std::fs::File;
use std::io::Read;



fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <Directory>", args[0]);
        std::process::exit(1);
    }

    let mut buf = String::new();
    let mut file = File::open(&args[1]).unwrap();
    file.read_to_string(&mut buf).unwrap();

    let program = parser::parse(&mut buf.chars().peekable(), false);
    let code = compiler::compile(&program);

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
