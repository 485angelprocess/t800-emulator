use std::io::Error;

use crate::mem::{Mem, Stack, STACK_SIZE};

mod workspace;

use workspace::{EventState, ProcPriority, WorkspaceCache, MOST_NEG, NOT_PROCESS_P};

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

pub struct ProcFlag{
    go_to_snp: bool,
    io: bool,
    move_bit: bool,
    time_del: bool,
    time_ins: bool,
    dist_and_ins: bool,
    error : bool,
    halt_on_error: bool
}

impl Default for ProcFlag{
    fn default() -> Self {
        Self{
            go_to_snp: false,
            io: false,
            move_bit: false,
            time_del: false,
            time_ins: false,
            dist_and_ins: false,
            error: false,
            halt_on_error: true
        }
    }
}

impl ProcFlag{
    fn get_state(&self) -> ProcState{
        if self.halt_on_error & self.error{
            return ProcState::HALTED;
        }
        if self.go_to_snp{
            return ProcState::IDLE;
        }
        return ProcState::ACTIVE;
    }
    
    fn deschedule(&mut self){
        self.go_to_snp = true;
    }
    
    fn set_error(&mut self){
        self.error = true;
    }
    
    fn clear(&mut self){
        self.go_to_snp = false;
        self.error = false;
        self.io = false;
        self.move_bit = false;
        self.time_del = false;
        self.time_ins = false;
        self.dist_and_ins = false;
        self.halt_on_error = true;
    }
}

fn mask4(v: RTYPE) -> RTYPE{
    (v << 4) >> 4
}

// TODO move to stat
#[derive(Clone, Copy, PartialEq)]
pub enum ProcState{
    ACTIVE,
    IDLE,
    HALTED
}

struct Params{
    pub fptr: [RTYPE; 2],
    pub bptr: [RTYPE; 2],
    pub tptr: [RTYPE; 2],
    pub timeslice: i32,
    pub interrupt: bool,
    
}

impl Params{
    pub fn new() -> Self{
        Params{
            fptr: [NOT_PROCESS_P; 2],
            bptr: [NOT_PROCESS_P; 2],
            tptr: [NOT_PROCESS_P; 2],
            timeslice: 0,
            interrupt: false,
        }
    }
}

pub struct Proc{
    stack: Stack,
    
    // Main registers
    pc: ATYPE,
    workspace: ATYPE,
    operand:  RTYPE,
    
    // Internal variables
    priority: RTYPE,
    
    // Additional registers
    params: Params,
    
    flag: ProcFlag,
    cache: WorkspaceCache,
    mem: Mem
}

impl Proc{
    pub fn new(m: Mem) -> Self{
        Self{
            stack: Stack::new(),
            pc: 0,
            workspace: 0,
            operand: 0,
            priority: ProcPriority::LOW,
            params: Params::new(),
            flag: ProcFlag::default(),
            cache: WorkspaceCache::new(m.clone()),
            mem: m
        }
    }
    
    /// Set process to idle
    fn deschedule(&mut self){
        todo!("Deschedule action not implemented")
    }
    
    fn start_process(&mut self){
        todo!("Start process not implemented");
    }
    
    /// Handle host link communication
    fn host_link_handle(&mut self){
        todo!("Host link communication not handleed");
    }
    
    /// Checks when current process needs descheduling
    /// Runs from 'j' and 'lend'
    fn check_deschedule(&mut self){
        self.host_link_handle();
        
        if self.params.timeslice > 1{
            // Must change process
            self.params.timeslice = 0;
            
            self.deschedule();
            
            self.start_process();
        }
    }
    
    fn interrupt(&mut self){
       
       // Sanity check, cannot already be doing an interrupt
       if self.params.interrupt{
           panic!("Error multiple interrupts of low priority processes :(");
       }
       
       // Store my registers
       // This is interrupt table locations,
       // May want to rewrite this to be more generic
       self.mem.write(MOST_NEG + (11 << 2), self.workspace | self.priority); // todo: check priority values
       self.mem.write(MOST_NEG + (12 << 2), self.pc);
       self.mem.write(MOST_NEG + (13 << 2), self.stack.a());
       self.mem.write(MOST_NEG + (14 << 2), self.stack.b());
       self.mem.write(MOST_NEG + (15 << 2), self.stack.c());
       // In emulator, the status and error reg are commented out
       // so leaving those out
    }
    
