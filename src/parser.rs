use instruction::Instruction;

use std::iter::Peekable;
use std::str::Chars;

pub fn parse<'a>(code: &mut Peekable<Chars<'a>>, loopp: bool) -> Vec<Instruction> {
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
        } else if c[0].decp() && c[1].backp() && c[2].incp() && c[3].forwardp() {
            if c[0].add_count() == 1 && c[1].move_count() == c[3].move_count() {
                if -c[1].move_count() != -36 {
                    return Instruction::Add(-c[1].move_count(), c[2].add_count());
                }
            }
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

