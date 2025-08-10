use proc::Proc;

mod proc;
mod mem;
mod parse;

fn main() {
    println!("Hello, world!");
    
    let mut proc = Proc::new(0x1000_0000);
    
    println!("Initial stack: {:?}", proc.get_stack());
    
    let _result = proc.run(0x12);
    
    println!("After ldlp: {:?}", proc.get_stack());
    
    let _result = proc.run(0xF0);
    
    println!("After reverse: {:?}", proc.get_stack());
}