    fn clear_timer(&mut self){
        
        // TODO write up so it's more clearly referencing timing link
        let mut ptr = match self.priority{
            ProcPriority::HIGH => {
                while self.params.tptr[0] == self.workspace{
                    // Time value reached flag
                    self.params.tptr[0] = self.mem.read(self.workspace - 16);
                }
                
                self.params.tptr[0]
            }
            ProcPriority::LOW => {
                while self.params.tptr[1] == self.workspace{
                    self.params.tptr[1] = self.mem.read(self.workspace - 16);
                }
                
                self.params.tptr[1]
            }
            _ => panic!("Invalid process priority")
        };
        
        let mut last_ptr = ptr;
        while ptr != NOT_PROCESS_P{
            if ptr == self.workspace{
                ptr = self.mem.read(ptr - 16);
                self.mem.write(last_ptr - 16, ptr);
            }
            else{
                last_ptr = ptr;
                ptr = self.mem.read(ptr - 16);
            }
        }
    }
    
    /// Schedule new process
    /// Add a process to the relevant priority queue
    fn schedule(&mut self, wp: RTYPE, priority: RTYPE){
        // Remove from timer queue if alt
        let state = self.cache.get_state(wp);
        if state == EventState::READY{
            self.clear_timer();
        }
        
        // If a high priority process is being scheduled,
        // while a low priority process runs, interrupt
        if priority == ProcPriority::HIGH && self.priority == ProcPriority::LOW{
            self.interrupt();
            
            // Load new process
            self.priority = ProcPriority::HIGH;
            self.workspace = wp; // update workspace
            self.pc = self.cache.get_iptr(self.workspace); // get program counter
        }
        else{
            // Do not need to interrupt
            
            // Get front of process list pointer
            let ptr = match priority{
                ProcPriority::HIGH => self.params.fptr[0],
                ProcPriority::LOW  => self.params.fptr[1],
                _ => panic!("Invalid priority")
            };
            
            if ptr == NOT_PROCESS_P{
                // Empty process list, create
                match priority{
                    ProcPriority::HIGH => {
                        self.params.fptr[0] = wp;
                        self.params.bptr[0] = wp;
                    },
                    ProcPriority::LOW => {
                        self.params.fptr[1] = wp;
                        self.params.bptr[1] = wp;
                    },
                    _ => panic!("Invalid priority")
                };
            }
            else{
                // Process list already exists
                
                // Get workspace pointer of last process in list
                let last_ptr = match priority{
                    ProcPriority::HIGH => self.params.bptr[0],
                    ProcPriority::LOW => self.params.bptr[1],
                    _ => panic!("Invalid priority")
                };
                
                // link new process onto end of list
                self.cache.set_link(last_ptr, wp);
                
                // Update end of process list pointer
                match priority {
                    ProcPriority::HIGH => self.params.bptr[0] = wp,
                    ProcPriority::LOW  => self.params.bptr[1] = wp,
                    _ => panic!("Invalid priority")   
                }
            }
        }
    }
    
    /// Set error flag, halts if HaltOnError set
    fn set_error(&mut self){
        self.flag.set_error();
    }
    
