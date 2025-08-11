/* Define secondary functions */

use super::{NOT_PROCESS_P, OpErr, OpVal, Priority, ProcLibrary};
use crate::{mem::*, proc::GO_TO_SNP_BIT};

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
            p.status = p.status | GO_TO_SNP_BIT;
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
            p.status = p.status | GO_TO_SNP_BIT;
        }
        Ok(OpVal::Null)
    });
    
    // stop process
    pl.define_indirect("stopp", 0x15, |p|{
        p.mem.write(p.workspace - 4, p.pc);
        p.status = p.status | GO_TO_SNP_BIT;
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
        p.mem.write(a, NOT_PROCESS_P);
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
    // normalizes the double value of A
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
        let _a = p.stack.set(0, p.stack.a() + p.pc);
        Ok(OpVal::Null)
    });
    
    // xdble
    pl.define_indirect("xdble", 0x1D, |p|{
        let a = p.stack.pop();
        Ok(OpVal::List(vec![a >> 31, a]))
    });
    
    // ldri - load priority
    pl.define_indirect("ldpri", 0x1E, |p|{
        Ok(OpVal::Int(match p.priority(){
            Priority::Low => 1,
            Priority::High => 0
        }))
    });
    
    // remainde
    pl.define_indirect("rem", 0x1F, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        
        if a == 0 || a == -0{
            return Err(OpErr::DivideByZero);
        }
        
        Ok(OpVal::Int(b % a))
    });
    
    // return
    pl.define_indirect("ret", 0x20, |p|{
        p.pc = p.mem.read(p.workspace);
        p.workspace += 16;
        p.update_wdesc(p.workspace | p.priority() as i32);
        Ok(OpVal::Null)
    });
    
    // lend
    // Loop end
    pl.define_indirect("lend", 0x21, |p|{
        let a = p.stack.a();
        let b = p.stack.b() & (!0b11);
        
        // Decrement by 1
        let wp = p.mem.read(b+4) - 1;
        p.mem.write(b+4, wp);
        if wp == 0{
            return Ok(OpVal::Null)
        }
        
        p.mem.write(b, wp);
        p.pc = p.pc - a;
        if p.priority() == Priority::Low{
            // Deschedule
            p.mem.write(p.workspace - 4, p.pc);
            if p.get_front_pointer(Priority::Low) == NOT_PROCESS_P{
                p.set_front_pointer(Priority::High, p.workspace);
            }
            else{
                p.mem.write(p.get_back_pointer(Priority::Low) - 8, p.workspace);
            }
            p.set_back_pointer(Priority::Low, p.workspace);
            p.status |= GO_TO_SNP_BIT;
        }
        Ok(OpVal::Null)
    });
    
    // Load timer
    pl.define_indirect("ldtimer", 0x22, |p|{
        Ok(OpVal::Int(p.get_clock_register(p.priority())))
    });
    
    // Test err
    pl.define_indirect("testerr", 0x29, |p|{
        let err = p.error;
        p.error = 0;
        Ok(OpVal::Int(err as i32))
    });
    
    // Test analysis pin
    pl.define_indirect("testpranal", 0x2A, |_p|{
        println!("Analysis pin non existent");
        Ok(OpVal::Int(0))
    });
    
    // Division
    pl.define_indirect("div", 0x2C, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        if a == 0{
            return Err(OpErr::DivideByZero)
        }
        Ok(OpVal::Int(b / a))
    });
    
    // Dist
    pl.define_indirect("dist", 0x2E, |_p|{
        todo!("Dist not implemented");
    });
    
    pl.define_indirect("diss", 0x30, |_p|{
        todo!("Diss not implemented");
    });
    
    // bitwise not
    pl.define_indirect("not", 0x32, |p|{
        let a = p.stack.pop();
        Ok(OpVal::Int(!a))
    });
    
    // xor
    pl.define_indirect("xor",0x33, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(b ^ a))
    });
    
    // Bit count?
    pl.define_indirect("bcnt", 0x34, |p|{
        let a = p.stack.pop();
        Ok(OpVal::Int(a << 2))
    });
    
    // long subtraction
    pl.define_indirect("lsub", 0x38, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        let c = p.stack.pop();
        let result = b.checked_sub(a - (c & 0b1));
        match result{
            Some(v) => Ok(OpVal::Int(v)),
            None => Err(OpErr::Overflow)
        }
    });
    
    // xword
    pl.define_indirect("xword", 0x3A, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        if b < a{
            Ok(OpVal::Int(b))
        }
        else{
            Ok(OpVal::Int((b - a) << 1))
        }
    });
    
    // Store byte
    pl.define_indirect("sb", 0x3B, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop() & 0xFF;
        p.mem.write_byte(a, b as u8);
        Ok(OpVal::Null)
    });
    
    // gajw
    pl.define_indirect("gajw", 0x3C, |p|{
        p.workspace = p.stack.a() & (!0b11);
        p.update_wdesc(p.workspace | p.priority() as i32);
        Ok(OpVal::Null)
    });
    
    // wcnt
    pl.define_indirect("wcnt", 0x3F, |p|{
        let a = p.stack.pop();
        Ok(OpVal::List(vec![a & 0b11, a >> 2]))
    });
    
    // shr
    pl.define_indirect("shr", 0x40, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(b >> a))
    });
    
    // shl
    pl.define_indirect("shl", 0x40, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(a << b))
    });
    
    // minimum integer
    pl.define_indirect("mint", 0x42, |_p|{
        Ok(OpVal::Int(0x8000_0000u32 as i32))
    });
    
    // alt commands are not implemented for now
    pl.define_indirect("alt", 0x42, |_p|{
        todo!("alt command not implemented");
    });
    
    pl.define_indirect("altwt", 0x44, |_p|{
        todo!("altwt command not implemented");
    });
    
    pl.define_indirect("altend", 0x45, |_p|{
        todo!("altend command not implemented");
    });
    
    pl.define_indirect("and", 0x46, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(a & b))
    });
    
    pl.define_indirect("enbt", 0x47, |_p|{
        todo!("enbt not implemented");
    });
    
    pl.define_indirect("enbc", 0x48, |_p|{
        todo!("enbc not implemented");
    });
    
    pl.define_indirect("enbs", 0x48, |_p|{
        todo!("enbs not implemented");
    });
    
    // Move array in memory
    pl.define_indirect("move", 0x4A, |p|{
        // Move n bytes from one location to the next
        let mut a = p.stack.pop();
        let mut b = p.stack.pop();
        let mut c = p.stack.pop();
        while a > 0{
            p.mem.write_byte(b, p.mem.read_byte(c));
            c += 1;
            b += 1;
            a -= 1;
        }
        Ok(OpVal::Null)
    });
    
    // Or arithmetic
    pl.define_indirect("or", 0x4B, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(a | b))
    });
    
    // csngl
    // Check if double value can be reduced to single
    pl.define_indirect("csngl", 0x4C, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        if a < 0 && b != -1{
            return Err(OpErr::NotSingle);
        }
        if a >= 0 && b != 0{
            return Err(OpErr::NotSingle);
        }
        Ok(OpVal::Int(a))
    });
    
    // ccnt1
    pl.define_indirect("ccnt1", 0x4D, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        
        if b == 0 || b > a{
            return Err(OpErr::Count);
        }
        return Ok(OpVal::Int(b))
    });
    
    // talt
    pl.define_indirect("talt", 0x4E, |_p|{
        todo!("talt not defined");
    });
    
    // taltwt
    pl.define_indirect("taltwt", 0x51, |_p|{
        todo!("taltwt not defined");
    });
    
    // sum
    pl.define_indirect("sum", 0x52, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(a + b))
    });
    
    // mul
    pl.define_indirect("mul", 0x53, |p|{
        let a = p.stack.pop();
        let b = p.stack.pop();
        Ok(OpVal::Int(a * b))
    });
    
    // dup
    pl.define_indirect("dup", 0x5A, |p|{
        Ok(OpVal::Int(p.stack.a()))
    });
}