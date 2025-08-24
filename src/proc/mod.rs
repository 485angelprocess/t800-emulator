mod secondary;

use std::collections::{hash_map::IntoKeys, HashMap};

use secondary::{define_w_prefix, define_wo_prefix};

use std::rc::Rc;

use crate::mem::*;

type RTYPE = i32;
type ATYPE = i32;

// bits
const GO_TO_SNP_BIT: usize = 0x02;
const HALT_ON_ERROR_BIT: usize = 0x80;
const ERROR_FLAG: usize = 0x8000_0000;

const NOT_PROCESS_P: i32 = 0x8000_0000u32 as i32;

enum OpVal{
    Int(RTYPE),
    List(Vec<RTYPE>),
    Null
}

#[repr(u8)]
#[derive(Debug)]
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum DirectOp{
    JUMP,
    LDLP,
    PFIX,
    LDNL,
    LDC,
    LDNLP,
    NFIX,
    LDL,
    ADC,
    CALL,
    CJ,
    AJW,
    EQC,
    STL,
    STNL,
    OPR
}

enum IndirectOp{
    None // TODO
}

#[derive(Debug)]
pub enum OpErr{
    Err,
    Overflow,
    DivideByZero,
    NotSingle,
    Count
}

type OperandType = u8;
/// Function result
type OpResult = Result<OpVal, OpErr>;
/// Generic function which alters processor state
type OpFn = Rc<dyn Fn(&mut Proc, OperandType) -> OpResult>;
type IndirectOpFn =Rc<dyn Fn(&mut Proc) -> OpResult>;

pub enum Flag{
    ERROR
}

#[derive(PartialEq)]
#[repr(u8)]
enum Priority{
    Low = 1,
    High = 0
}

struct ProcLibrary{
    pub direct: [OpFn; 16],
    indirect_id: HashMap<usize, usize>,
    indirect_fn: Vec<IndirectOpFn>,
    indirect_name: Vec<String>
}

fn direct() -> [OpFn; 16]{
    [
        // Jump
        Rc::new(|p, v|{
            let operand = p.shift_operand(v);
            p.pc = p.pc + operand;
            if p.priority() == Priority::Low{
                p.deschedule();
            }
            Ok(OpVal::Null)
        }),
        // LDLP
        Rc::new(|p, v|{
            let operand = p.shift_operand(v);
            Ok(OpVal::Int(p.workspace + (operand << 2)))
        }),
        // PFIX
        Rc::new(|p, v|{
            p.operand = (p.operand).wrapping_add(v as RTYPE) << 4;
            Ok(OpVal::Null)
        }),
        // LDNL
        Rc::new(|p, v|{
            let operand = p.shift_operand(v) << 2;
            let a = p.stack.pop();
            Ok(OpVal::Int(p.mem.read(a.wrapping_add(operand))))
        }),
        // LDC
        Rc::new(|p, v|{
           let operand = p.shift_operand(v);
           p.stack.push(operand);
           Ok(OpVal::Null) 
        }),
        // LDNLP
        Rc::new(|p, v|{
            let operand = p.shift_operand(v) << 2;
            let a= p.stack.pop();
            Ok(OpVal::Int(a.wrapping_add(operand)))
        }),
        // NFIX
        Rc::new(|p, v|{
             p.operand = (!(p.operand).wrapping_add(v as RTYPE)) << 4;
             Ok(OpVal::Null)
        }),
        // LDL
        Rc::new(|p, v|{
           let address = p.workspace.wrapping_add(p.shift_operand(v) << 2);
           Ok(OpVal::Int(p.mem.read(address)))
        }),
        // ADC
        Rc::new(|p, v|{
            let a = p.stack.pop();
            let operand = p.shift_operand(v);
            Ok(OpVal::Int(a.wrapping_add(operand)))
        }),
        // CALL
        Rc::new(|p, v|{
            // Store register stack
            let a = p.stack.pop();
            let b = p.stack.pop();
            let c = p.stack.pop();
            p.mem.write(p.workspace - 4, a);
            p.mem.write(p.workspace - 8, b);
            p.mem.write(p.workspace - 12, c);
            p.mem.write(p.workspace - 16, p.pc);
            p.workspace -= 16;
            p.pc += p.shift_operand(v) - 4;
            Ok(OpVal::Null)
        }),
        // CJ
        Rc::new(|p, v|{
           let a = p.stack.pop();
           if a == 0{
               p.pc = p.pc.wrapping_add(p.shift_operand(v));
           }
           Ok(OpVal::Null)
        }),
        // AJW
        Rc::new(|p, v|{
            let operand = p.shift_operand(v) << 2;
            p.update_wdesc((p.workspace + operand) | (p.descriptor & 0b1));
            
            Ok(OpVal::Null)
        }),
        // EQC
        Rc::new(|p, v|{
            let operand = p.shift_operand(v);
            let a = p.stack.pop();
            if a == operand{
                Ok(OpVal::Int(1))
            }
            else{
                Ok(OpVal::Int(0))
            }
        }),
        // STL
        Rc::new(|p, v|{
            let offset = p.shift_operand(v) << 2;
            let a = p.stack.pop();
            p.mem.write(p.workspace + offset, a);
            Ok(OpVal::Null)
        }),
        // STNL
        Rc::new(|p, v|{
            let b = p.stack.b();
            let a = p.stack.pop();
            
            let offset = p.shift_operand(v) << 2;
            p.mem.write(a+offset, b);
            Ok(OpVal::Int(a+offset))
        }),
        // OPR
        Rc::new(|p, v|{
            let operand = p.shift_operand(v) as usize;
            let name = p.library.get_indirect_name(operand);
            p.library.get_indirect(operand).clone()(p)
        })
    ]
}

