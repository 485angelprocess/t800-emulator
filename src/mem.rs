use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::cell::RefCell;

pub struct Mem{
    contents: Rc<RefCell<HashMap<i32, i32>>>
}

impl Clone for Mem{
    fn clone(&self) -> Self {
        Self{
            contents: self.contents.clone()
        }
    }
}

impl Mem{
    pub fn new() -> Self{
        Self{
            contents: Rc::new(RefCell::new(HashMap::new()))
        }
    }
    
    pub fn write(&mut self, address: i32, value: i32){
        assert!(address % 4 == 0);
        let mut c = self.contents.borrow_mut();
        *c.entry(address).or_insert(value) = value;
        
        println!("Contents {:#?}", c);
    }
    
    pub fn read(&self, address: i32) -> i32{
        assert!(address % 4 == 0);
        let c = self.contents.borrow();
        if c.contains_key(&address){
            return c[&address];
        }
        0
    }
}

pub const STACK_SIZE: usize = 3;

pub struct Stack{
    reg: Rc<RefCell<[i32; STACK_SIZE]>>
}

impl Stack{
    pub fn new() -> Self{
        Self{
            reg: Rc::new(RefCell::new([0; STACK_SIZE]))
        }
    }
    pub fn push(&mut self, value: i32){
        let mut c = self.reg.borrow_mut();
        for i in (1..STACK_SIZE).rev(){
            c[i] = c[i - 1];
        }
        c[0] = value;
    }
    
    pub fn a(&self) -> i32{
        self.reg.borrow()[0].clone()
    }
    
    pub fn b(&self) -> i32{
        self.reg.borrow()[1].clone()
    }
    
    pub fn c(&self) -> i32{
        self.reg.borrow()[2].clone()
    }
    
    pub fn get(&self, index: usize) -> i32{
        self.reg.borrow()[index].clone()
    }
    
    pub fn set(&self, index: usize, value: i32){
        self.reg.borrow_mut()[index] = value;
    }
}