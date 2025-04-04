use std::io::Error;

use crate::mem::{Mem, Stack, STACK_SIZE};

type RTYPE = i32;
type ATYPE = i32;

#[derive(Clone, Copy, PartialEq)]
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
    IDLE,
    HALTED
}

struct ProcFlag{
    pub error: bool,
    pub halt_on_error: bool
}

impl Default for ProcFlag{
    fn default() -> Self {
        Self{
            error: false,
            halt_on_error: true
        }
    }
}

pub struct Proc{
    stack: Stack,
    pc: ATYPE,
    workspace: ATYPE,
    operand:  RTYPE,
    flag: ProcFlag,
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
            flag: ProcFlag::default(),
            state: ProcState::ACTIVE,
            mem: m
        }
    }
    
    /// Set process to idle
    fn deschedule(&mut self){
        self.state = ProcState::IDLE;
    }
    
    /// Stops process operation
    fn halt(&mut self){
        self.state = ProcState::HALTED;
    }
    
    /// Set error flag, halts if HaltOnError set
    fn set_error(&mut self){
        self.flag.error = true;
        if self.flag.halt_on_error{
            self.halt();
        }
    }
    
    /*********Instructions ***********/
    /// Load constant
    fn ldc(&mut self, value: RTYPE){
        // LDC instruction
        // Note check how lowest nibble works here
        self.operand = mask4(self.operand) + value;
        self.stack.push(self.operand);
        self.operand = 0;
        // this should only be pushing 4 bits, but does it clear upper bits
    }
    
    /// Add constant
    fn adc(&mut self, value: RTYPE){
        self.operand = mask4(self.operand) + value;
        
        let a = self.stack.a();
        
        // Add while checking for overflow
        if let Some(result) = a.checked_add(self.operand){
            self.stack.set(0, result);
        }
        else{
            self.stack.set(0, a.wrapping_add(self.operand));
            self.set_error();
        }
        
        self.operand = 0;
    }
    
    /// Set prefix, loads 4 bits into operand register
    fn prefix(&mut self, value: RTYPE){
        self.operand = mask4(self.operand) + value;
        self.operand = self.operand << 4;
    }
    
    /// Negative prefix, loads 4 bits into operand register and then complements
    fn neg_prefix(&mut self, value: RTYPE){
        // complement
        self.operand = !(mask4(self.operand) + value);
        // shift up
        self.operand = self.operand << 4;
    }
    
    /// Load from workspace
    fn ldl(&mut self, value: RTYPE) {
        // 4 * (workspace pointer + operand)
        let a = (value << 2) + self.workspace;
        let result = self.mem.read(a);
        self.stack.push(result);
    }
    
    /// Load workspace pointer into register
    fn ldlp(&mut self, value: RTYPE){
        self.stack.push(self.workspace + (value << 2));
    }
    
    /// Store register A into workspace
    fn stl(&mut self, value: RTYPE){
        let a = (value << 2) + self.workspace;
        println!("Writing {}: {}", a, self.stack.a());
        self.mem.write(a, self.stack.a());
    }
    
    /// Load non local, read from location set by register A
    fn ldnl(&mut self, value: RTYPE){
        // Load non local
        let a = (value << 2) + self.stack.a();
        self.stack.set(0, self.mem.read(a));
    }
    
    /// Store non local, write to location pointed by register A
    fn stnl(&mut self, value: RTYPE){
        // Store non local
        // Writes contents of B register into address pointed to by A
        let a = (value << 2) + self.stack.a();
        self.mem.write(a, self.stack.b());
    }
    
    /// Load non local pointer, sets A to an offset of the pointer in A
    fn ldnlp(&mut self, value: RTYPE){
        let a = (value << 2) + self.stack.a();
        self.stack.set(0, a);
    }
    
    /// Adjust workspace by 4*operand register
    fn ajw(&mut self, value: RTYPE){
        // Adjust workspace pointer
        // if value is negative, this allocates more memory
        // if vlaue is postive, this dellocates memry
        self.operand = mask4(self.operand) + value;
        self.workspace = self.workspace + (self.operand << 2);
        self.operand = 0;
    }
        
    /// Offset program counter by operand register    
    /// Desceduling point
    fn jump(&mut self, value: RTYPE){
        self.operand = mask4(self.operand) + value;
        self.pc += self.operand;
        self.operand = 0;
        // Allow other processes to run
        self.deschedule();
    }
    
    /// Store state in stack, and then jump to new location        
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
            
    /// Conditional jump, jumps if A is 0
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
            
    /// Tests if A is equal to operand register, pushes 1 into register stack if true, 0 if not
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
    
    /// Map indirect commands      
    fn operate(&mut self, value: i32){
        let group = self.operand >> 4;
        match group{
            0x0 => self.indirect_0(value),
            0x1 => self.indirect_1(value),
            0x2 => self.indirect_2(value),
            0x3 => self.indirect_3(value),
            0x4 => self.indirect_4(value),
            0x5 => self.indirect_5(value),
            0x7 => self.indirect_7(value),
            _ => panic!("Unimplemented indirect family: {:#01X}", group)
        }
        self.operand = 0;
    }
                            
    /**********Indirect instructions *********/
    fn indirect_0(&mut self, value: i32){
        match value{
            0x0 => self.reverse(),
            0x2 => self.byte_subscript(),
            0x5 => self.add(),
            _ => panic!("Unimplemented command for indirect family 0, {:#01X}", value)
        }
    }
    
    fn indirect_1(&mut self, value: i32){
        match value{
            0x3 => self.check_subscript_from_zero(),
            _ => panic!("Unimplemented command for indirect family 1, {:#01X}", value)
        }
    }
    
    fn indirect_2(&mut self, value: i32){
        match value{
            0x7 => self.clear_halt_error(),
            _ => panic!("Unimplented command for indirect family 2: {:#01X}", value)
        }
    }
    
    fn indirect_3(&mut self, value: i32){
        match value{
            0x4 => self.bcnt(),
            _ => panic!("Unimplemented command for indirect family 3, {:#01X}", value)
        }
    }
    
    fn indirect_4(&mut self, value: i32){
        match value{
            0x3 => self.alt(),
            0x4 => self.alt_wait(),
            0x5 => self.alt_end(),
            0xB => self.logical_and(),
            0xC => self.check_single(),
            0xD => self.ccnt1(),
            _ => panic!("Unimplemented command for indirect family 4 (alt): {:#01X}", value)
        }
    }
    
    fn indirect_5(&mut self, value: i32){
        match value{
            0x6 => self.check_word(),
            _ => panic!("Unimplemented command for indirect family 5: {:#01X}", value)
        }
    }
    
    fn indirect_7(&mut self, value: i32){
        match value{
            0x4 => self.crc_word(),
            0x5 => self.crc_byte(),
            0x6 => self.bitcnt(),
            0x7 => self.bit_reverse_word(),
            0x8 => self.bit_rev_n_bits(),
            _ => panic!("Unimplemented command for indirect family 7: {:#01X}", value)
        }
    }
    
    fn reverse(&mut self){
        // Swap A and B registers
        self.stack.swap();
    }
    
    // Arithmetic
    fn add(&mut self){
        let a = self.stack.a();
        let b = self.stack.b();
        if let Some(result) = a.checked_add(b){
            self.stack.set(0, result);
        }
        else{
            self.stack.set(0, a.wrapping_add(b));
            self.set_error();
        }
        self.stack.set(1, self.stack.c());
    }
     
    fn bcnt(&mut self){
        // Returns four times A into A
        // Helpful for word offsets
        // B and C are unaffected
        self.stack.set(0, self.stack.a() << 2);
    }
    
    /// Bit count 0x27 0xF6
    /// Counts the number of bits in A and adds to B
    /// Loads contents of C into B
    fn bitcnt(&mut self){
        // Counts the number of bits in A and 
        // then adds that value to B
        let mut sum = 0;
        let a = self.stack.a();
        for i in 0..32{
            if 1 << i & a > 0{
                sum += 1;
            }
        }
        self.stack.set(0, self.stack.b() + sum);
        self.stack.set(1, self.stack.c());
    }
     
    /// BITREVNBITS 0x27 0xF8
    /// Reverse bottom n bits in word
    fn bit_rev_n_bits(&mut self){
        let n_bits = self.stack.a();
        let mut b = self.stack.b();
        
        // mask lower n bits
        b = b & !((1 << n_bits) - 1);
        
        let b_source = self.stack.b();
        
        for i in 0..n_bits{
            let v = b_source & (1 << i);
            if v > 0{
                b |= 1 << (n_bits - 1 - i);
            }
        }
        self.stack.set(0, b);
        self.stack.set(1, self.stack.c());
    }
     
    /// BITREVWORD 0x27 0xF7
    /// Reverse bits in word
    fn bit_reverse_word(&mut self){
        let a = self.stack.a();
        let mut result = 0;
        for i in 0..32{
            let v = a & (1 << i);
            if v > 0{
                result |= 1 << (31 - i);
            }
        }
        self.stack.set(0, result);
    }
     
    /// BSUB 0xF2
    /// Byte subscript assumes that A is the base address of an array
    /// and B is a byte index into the array. It adds A and B leaving the result in A.
    /// C is popped into B
    fn byte_subscript(&mut self){
        self.stack.set(0, self.stack.a().wrapping_add(self.stack.b()));
        self.stack.set(1, self.stack.c());
    }
     
    /// CCNT1 0x24 0xFD
    /// Check count from One
    /// Error is set if count is not from one (i.e., B is zero) or out of bounds (i.e., B is greater than A).
     fn ccnt1(&mut self){
        if self.stack.b() == 0{
            self.set_error();
        }
        if self.stack.b() as u32 > self.stack.a() as u32{
            self.set_error();
        }
        self.stack.set(0, self.stack.b());
        self.stack.set(1, self.stack.c());
    }
    
    /// CSNGL 0x24 0xFC
    /// Error is set if the value will not fit into a single word
    fn check_single(&mut self){
        // This is essentially checking if the 64 bit value in AB can fit into one word
        let a = self.stack.a();
        let b = self.stack.b();
        println!("Check single with values A: {}, B: {}", a, b);
        if a < 0 && b != -1{
            // Anything  but a small negative number
            self.set_error();
        }
        if a >= 0 && b != 0{
            // small positive number
            self.set_error();
        }
        let mut ab = self.stack.a();
        if b == -1{
            // This might be a complement
            ab = -1 * self.stack.a();
        }
        // set to the combined value and move c up
        self.stack.set(0, ab);
        self.stack.set(1, self.stack.c());
    }
    
    /// CSUB0 0x21 0xF3
    /// Error is set if B is greater or equal to A, otherwise it remains upchanged
    fn check_subscript_from_zero(&mut self){
        let a: u32 = self.stack.a() as u32;
        let b: u32 = self.stack.b() as u32;
        
        if b >= a{
            self.set_error();
        }
        
        // A is popped out of the stack
        self.stack.set(0, self.stack.b());
        self.stack.set(1, self.stack.c());
    }
    
    /// CWORD check word 0x25 0xF6
    /// Error set if the value will not fit into a specified partword size
    fn check_word(&mut self){
        let a = self.stack.a();
        let b = self.stack.b();
        
        if b >= a || b < -1*a{
            self.set_error();
        }
        
        self.stack.set(0, self.stack.b());
        self.stack.set(1, self.stack.c());
    }
    
    /// CFLERR Check single FP Inf or NaN T313
       
    fn logical_and(&mut self){
        // A equals the bitwise AND of A and B
        self.stack.set(0, self.stack.a() & self.stack.b());
        self.stack.set(1, self.stack.c());
    }
    
    // Control operation
    fn ret(&mut self){
        // Returns the execution flow from a subroutine back to the calling thread of execution
        
        // Load program counter and register space from stack
        self.pc = self.mem.read(self.workspace);
        self.stack.set(0, self.mem.read(self.workspace + 4));
        self.stack.set(1, self.mem.read(self.workspace + 8));
        self.stack.set(2, self.mem.read(self.workspace + 12));
        
        // reset stack
        self.workspace = self.workspace + 12;
    }
    
    fn ldpi(&mut self){
        // LDPI Load pointer to instruction adds the current value of the instruction pointer to A
        self.stack.set(0, self.stack.a() + self.pc);
    }
    
    fn gajw(&mut self){
        // GAJW General adjust workspace exchanges the contents of the workspace pointer and A. A should be word-aligned.
        let wp = self.workspace;
        self.workspace = self.stack.a();
        if self.workspace % 4 > 0{
            dbg!("Warning: GAJW set workspace to {}, which is not word-aligned", self.workspace);
        }
        self.stack.set(0, wp);
    }
    
    fn dup(&mut self){
        // DUP Duplicate top of stack duplicates the contents of A into B.
        self.stack.set(1, self.stack.a());
        // TODO warning or error
        // The DUP instruction is available on the T800 and not the T414
    }
    
    fn gcall(&mut self){
        // GCALL
        // General call exchanges the contents of the instruction pointer and A
        // Execution then continues at the new address formerly contained in A
        // This can be used to generate a subroutine call at run time by:
        //  1. Build a stack frame like CALL
        //  2. Load A with the address of the subroutine
        //  3. Execute GCALL
        let pc = self.pc;
        self.pc = self.stack.a();
        self.stack.set(0, pc);
    }
    
    /// CLRHALTERR Clear HaltOnError flag 0x25 0xF7
    fn clear_halt_error(&mut self){
        self.flag.halt_on_error = false;
    }
    
    /// CRC on topmost byte of A 0x27 0xF5
    fn crc_byte(&mut self){
        // don't have a great idea of how to implement CRC rn
        todo!("CRC byte not implemented");
    }
    
    /// CRCWORD calculate CRC on word T800 0x25 0xF4
    fn crc_word(&mut self){
        todo!("CRC word not implemented")
    }
    
    // Alt mode managements       
    fn alt(&mut self){
        todo!("Alt command not implemented");
    }

    fn alt_wait(&mut self){
        todo!("Alt wait not implemented");
    }
            
    fn alt_end(&mut self){
        todo!("Alt command not implemented");
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
            Flag::ERROR => self.flag.error
        }
    }
    
    pub fn clear(&mut self){
        self.flag.error = false;
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
     
    pub fn reset(&mut self, workspace: i32){
        self.pc = 0;
        self.workspace = workspace;
        self.stack.set(0, 0);
        self.stack.set(1, 0);
        self.stack.set(2, 0);
    }
     
    /********* run instruction *****************/ 
    pub fn run(&mut self, op: DirectOp, value: RTYPE) -> Result<(), Error>{
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
        };
        Ok(())
    }
    
    pub fn state(&self) -> ProcState{
        self.state
    }
}