impl ProcLibrary{
    fn new() -> Self{
        Self{
            direct: direct(),
            indirect_id: HashMap::new(),
            indirect_fn: Vec::new(),
            indirect_name: Vec::new()
        }
    }
    
    /// Define new indirect function
    fn define_indirect<T: for<'a> Fn(&'a mut Proc) -> OpResult + 'static>(&mut self, name: &str, opcode: usize, f: T){
        let id = self.indirect_fn.len();
        self.indirect_id.insert(opcode, id);
        self.indirect_fn.push(Rc::new(f));
        self.indirect_name.push(name.to_string());
    }
    
    /// Get function pointer of indirect function
    fn get_indirect(&self, opcode: usize) -> &IndirectOpFn{
        &self.indirect_fn[self.indirect_id[&opcode]]
    }
    
    /// Get all indirect codes
    fn get_indirect_codes(&self) -> IntoKeys<usize, usize>{
        self.indirect_id.clone().into_keys()
    }
    
    /// Get name of indirect operation
    fn get_indirect_name(&self, opcode: usize) -> String{
        self.indirect_name[self.indirect_id[&opcode]].clone()
    }
}

pub struct Proc{
    stack: Stack,
    
    // Main registers
    pc: ATYPE,
    workspace: ATYPE,
    operand:  RTYPE,
    
    descriptor: RTYPE,
    
    status: usize,
    error: usize,
    
    // Additional Register
    
    // Data space
    mem: Mem,
    
    // Library
    library: ProcLibrary
}

impl Proc{
    pub fn new(workspace: ATYPE) -> Self{
        let mut p = Proc {
            stack: Stack::new(),
            pc: ATYPE::default(),
            workspace: workspace,
            status: 0,
            error: 0,
            descriptor: 0,
            operand: RTYPE::default(),
            mem: Mem::new(DRAM_SIZE),
            library: ProcLibrary::new()
        };
        p.setup();
        p
    }
    
    /// Throw error flag in processor
    pub fn throw_error(&mut self, e: OpErr){
        todo!("Error flag not implemented");
    }
    
    /// Get program counter
    pub fn program_counter(&self) -> i32{
        return self.pc;
    }
    
    /// Get active workspace pointer
    pub fn workspace_pointer(&self) -> i32{
        return self.workspace
    }
    
