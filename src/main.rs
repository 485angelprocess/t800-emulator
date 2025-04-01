use std::rc::Rc;
use std::cell::RefCell;

use mem::Mem;
use proc::DirectOp;

mod proc;
mod mem;
mod test;

fn main() {
    println!("Hello, world!");
    
    let m = Mem::new();
    let mut p = proc::Proc::new(m.clone());
    
    p.run(DirectOp::LDC, 10);
    p.report_state();
    p.run(DirectOp::LDC, 5);
    p.report_state();
}
