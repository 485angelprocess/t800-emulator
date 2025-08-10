use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::cell::RefCell;
/*
const LINK0_OUTPUT: i32 = 0x8000_0000;
const LINK1_OUTPUT: i32 = 0x8000_0004;
const LINK2_OUTPUT: i32 = 0x8000_0008;
const LINK3_OUTPUT: i32 = 0x8000_000C;
const LINK0_INPUT: i32  = 0x8000_0010;
const LINK1_INPUT: i32  = 0x8000_0014;
const LINK2_INPUT: i32  = 0x8000_0018;
const LINK3_INPUT: i32  = 0x8000_001C;
const EVENT_CHANNEL: i32= 0x8000_0020;
const HIGH_POINTER: i32 = 0x8000_0024;
const LOW_POINTER: i32  = 0x8000_0028;
const LOW_WORKSPACE:i32 = 0x8000_002C;
*/
pub const REG_BASE: i32 = 0x1000_0000u32 as i32;
pub const CLOCK_REG_0: i32 = REG_BASE + 0x00;
pub const CLOCK_REG_1: i32 = REG_BASE + 0x00;
pub const FRONT_PTR_0: i32 = REG_BASE + 0x00;
pub const FRONT_PTR_1: i32 = REG_BASE + 0x00;
pub const BACK_PTR_0: i32  = REG_BASE + 0x00;
pub const BACK_PTR_1: i32  = REG_BASE + 0x00;

pub const REGISTER_CACHE: i32 = 0x8000_002Cu32 as i32;

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
    
    pub fn read_byte(&self, address: i32) -> u8{
        let base_address = (address >> 2) << 2;
        let offset = address & 0b11;
        let c = self.contents.borrow();
        if c.contains_key(&base_address){
            let value = c[&base_address];
            return ((value >> offset) & 0xFF) as u8;
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
    
    pub fn pop(&mut self) -> i32{
        let mut c = self.reg.borrow_mut();
        
        let v = c[0];
        c[0] = c[1];
        c[1] = c[2];
        v
    }
    
    pub fn swap(&mut self){
        // Swap A and B registers
        let mut c = self.reg.borrow_mut();
        let a = c[0];
        c[0] = c[1];
        c[1] = a;
    }
    
    /// Register A
    pub fn a(&self) -> i32{
        self.reg.borrow()[0].clone()
    }
    
    /// Register B
    pub fn b(&self) -> i32{
        self.reg.borrow()[1].clone()
    }
    
    /// Register C
    pub fn c(&self) -> i32{
        self.reg.borrow()[2].clone()
    }
    
    
    /// Get register value via index
    /// 0 - A, 1 - B, 2 - C
    pub fn get(&self, index: usize) -> i32{
        self.reg.borrow()[index].clone()
    }
    
    /// Set register via index
    /// 0 - A, 1 - B, 2 - C
    pub fn set(&self, index: usize, value: i32){
        self.reg.borrow_mut()[index] = value;
    }
}