    /// Get a register from the stack
    pub fn get_reg(&self, i: usize) -> i32{
        self.stack.get(i)
    }
    
    /// Get names of indirect operations
    pub fn get_indirect_ops(&self) -> Vec<(String, usize)>{
        let mut maps = Vec::new();
        for op in self.library.get_indirect_codes(){
            maps.push( (self.library.get_indirect_name(op), op) );
        }
        maps
    }
    
    fn setup(&mut self){
        define_wo_prefix(&mut self.library);
        define_w_prefix(&mut self.library);
    }
    
    /// Get the pointer at the front of the process queue
    fn get_front_pointer(&self, pri: Priority) -> RTYPE{
        match pri{
            Priority::Low => self.mem.read(FRONT_PTR_1),
            Priority::High => self.mem.read(FRONT_PTR_0)
        }
    }
    
    /// Set the pointer at the start of the process queue
    fn set_front_pointer(&mut self, pri: Priority, v: RTYPE){
        match pri{
            Priority::Low => self.mem.write(FRONT_PTR_1, v),
            Priority::High => self.mem.write(FRONT_PTR_0, v)
        }
    }
    
    /// Get the pointer at the back of the process queue
    fn get_back_pointer(&self, pri: Priority) -> RTYPE{
        match pri{
            Priority::Low => self.mem.read(BACK_PTR_1),
            Priority::High => self.mem.read(BACK_PTR_0)
        }
    }
    
    /// Set the pointer at the back of the process queue
    fn set_back_pointer(&mut self, pri: Priority, v: RTYPE){
        match pri{
            Priority::Low => self.mem.write(BACK_PTR_1, v),
            Priority::High => self.mem.write(BACK_PTR_0, v)
        }
    }
    
    /// Get the value of the clock register
    fn get_clock_register(&self, _pri:Priority) -> RTYPE{
        0 // STUB
    }
    
    /// Check if the current process should be switched
    /// If so, adds itself to the back of the process queue
    /// and set status flag
    fn deschedule(&mut self){
        // We save data at a few locations
        self.mem.write(self.workspace - 4, self.pc);
        if self.get_front_pointer(Priority::Low) == NOT_PROCESS_P{
            self.set_front_pointer(Priority::Low, self.workspace);
        }
        else{
            // Update the last value in queue
            self.mem.write(self.get_back_pointer(Priority::Low) - 8, self.workspace);
        }
        self.set_back_pointer(Priority::Low, self.workspace);
        self.status = self.status | GO_TO_SNP_BIT;
        //self.mem.write(self.workspace - 8, );
    }
    
    fn priority(&self) -> Priority{
        if self.descriptor & 1 == 0{
            Priority::High
        }
        else{
            Priority::Low
        }
    }
    
    pub fn mem_reference(&self) -> Mem{
        self.mem.clone()
    }
    
    pub fn shift_operand(&mut self, op: u8) -> RTYPE{
        let o = (self.operand).wrapping_add(op as RTYPE);
        self.operand = 0;
        o
    }
    
    /// Save registers to memory
    fn save_registers(&mut self){
        // Save registers space
        self.mem.write(REGISTER_CACHE, self.descriptor);
        if self.descriptor != NOT_PROCESS_P + 1{
            self.mem.write(REGISTER_CACHE+4, self.pc);
            self.mem.write(REGISTER_CACHE+8, self.stack.a());
            self.mem.write(REGISTER_CACHE+12, self.stack.b());
            self.mem.write(REGISTER_CACHE+16, self.stack.c());
            self.mem.write(REGISTER_CACHE+20, self.status as i32);
            // TODO: Cache float stack
        }
    }
    
    /// Load registers from memory
    fn restore_registers(&mut self){
        let wdesc = self.mem.read(REGISTER_CACHE);
        self.update_wdesc(wdesc);
        if self.descriptor != NOT_PROCESS_P + 1{
            self.pc = self.mem.read(REGISTER_CACHE+4);
            self.stack.set(0, self.mem.read(REGISTER_CACHE+8));
            self.stack.set(1, self.mem.read(REGISTER_CACHE+12));
            self.stack.set(2, self.mem.read(REGISTER_CACHE+16));
            self.status = self.mem.read(REGISTER_CACHE+20) as usize;
            
            // TODO: Restore float stack
        }
    }
    