    /*********Direct sInstructions ***********/
    /// Load constant
    fn ldc(&mut self, value: RTYPE){
        // LDC instruction
        self.operand = mask4(self.operand) + value;
        self.stack.push(self.operand);
        self.operand = 0;
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
            0x4 => self.diff(),
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
            0xC => self.div(),
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
            0xA => self.dup(),
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
    
    /// DIFF 0xF4
    /// Gets the difference of B - A, and pops C into B
    fn diff(&mut self){
        self.stack.set(0, self.stack.b() - self.stack.a());
        self.stack.set(1, self.stack.c());
    }
    
    /// DIV 0x22 0xFC
    /// Divide, error is set on division by 0
    fn div(&mut self){
        if self.stack.a() == 0{
            // divide by 0
            self.set_error();
            return;
        }
        if self.stack.a() == -1 && self.stack.b() == i32::MIN{
            // This is an overflow case
            self.set_error();
            return;
        }
        self.stack.set(0, self.stack.b() / self.stack.a());
        self.stack.set(1, self.stack.c());
    }
    
    /// DUP 0x25 0xFA
    /// Duplicates top of stack
    fn dup(&mut self){
        self.stack.set(2, self.stack.b());
        self.stack.set(1, self.stack.a());
        
        // Throw warning if trying to emulate the T414
    }
    
    /// ENBC Enable channel 0x24 0xF8
    /// Enables a channel pointed to by B, only if Ais one
    /// a) no process on channel B, store the current process workspace
    ///     into the channel to start communication
    /// b) the current process is waiting on channel B, do nothing
    /// c) another process is waiting, ready flag is stored at workspace -3, C is popped into B
    fn enbc(&mut self){
        if self.stack.a() != 0{ // unclear what logical TRUE is
            todo!("Enable channel not implemented");
        }
    }
    
    /// CFLERR Check single FP Inf or NaN T313
    fn logical_and(&mut self){
        // A equals the bitwise AND of A and B
        self.stack.set(0, self.stack.a() & self.stack.b());
        self.stack.set(1, self.stack.c());
    }
    
    /// RET
    /// Control operation
    /// Returns the execution flow from a subroutine back to the calling thread of execution
    fn ret(&mut self){
        // Load program counter and register space from stack
        self.pc = self.mem.read(self.workspace);
        self.stack.set(0, self.mem.read(self.workspace + 4));
        self.stack.set(1, self.mem.read(self.workspace + 8));
        self.stack.set(2, self.mem.read(self.workspace + 12));
        
        // reset stack
        self.workspace = self.workspace + 12;
    }
    
    /// LDPI Load pointer to instruction adds the current value of the instruction pointer to A
    fn ldpi(&mut self){
        
        self.stack.set(0, self.stack.a() + self.pc);
    }
    
    /// GAJW General adjust workspace exchanges the contents of the workspace pointer and A. A should be word-aligned.
    fn gajw(&mut self){
        let wp = self.workspace;
        self.workspace = self.stack.a();
        if self.workspace % 4 > 0{
            println!("Warning: GAJW set workspace to {}, which is not word-aligned", self.workspace);
        }
        self.stack.set(0, wp);
    }
    
    /// GCALL
    /// General call exchanges the contents of the instruction pointer and A
    /// Execution then continues at the new address formerly contained in A
    /// This can be used to generate a subroutine call at run time by:
    ///  1. Build a stack frame like CALL
    ///  2. Load A with the address of the subroutine
    ///  3. Execute GCALL
    fn gcall(&mut self){
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
    
    // Concurrent scheduling
    /// Startp
    /// Starts a new concurrent process
    fn startp(&mut self){
        let temp = self.stack.a();// & 0xfffffffe;
        self.mem.write(temp - 4, self.pc + self.stack.b());
        self.schedule(temp, self.priority);
    }
    
    // Alt mode managements   
    /// ALT 0x24 0xF3
    /// Stores the flag Enabling.p in workspace location -3 (State.s)
    /// Shows that the enabling of a ALT construct is occurring    
    fn alt(&mut self){
        self.cache.set_state(self.workspace, EventState::ENABLING);
    }
             
    /// ALTEND 0x24 0xF5
    /// Alt end is executed after the process containing ALT has been
    /// rescheduled. Workspace location zero contains the offset
    /// from the instruction pointer to the guard routine to execute
    /// This offset is added to the instruction pointer
    /// and execution continues at the appropriate guard's service routine
    fn alt_end(&mut self){
        // The instruction counter is already at the location of the next instruction
        self.pc = self.pc + self.cache.get_guard_offset(self.workspace);
    }
    
    /// ALTWT 0x24 0xF4
    /// Store -1 in workspace location 0, and waits until State.s is ready
    /// Process is descheduled until one of the guards is ready
    fn alt_wait(&mut self){
        // From documentation:
        // DisableStatus is defined to be Wptr[0]
        self.mem.write(self.workspace, -1); // DisableStatus is set to -1
        
        if self.cache.get_state(self.workspace) != EventState::READY{
            // Have to go back 2 because this is a operate instruction
            self.pc -= 2; // loop (kinda annoying way to do this)
        }
    }
    
    /********** Debug methods ***********/
    pub fn get_reg(&self, index: usize) -> RTYPE{
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
    
    pub fn set_reg(&mut self, index: usize, value: RTYPE){
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
            DirectOp::PFIX => self.prefix(value), // Push 4 bits onto operand register
            DirectOp::NFIX => self.neg_prefix(value), // push 4 bits onto operand and complement
            DirectOp::AJW  => self.ajw(value), // adjust workspace
            DirectOp::JUMP => self.jump(value), // jump (deschedule)
            DirectOp::CJ   => self.cj(value), // conditional jump
            DirectOp::CALL => self.call(value), // call subroutine
            DirectOp::EQC  => self.eqc(value), // equal check
            DirectOp::OPR  => self.operate(value), // indirect operations
        };
        Ok(())
    }
    
    pub fn state(&self) -> ProcState{
        self.flag.get_state()
    }
}