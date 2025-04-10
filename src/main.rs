use std::rc::Rc;
use std::cell::RefCell;

use mem::Mem;
use proc::DirectOp;

use std::io;

mod proc;
mod mem;
mod test;
mod visual;
mod parse;
mod scheduler;

mod channel;

fn main()  -> Result<(), io::Error> {
    println!("Hello, world!");
    
    let mut tui = visual::ProcessorTui::new();
    
    let program = vec![
        0x42,
        0x44,
        0x46,
        0x47,
        0x22,
        0xF3
    ];
    
    for p in program{
        let inst = parse::parse_op_from_hex(p);
        tui.upload_instruction(inst.0, inst.1);   
    }
    // tui.upload_instruction(DirectOp::LDC, 0x2);
    // tui.upload_instruction(DirectOp::LDC, 0x4);
    // tui.upload_instruction(DirectOp::LDC, 0x6);
    // tui.upload_instruction(DirectOp::LDC, 0x7);
    // tui.upload_instruction(DirectOp::PFIX, 0x2);
    // tui.upload_instruction(DirectOp::OPR, 0x3);
    
    return tui.run();
}
