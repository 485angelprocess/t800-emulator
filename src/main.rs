use asm::Assemble;
#[warn(unused_imports)]

use proc::Proc;

mod proc;
mod mem;
mod parse;

mod asm;

mod visual;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    
    let mut proc = Proc::new(0x1000_0000);
    
    let mut asm = Assemble::new();
    asm.setup(&proc);
    
    let mut machine: Vec<u8> = Vec::new();
    
    if let Ok(lines) = read_lines("lib/hello.s"){        
        for l in lines{
            let ml = l.unwrap();
            match asm.read_line(ml.as_str()){
                Some(values) => {
                    for v in values{
                        machine.push(v);
                    }
                },
                None => ()
            }
        }
    }
    
    for i in 0..machine.len(){
        match proc.run(machine[i]){
            Err(e) => {
                println!("Got error on line {}: {:#2x}: {:?}", i, machine[i], e);
            },
            _ => ()
        }
    }
}