    fn update_wdesc(&mut self, wdesc: RTYPE){
        self.descriptor = wdesc;
        self.workspace = wdesc & (!0b11);
    }
    
    fn activate_process(&mut self){
        // TODO clear Oreg
        self.pc = self.mem.read(self.workspace - 4);
    }
    
    pub fn run_process(&mut self, wdesc: RTYPE){
        let wpri = wdesc & 0b1;
        let waddress = wdesc & !0b11;
        
        match self.priority(){
            Priority::High => {
                if wpri > 0{
                    // Add low priority to queue
                    if self.get_front_pointer(Priority::Low) == NOT_PROCESS_P{
                        self.set_front_pointer(Priority::Low, waddress);
                    }
                    else{
                        let bp = self.get_back_pointer(Priority::Low);
                        self.mem.write(bp - 8, waddress);
                    }
                    self.set_back_pointer(Priority::Low, waddress);
                }
                else{
                    // Adding high priority to queue
                    if self.get_front_pointer(Priority::High) == NOT_PROCESS_P{
                        self.set_front_pointer(Priority::High, waddress);
                    }
                    else{
                        let bp = self.get_back_pointer(Priority::High);
                        self.mem.write(bp - 8, waddress);
                    }
                    self.set_back_pointer(Priority::High, waddress);
                }
            },
            Priority::Low => {
                if wpri == 0{
                    // Switch immediately to new high priority process
                    self.save_registers();
                    self.update_wdesc(wdesc);
                    self.status = self.status & (ERROR_FLAG | HALT_ON_ERROR_BIT);
                    self.activate_process();
                }
            }
        }
    }
    
    pub fn run(&mut self, instruction: u8) -> Result<(), OpErr>{
        let op = instruction >> 4;
        let v = instruction & 0xF;
        
        // TODO check how branch or others work
        self.pc += 1;
        
        let result = match self.library.direct[op as usize].clone()(self, v){
            Ok(v) => v,
            Err(e) => {
                println!("Got error {:?}", e);
                self.error = 1;
                OpVal::Null
            }
        };
        
        match result{
            OpVal::Int(v) => self.stack.push(v),
            OpVal::List(values) =>{
                for v in values{
                    self.stack.push(v);
                }
            }
            _ => ()
        };
        Ok(())
    }
    
    pub fn get_stack(&self) -> Vec<RTYPE>{
        let mut v = Vec::new();
        for i in 0..STACK_SIZE{
            v.push(self.stack.get(i));
        }
        v
    }
}

#[cfg(test)]
mod processor_tests{
    use super::*;
    
    #[test]
    fn ldc(){
        let mut proc = Proc::new(0x1000);
        let m = proc.mem_reference();
        
        let _ = proc.run(0x42);
        let _ = proc.run(0xD0);
        
        assert_eq!(m.read(0x1000), 0x2);
    }
    
    #[test]
    fn pfix(){
        let mut proc = Proc::new(0x1000);
        let m = proc.mem_reference();
        
        let _ = proc.run(0x24);
        let _ = proc.run(0x23);
        let _ = proc.run(0x42);
        let _ = proc.run(0xD0);
        assert_eq!(m.read(0x1000), 0x432);
    }
    
    #[test]
    fn adc(){
        let mut proc = Proc::new(0x1000);
        let m = proc.mem_reference();
        
        let _ = proc.run(0x46);
        let _ = proc.run(0x83);
        let _ = proc.run(0xD0);
        
        assert_eq!(m.read(0x1000), 0x9);
    }
    
    #[test]
    fn ldlp(){
        let mut proc = Proc::new(0x1000);
        let m = proc.mem_reference();
        
        // puts stack pointer + 4*operand
        let _ = proc.run(0x12);
        let _ = proc.run(0xD0);
        
        assert_eq!(m.read(0x1000), 0x1008); 
    }
}