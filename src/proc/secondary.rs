/* Define secondary functions */

use std::ops::Add;

use super::{NotProcess_p, OpErr, OpVal, Priority, ProcLibrary};
use crate::{mem::*, proc::GotoSNPBit};

/// Instructions encoded without using prefix
pub fn define_wo_prefix(pl: &mut ProcLibrary){
    // Reverse top of stack
    pl.define_indirect("rev",0x0, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        p.stack.push(a);
        p.stack.push(b);
        Ok(OpVal::Null)
    });
    
    // Load byte
    pl.define_indirect("lb", 0x1, |p|{
        let a = p.stack.pop();
        let v = p.mem.read_byte(a);
        Ok(OpVal::Int(v as i32))
    });
    
    // Bsub byte subscript
    pl.define_indirect("bsub", 0x2, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(a.wrapping_add(b)))
    });
    
    // End process
    pl.define_indirect("endp", 0x3, |p|{
        // TODO check documentation for endp
        let a = p.stack.pop();
        let flag = p.mem.read(a + 4);
        if flag == 1{
            p.pc = p.mem.read(a);
            p.workspace = a;
        }
        else{
            p.mem.write(a+4, flag-1);
            p.status = p.status | GotoSNPBit;
        }
        Ok(OpVal::Null)
    });
    
    // 0x4 Difference
    
    // 0x5 Add
    pl.define_indirect("add", 0x5, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        
        match a.checked_add(b){
            Some(v) => p.stack.push(v),
            None => p.throw_error(OpErr::Overflow)
        }
        Ok(OpVal::Null)
    });
    
    // 0x6 General call
    pl.define_indirect("gcall", 0x6, |p|{
        let a = p.stack.pop();
        let t = p.pc;
        p.pc = a;
        Ok(OpVal::Int(t))
    });
    
    // 0x7 Input message
    
    // 0x8 Product
    pl.define_indirect("prod", 0x8, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(a * b))
    });
    
    // 0x9 Greather than
    pl.define_indirect("gt", 0x9, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        if b > a{
            Ok(OpVal::Int(1))
        }
        else{
            Ok(OpVal::Int(0))
        }
    });
    
    // 0xA Word subscript
    // Line 1344 of nanochess transputer_emulator.js transputer has a order of operatations 
    pl.define_indirect("wsub", 0xA, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop() << 2;
        Ok(OpVal::Int(a + b))
    });
    
    // 0xB Output message
    
    // 0xD Start process
    // startp
    pl.define_indirect("startp", 0x0D, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        // Start new process at a
        p.mem.write(a - 4, p.pc + b);
        
        // Run with same priority
        p.run_process(a | (p.descriptor & 0b1));
        Ok(OpVal::Null)
    });
    
    // 0xF Output word
    
    // 0xE Output byte
    
    // 0xC Subtract
    pl.define_indirect("sub", 0xC, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(b - a))
    });
}

pub fn define_w_prefix(pl: &mut ProcLibrary){
    // tin
    pl.define_indirect("tin", 0x2B, |p|{
        let temp =match p.priority(){
            Priority::Low => p.mem.read(CLOCK_REG_0),
            Priority::High => p.mem.read(CLOCK_REG_1)
        };
        let a = p.stack.pop();
        if a - temp > 0{
            // Should wait
            // Save workspace state
            p.mem.write(p.workspace - 4, p.pc);
            p.mem.write(p.workspace - 20, a); // Time for awakening
            // Get addresses of timers
            todo!("Link timer");
            p.status = p.status | GotoSNPBit;
        }
        Ok(OpVal::Null)
    });
    
    // stop process
    pl.define_indirect("stopp", 0x15, |p|{
        p.mem.write(p.workspace - 4, p.pc);
        p.status = p.status | GotoSNPBit;
        Ok(OpVal::Null)
    });
    
    // Run p
    pl.define_indirect("runp", 0x39, |p|{
        let a = p.stack.pop();
        p.run_process(a);
        Ok(OpVal::Null)
    });
    
    // Save Low priority info
    pl.define_indirect("savel", 0x3D, |p|{
        let a = p.stack.pop();
        p.mem.write(a, p.get_front_pointer(Priority::Low));
        p.mem.write(a+4, p.get_back_pointer(Priority::Low));
        Ok(OpVal::Null)
    });
    
    // Save high priority info
    pl.define_indirect("saveh",0x3E, |p|{
        let a = p.stack.pop();
        p.mem.write(a, p.get_front_pointer(Priority::High));
        p.mem.write(a+4, p.get_back_pointer(Priority::High));
        Ok(OpVal::Null)
    });
    
    // Set front and back process pointers
    // STHB
    pl.define_indirect("sthb", 0x50, |p|{
        let a = p.stack.pop();
        p.set_back_pointer(Priority::High, a);
        Ok(OpVal::Null)
    });
    
    // Sthl
    pl.define_indirect("stlb", 0x17, |p|{
        let a = p.stack.pop();
        p.set_back_pointer(Priority::Low, a);
        Ok(OpVal::Null)
    });
    
    // STHB
    pl.define_indirect("sthf", 0x18, |p|{
        let a = p.stack.pop();
        p.set_front_pointer(Priority::High, a);
        Ok(OpVal::Null)
    });
    
    // Sthl
    pl.define_indirect("stlf", 0x1C, |p|{
        let a = p.stack.pop();
        p.set_front_pointer(Priority::Low, a);
        Ok(OpVal::Null)
    });
    
    // Reset channel
    pl.define_indirect("resetch", 0x12, |p|{
        let a = p.stack.pop();
        let v = p.mem.read(a);
        p.mem.write(a, NotProcess_p);
        Ok(OpVal::Int(v))
    });
    
    // Set error
    pl.define_indirect("seterr", 0x10, |p|{
        p.error = 1;
        Ok(OpVal::Null)
    });
    
    // Csub0
    pl.define_indirect("csub0",0x13, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        if b >= a{
            p.error = 1;
        }
        Ok(OpVal::Int(b))
    });
    
    // ladd
    pl.define_indirect("ladd", 0x16, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        let c = p.stack.pop();
        match b.checked_add(a + (c & 0b1)){
            Some(v) => Ok(OpVal::Int(v)),
            None => Err(OpErr::Overflow)   
        }
    });
    
    // norm
    pl.define_indirect("norm",0x19, |p|{
        let mut c = 0;
        let mut a = p.stack.pop();
        let mut b = p.stack.pop();
        while c < 64 && ((b & 0x8000_0000u32 as i32) == 0){
            b = (b << 1) | (a >> 31);
            a = a << 1;
            c += 1;
        }
        Ok(OpVal::List(vec![a, b]))
    });
    
    // ldpi
    pl.define_indirect("ldpi", 0x1B, |p|{
        todo!("LDPI")
    });
}