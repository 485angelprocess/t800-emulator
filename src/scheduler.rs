use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{mem::Mem, proc::{Proc, ProcState}};

enum SchedulePriority{
    LOW,
    HIGH
}

struct Scheduler{
    proc: Rc<RefCell<Proc>>,
    mem: Rc<RefCell<Mem>>
}

impl Scheduler{
    fn step(&mut self){
        let s = self.proc.try_borrow().unwrap().state();
        self.schedule(s);
    }
    
    fn schedule(&mut self, state: ProcState){
        match state{
            ProcState::ACTIVE => (),
            ProcState::IDLE   => self.reschedule(false),
            ProcState::HALTED => todo!("Handle and unwind halted process")
        }
    }
    
    fn reschedule(&mut self, store_stack: bool){
        // swap processes
        println!("Reschedule");
        
        // Store stack
        
        // Store workspace and program counter
    }
    
    fn active_priority(&self) -> SchedulePriority{
        SchedulePriority::LOW
    }
    
    
}

