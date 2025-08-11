use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex, MutexGuard};
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
pub const CLOCK_REG_1: i32 = REG_BASE + 0x04;
pub const FRONT_PTR_0: i32 = REG_BASE + 0x08;
pub const FRONT_PTR_1: i32 = REG_BASE + 0x0C;
pub const BACK_PTR_0: i32  = REG_BASE + 0x10;
pub const BACK_PTR_1: i32  = REG_BASE + 0x14;

pub const REGISTER_CACHE: i32 = 0x8000_002Cu32 as i32;

pub const TERMINAL_OUT: i32 = 0x0001_0000;

pub const DRAM_SIZE: usize = 1024*1024*128*2;

pub struct Mem{
    contents: Arc<Mutex<Vec<u8>>>
}

impl Clone for Mem{
    fn clone(&self) -> Self {
        Self{
            contents: self.contents.clone()
        }
    }
}

impl Mem{
    pub fn new(capacity: usize) -> Self{
        let m = Mem{
            contents: Arc::new(Mutex::new(vec![0; capacity]))
        };
        m
    }
    
    fn get(&self) -> MutexGuard<'_, Vec<u8>>{
        self.contents.lock().unwrap()
    }
    
    pub fn write(&mut self, address: i32, value: i32){
        // Write a word
        assert!(address % 4 == 0);
        for i in 0..4{
            self.write_byte(address + i, ((value >> (8*i)) & 0xFF) as u8);
        }
    }
    
    pub fn read(&self, address: i32) -> i32{
        assert!(address % 4 == 0);
        let mut sum = 0;
        for i in 0..4{
            sum += (self.get()[(address+i) as usize] as i32) << (i*8);
        }
        sum
    }
    
    pub fn read_byte(&self, address: i32) -> u8{
        return self.get()[address as usize]
    }
    
    pub fn write_byte(&mut self, address: i32, value: u8){
        match address{
            TERMINAL_OUT => {
                let _ = stdout().write(&[value]);
            },
            _ => {
                self.get()[address as usize] = value;
            }
        }
        
    }
}

pub const STACK_SIZE: usize = 3;

pub struct Stack{
    reg: [i32; STACK_SIZE]
}

impl Stack{
    pub fn new() -> Self{
        Self{
            reg: [0; STACK_SIZE]
        }
    }
    pub fn push(&mut self, value: i32){
        let c = &mut self.reg;
        for i in (1..STACK_SIZE).rev(){
            c[i] = c[i - 1];
        }
        c[0] = value;
    }
    
    pub fn pop(&mut self) -> i32{
        let c = &mut self.reg;
        
        let v = c[0];
        c[0] = c[1];
        c[1] = c[2];
        v
    }
    
    pub fn swap(&mut self){
        // Swap A and B registers
        let c = &mut self.reg;
        let a = c[0];
        c[0] = c[1];
        c[1] = a;
    }
    
    /// Register A
    pub fn a(&self) -> i32{
        self.reg[0]
    }
    
    /// Register B
    pub fn b(&self) -> i32{
        self.reg[1]
    }
    
    /// Register C
    pub fn c(&self) -> i32{
        self.reg[2]
    }
    
    
    /// Get register value via index
    /// 0 - A, 1 - B, 2 - C
    pub fn get(&self, index: usize) -> i32{
        self.reg[index]
    }
    
    /// Set register via index
    /// 0 - A, 1 - B, 2 - C
    pub fn set(&mut self, index: usize, value: i32){
        self.reg[index] = value;
    }
}