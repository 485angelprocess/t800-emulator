use crate::mem::{Mem, Stack, STACK_SIZE};

type RTYPE = i32;
type ATYPE = i32;
const MASK: u32 = 0xFFFFFFF0;

pub enum DirectOp{
    JUMP, // j Jump
    LDLP, // jdlp load local pointer
    PFIX, // prefix
    LDNL, // load non-local
    LDC,  // load constant
    LDNLP, // load non-local pointer
    NFIX, // negative prefix
    LDL, // Load local
    ADC,  // Add constant
    CALL, // Call subroutine
    CJ,   // condition jump
    AJW,  // adjust workspace
    EQC, // equals constant
    STL, // Store local
    STNL, // store non local
    OPR,  // operate
}

pub enum IndirectOp{
    REV
}

pub enum Flag{
    ERROR
}

fn mask4(v: RTYPE) -> RTYPE{
    (v << 4) >> 4
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProcState{
    ACTIVE,
    ENABLING,
    WAITING,
    READY,
    IDLE
}

pub struct Proc{
    stack: Stack,
    pc: ATYPE,
    workspace: ATYPE,
    operand:  RTYPE,
    error: bool,
    state: ProcState,
    mem: Mem
}

impl Proc{
    pub fn new(m: Mem) -> Self{
        Self{
            stack: Stack::new(),
            pc: 0,
            workspace: 0,
            operand: 0,
            error : false,
            state: ProcState::ACTIVE,
            mem: m
        }
    }
    
    /*********Instructions ***********/
    fn ldc(&mut self, value: RTYPE){
        // LDC instruction
        // Note check how lowest nibble works here
        // this should only be pushing 4 bits, but does it clear upper bits
        self.operand = mask4(self.operand) + value;
        self.stack.push(self.operand);
        self.operand = 0;
    }
    
    fn adc(&mut self, value: RTYPE){
        self.operand = mask4(self.operand) + value;
        
        let a = self.stack.a();
        
        // Add while checking for overflow
        if let Some(result) = a.checked_add(self.operand){
            self.stack.set(0, result);
        }
        else{
            self.stack.set(0, a.wrapping_add(self.operand));
            self.error = true;
        }
        
        self.operand = 0;
    }
    
    fn prefix(&mut self, value: RTYPE){
        self.operand = mask4(self.operand) + value;
        self.operand = self.operand << 4;
    }
    
    fn neg_prefix(&mut self, value: RTYPE){
        // complement
        self.operand = !(mask4(self.operand) + value);
        // shift up
        self.operand = self.operand << 4;
    }
    
    fn ldl(&mut self, value: RTYPE) {
        // 4 * (workspace pointer + operand)
        let a = (value << 2) + self.workspace;
        let result = self.mem.read(a);
        self.stack.push(result);
    }
    
    fn ldlp(&mut self, value: RTYPE){
        self.stack.push(self.workspace + (value << 2));
    }
    
    fn stl(&mut self, value: RTYPE){
        let a = (value << 2) + self.workspace;
        println!("Writing {}: {}", a, self.stack.a());
        self.mem.write(a, self.stack.a());
    }
    
    fn ldnl(&mut self, value: RTYPE){
        // Load non local
        let a = (value << 2) + self.stack.a();
        self.stack.set(0, self.mem.read(a));
    }
    
    fn stnl(&mut self, value: RTYPE){
        // Store non local
        // Writes contents of B register into address pointed to by A
        let a = (value << 2) + self.stack.a();
        self.mem.write(a, self.stack.b());
    }
    
    fn ldnlp(&mut self, value: RTYPE){
        let a = (value << 2) + self.stack.a();
        self.stack.set(0, a);
    }
    
    fn ajw(&mut self, value: RTYPE){
        // Adjust workspace pointer
        // if value is negative, this allocates more memory
        // if vlaue is postive, this dellocates memry
        self.operand = mask4(self.operand) + value;
        self.workspace = self.workspace + (self.operand << 2);
        self.operand = 0;
    }
            
    fn jump(&mut self, value: RTYPE){
        self.operand = mask4(self.operand) + value;
        self.pc += self.operand;
        self.operand = 0;
        // Allow other processes to run
        self.state = ProcState::IDLE;
    }
            
    fn call(&mut self, value: RTYPE){
        // Pushes C, B, A and instruction pointer to workspace
        self.mem.write(self.workspace, self.stack.c());
        self.mem.write(self.workspace - 4, self.stack.b());
        self.mem.write(self.workspace - 8, self.stack.a());
        self.mem.write(self.workspace - 12, self.pc);
        
        self.workspace = self.workspace - 12;
        
        // Jumps to relative location
        self.operand = mask4(self.operand) + value;
        self.pc = self.pc + self.operand;
        self.operand = 0;
    }
            
    fn cj(&mut self, value: RTYPE){
        // Jumps if A is zero
        if self.stack.a() == 0{
            self.operand = mask4(self.operand) + value;
            self.pc += self.operand;
            self.operand = 0;
            // IDLES?
        }
        else{
            self.stack.pop();
        }
    }
            
    fn eqc(&mut self, value: RTYPE){
        self.operand = mask4(self.operand) + value;
        if self.stack.a() == self.operand{
            self.stack.push(1);
        }
        else{
            self.stack.push(0);
        }
        self.operand = 0;
    }
            
    fn operate(&mut self, value: i32){
        let group = self.operand >> 4;
        match group{
            0x0 => self.indirect_0(value),
            0x4 => self.indirect_4(value),
            _ => panic!("Unimplemented indirect family: {}", self.operand)
        }
        self.operand = 0;
    }
                            
    /**********Indirect instructions *********/
    fn indirect_0(&mut self, value: i32){
        match value{
            0x0 => self.reverse(),
            0x5 => self.add(),
            _ => panic!("Unimplemented command for indirect family 0, {}", value)
        }
    }
    
    fn indirect_4(&mut self, value: i32){
        match value{
            0x3 => self.alt(),
            0x4 => self.alt_wait(),
            0x5 => self.alt_end(),
            0xB => self.logical_and(),
            _ => panic!("Unimplemented command for indirect family 4 (alt): {}", value)
        }
    }
    
    fn reverse(&mut self){
        // Swap A and B registers
        self.stack.swap();
    }
    
    fn add(&mut self){
        let a = self.stack.a();
        let b = self.stack.b();
        if let Some(result) = a.checked_add(b){
            self.stack.set(0, result);
        }
        else{
            self.stack.set(0, a.wrapping_add(b));
            self.error = true;
        }
        self.stack.set(1, self.stack.c());
    }
     
    fn logical_and(&mut self){
        // A equals the bitwise AND of A and B
        self.stack.set(0, self.stack.a() & self.stack.b());
        self.stack.set(1, self.stack.c());
    }
     
    // Alt mode managements       
    fn alt(&mut self){
        panic!("Alt command not implemented");
    }

    fn alt_wait(&mut self){
        panic!("Alt wait not implemented");
    }
            
    fn alt_end(&mut self){
        panic!("Alt command not implemented");
    }
            
    /********** Debug methods ***********/
    pub fn peek(&self, index: usize) -> RTYPE{
        self.stack.get(index)
    }
    
    pub fn set_workspace_pointer(&mut self, value: ATYPE){
        self.workspace = value;
    }
    
    pub fn flag(&self, f: Flag) -> bool{
        match f{
            Flag::ERROR => self.error
        }
    }
    
    pub fn poke(&mut self, index: usize, value: RTYPE){
        self.stack.set(index, value);
    }
    
    pub fn program_counter(&self) -> ATYPE{
        self.pc
    }
    
    pub fn workspace_pointer(&self) -> ATYPE{
        self.workspace
    }
    
    pub fn report_state(&self){
        println!("Register contents");
        for i in 0..STACK_SIZE{
            println!("Reg {}: {}", i, self.stack.get(i));
        }
    }
     
    /********* run instruction *****************/ 
    pub fn run(&mut self, op: DirectOp, value: RTYPE){
        self.state = ProcState::ACTIVE;
        self.pc += 1; // Increment program counter
        match op{
            // Load constant pushes constant into stack 0
            DirectOp::LDC  => self.ldc(value), // load constant
            DirectOp::ADC  => self.adc(value), // add constant
            DirectOp::LDL  => self.ldl(value), // load local
            DirectOp::LDLP => self.ldlp(value), // local local pointer
            DirectOp::STL  => self.stl(value), // store local
            DirectOp::LDNL => self.ldnl(value), // Load non local
            DirectOp::STNL => self.stnl(value), // store non local
            DirectOp::LDNLP=> self.ldnlp(value), // load non local pointer
            DirectOp::PFIX => self.prefix(value),
            DirectOp::NFIX => self.neg_prefix(value),
            DirectOp::AJW  => self.ajw(value),
            DirectOp::JUMP => self.jump(value),
            DirectOp::CJ   => self.cj(value),
            DirectOp::CALL => self.call(value),
            DirectOp::EQC  => self.eqc(value),
            DirectOp::OPR  => self.operate(value),
        }
    }
    
    pub fn state(&self) -> ProcState{
        self.state
    }
}