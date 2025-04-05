#[cfg(test)]
mod direct{
    use crate::{mem::Mem, proc::{DirectOp, ProcState, Flag, Proc}};
    
    #[test]
    fn load_constant(){
        let mut p = Proc::new(Mem::new());
        assert!(p.peek(0) == 0);
        p.run(DirectOp::LDC, 5);
        assert!(p.peek(0) == 5);
        assert!(p.peek(1) == 0);
        assert!(p.peek(2) == 0);
        p.run(DirectOp::LDC, 10);
        assert!(p.peek(0) == 10);
        assert!(p.peek(1) == 5);
        assert!(p.peek(2) == 0);
    }
    
    #[test]
    fn add_constant(){
        let mut p = Proc::new(Mem::new());
        
        // standard addition
        assert!(p.peek(0) == 0);
        p.poke(0, 5);
        p.run(DirectOp::LDC, 6);
        p.run(DirectOp::ADC, 3);
        assert!(p.peek(0) == 9);
        assert!(p.peek(1) == 5);
        
        // Overflow
        p.poke(0, 2_147_483_647);
        p.run(DirectOp::ADC, 3);
        assert!(p.flag(Flag::ERROR));
    }

    #[test]
    fn load_local(){
        let mut m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        p.set_workspace_pointer(4);
        
        m.write(4, 9);
        m.write(8, 11);
        m.write(12, 13);
        
        p.run(DirectOp::LDL, 0);
        assert!(p.peek(0) == 9);
        
        p.run(DirectOp::LDL, 1);
        assert!(p.peek(0) == 11);
        assert!(p.peek(1) == 9);
        
        p.run(DirectOp::LDL, 2);
        assert!(p.peek(0) == 13);
        assert!(p.peek(1) == 11);
        assert!(p.peek(2) == 9);
    }
    
    #[test]
    fn store_local(){
        let mut m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        m.write(8, 10);
        m.write(12, 11);
        m.write(16, 13);
        
        p.set_workspace_pointer(8);
        
        p.poke(0, 15);
        p.run(DirectOp::STL, 0);
        assert!(m.read(8) == 15);
        
        p.run(DirectOp::STL, 1);
        assert!(m.read(12) == 15);
    }
    
    #[test]
    fn load_local_pointer(){
        let m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        p.set_workspace_pointer(1000);
        p.run(DirectOp::LDLP, 0);
        assert!(p.peek(0) == 1000);
        
        p.run(DirectOp::LDLP, 1);
        assert!(p.peek(0) == 1004);
        assert!(p.peek(1) == 1000);
    }

    #[test]
    fn load_non_local(){
        let mut m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        p.poke(0, 0x10004000);
        p.poke(1, 20);
        m.write(0x10004000, 5);
        m.write(0x10004004, 7);
     
        p.run(DirectOp::LDNL, 0);
        assert!(p.peek(0) == 5);
        assert!(p.peek(1) == 20);
        
        p.poke(0, 0x10004000);
        p.run(DirectOp::LDNL, 1);
        assert!(p.peek(0) == 7);
        assert!(p.peek(1) == 20);
    }
    
    #[test]
    fn store_non_local(){
        let m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        p.poke(0, 0x10004000);
        p.poke(1, 20);
        
        p.run(DirectOp::STNL, 0);
        assert!(p.peek(0) == 0x10004000);
        assert!(m.read(0x10004000) == 20);
        
        p.poke(0, 0x10004000);
        p.poke(1, 15);
        
        p.run(DirectOp::STNL, 1);
        assert!(p.peek(0) == 0x10004000);
        assert!(m.read(0x10004004) == 15);
    }

    #[test]
    fn load_non_local_pointer(){
        let m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        p.poke(0, 8);
        p.poke(1, 10);
        p.run(DirectOp::LDNLP, 1);
        assert!(p.peek(0) == 12);
        assert!(p.peek(1) == 10);
    }

    #[test]
    fn adjust_workspace(){
        let mut p = Proc::new(Mem::new());
        p.set_workspace_pointer(1000);
        assert!(p.workspace_pointer() == 1000);
        p.run(DirectOp::AJW, -2);
        assert!(p.workspace_pointer() == 992);
        p.run(DirectOp::AJW, 2);
        assert!(p.workspace_pointer() == 1000);
    }
    
    #[test]
    fn prefix(){
        let mut p = Proc::new(Mem::new());
        
        p.run(DirectOp::PFIX, 4);
        p.run(DirectOp::PFIX, 3);
        p.run(DirectOp::PFIX, 2);
        p.run(DirectOp::LDC, 1);
        assert!(p.peek(0) == 0x4321);
        
        p.run(DirectOp::NFIX, 1);
        p.run(DirectOp::LDC, 1);
        println!("Value of A is {}", p.peek(0));
        assert!(p.peek(0) == -31);
    }

    #[test]
    fn jump(){
        let mut p = Proc::new(Mem::new());
        p.run(DirectOp::JUMP, 2);
        assert!(p.program_counter() == 3);
        p.run(DirectOp::JUMP, 1);
        assert!(p.program_counter() == 5);
        assert!(p.state() == ProcState::IDLE);
    }
    
    #[test]
    fn conditional(){
        let mut p = Proc::new(Mem::new());
        
        // Run three programs and don't jump
        p.run(DirectOp::LDC, 1);
        p.run(DirectOp::EQC, 1);
        assert!(p.peek(0) == 1);
        p.run(DirectOp::CJ, 4);
        assert!(p.program_counter() == 3);
        
        // Run three instruction and then jump
        p.run(DirectOp::LDC, 2);
        p.run(DirectOp::EQC, 1);
        assert!(p.peek(0) == 0);
        assert!(p.program_counter() == 5);
        p.run(DirectOp::CJ, 4);
        println!("Program counter {}", p.program_counter());
        assert!(p.program_counter() == 10);
    }
}

mod indirect{
    use std::{iter::zip, ops::Deref};

    use crate::{mem::Mem, proc::{DirectOp, ProcState, Flag, Proc}};
    
    #[test]
    fn reverse(){
        let mut p = Proc::new(Mem::new());
        
        p.poke(0, 11);
        p.poke(1, 13);
        // run reverse
        p.run(DirectOp::OPR, 0);
        
        assert!(p.peek(0) == 13);
        assert!(p.peek(1) == 11);
    }
    
    #[test]
    fn add(){
        // A = A + B,
        // If overflow set error
        let mut p = Proc::new(Mem::new());
        
        p.poke(0, 5);
        p.poke(1, 7);
        p.poke(2, 9);
        
        p.run(DirectOp::PFIX, 0);
        p.run(DirectOp::OPR, 5);
        
        assert!(p.peek(0) == 12);
        assert!(p.peek(1) == 9);
        
        assert!(p.flag(Flag::ERROR) == false);
        
        // Overflow
        p.poke(0, 2_147_483_647);
        p.run(DirectOp::PFIX, 0);
        p.run(DirectOp::OPR, 5);
        
        assert!(p.flag(Flag::ERROR) == true);
    }
    
    #[test]
    fn alt(){
        let mut p = Proc::new(Mem::new());
        
        p.run(DirectOp::PFIX, 4);
        p.run(DirectOp::OPR, 3);
        
    }
    
    #[test]
    fn and(){
        // Perform logical and of register a and b
        let mut p = Proc::new(Mem::new());
        
        p.poke(0, 0b011111);
        p.poke(1, 0b101010);
        p.poke(2, 15);
        
        p.run(DirectOp::PFIX, 0x4);
        p.run(DirectOp::OPR, 0xB);
        
        assert!(p.peek(0) == 0b001010);
        // Copies C into B
        assert!(p.peek(1) == 15);
    }
    
    #[test]
    fn byte_count(){
        // Test BCNT
        let mut p = Proc::new(Mem::new());
        
        p.poke(0, 5);
        p.poke(1, 11);
        
        // 0x23 0xF4
        p.run(DirectOp::PFIX, 0x3);
        p.run(DirectOp::OPR, 0x4);
        
        assert!(p.peek(0) == 20);
        assert!(p.peek(1) == 11);
    }
    
    #[test]
    fn bit_counts(){
        // Test BITCNT 
        // 0x27 F6
        let mut p = Proc::new(Mem::new());
        
        p.poke(0, 0b10101);
        p.poke(1, 7);
        p.poke(2, 11);
        
        p.run(DirectOp::PFIX, 0x7);
        p.run(DirectOp::OPR, 0x6);
        
        assert!(p.peek(0) == 10);
        assert!(p.peek(1) == 11);
    }
    
    #[test]
    fn bit_rev_n_bits(){
        // Test bit reverse
        // reverses n bits of B
        let mut p = Proc::new(Mem::new());
        p.poke(0, 4); // number of bits to reverse
        p.poke(1, 0b1010_0101);
        p.poke(2, 13);
        
        p.run(DirectOp::PFIX, 0x7);
        p.run(DirectOp::OPR, 0x8);
        
        assert!(p.peek(0) == 0b1010_1010);
        assert!(p.peek(1) == 13);
    }
    
    #[test]
    fn bit_rev_word(){
        let mut p = Proc::new(Mem::new());
        
        p.poke(0, 0x12345678);
        
        p.run(DirectOp::PFIX, 0x7);
        p.run(DirectOp::OPR, 0x7);
        
        assert!(p.peek(0) == 0x1E6A2C48);
    }
    
    #[test]
    fn bsub(){
        let mut p = Proc::new(Mem::new());
        
        p.poke(0, 13);
        p.poke(1, 15);
        p.poke(2, 17);
        p.run(DirectOp::OPR, 0x2);
        
        assert!(p.peek(0) == 28);
        assert!(p.peek(1) == 17);
    }
    
    #[test]
    fn ccnt1(){
        let mut p = Proc::new(Mem::new());
        
        
        // Case B == 0, sets error
        p.poke(0, 10);
        p.poke(1, 0);
        p.poke(2, 5);
        
        p.run(DirectOp::PFIX, 0x4);
        p.run(DirectOp::OPR, 0xD);
        
        assert!(p.peek(0) == 0);
        assert!(p.peek(1) == 5);
        
        assert!(p.flag(Flag::ERROR) == true);
        p.clear();
        
        // Case B < A and B > 0, error is not set
        p.poke(0, 10);
        p.poke(1, 1);
        p.poke(2, 5);
        
        p.run(DirectOp::PFIX, 0x4);
        p.run(DirectOp::OPR, 0xD);
        
        assert!(p.peek(0) == 1);
        assert!(p.peek(1) == 5);
        
        assert!(p.flag(Flag::ERROR) == false);
        p.clear();
        
        // Case B > A, error is set
        p.poke(0, 10);
        p.poke(1, 100);
        p.poke(2, 5);
        
        p.run(DirectOp::PFIX, 0x4);
        p.run(DirectOp::OPR, 0xD);
        
        assert!(p.peek(0) == 100);
        assert!(p.peek(1) == 5);
        
        assert!(p.flag(Flag::ERROR) == true);
        p.clear();
        
        p.poke(0, -12);
        p.poke(1, 1);
        p.poke(2, 5);
        
        p.run(DirectOp::PFIX, 0x4);
        p.run(DirectOp::OPR, 0xD);
        
        assert!(p.peek(0) == 1);
        assert!(p.peek(1) == 5);
        
        assert!(p.flag(Flag::ERROR) == false);
        p.clear();
    }
    
    #[test]
    fn clrhalterr(){
        let mut p = Proc::new(Mem::new());
        
        // Case B == 0, sets error
        p.poke(0, 10);
        p.poke(1, 0);
        p.poke(2, 5);
        
        p.run(DirectOp::PFIX, 0x4);
        p.run(DirectOp::OPR, 0xD);
        
        assert!(p.peek(0) == 0);
        assert!(p.peek(1) == 5);
        
        assert!(p.state() == ProcState::HALTED);
        p.clear();
        
        // Case B == 0, sets error
        p.poke(0, 10);
        p.poke(1, 0);
        p.poke(2, 5);
        
        p.run(DirectOp::PFIX, 0x2);
        p.run(DirectOp::OPR, 0x7);
        
        p.run(DirectOp::PFIX, 0x4);
        p.run(DirectOp::OPR, 0xD);
        
        assert!(p.peek(0) == 0);
        assert!(p.peek(1) == 5);
        
        assert!(p.state() == ProcState::ACTIVE);
        p.clear();
    }
    
    #[test]
    fn check_single(){
        let ab: Vec<i64> = vec![15, 0x1_FFFF_FFFF, -15, -1 * 0x1_FFFF_FFFF];
        let error: Vec<bool> = vec![false, true, false, true];
        
        let mut p = Proc::new(Mem::new());
        
        for (input, err) in ab.iter().zip(error.iter()){
            let i = input.clone() as u64;
            let a = (i & 0xFFFF_FFFF) as i32;
            let b = ((i >> 32) & 0xFFFF_FFFF) as i32;
            p.poke(0, a);
            p.poke(1, b);
            
            p.run(DirectOp::PFIX, 0x4);
            p.run(DirectOp::OPR, 0xC);
            
            if *err{
                assert!(p.state() == ProcState::HALTED);
            }
            else{
                assert!(p.state() == ProcState::ACTIVE);
            }
            p.clear();
        }
    }
    
    #[test]
    fn check_subscript_from_zero(){
        let a_cases = vec![11, 11, 11];
        let b_cases = vec![8, 11, 15];
        let expect = vec![false, true, true];
        
        let mut p = Proc::new(Mem::new());
        
        for ((a, b), err) in a_cases.iter().zip(b_cases.iter()).zip(expect.iter()){
            p.poke(0, *a);
            p.poke(1, *b);
            
            p.run(DirectOp::PFIX, 0x1);
            p.run(DirectOp::OPR, 0x3);
            
            if *err{
                assert!(p.state() == ProcState::HALTED);
            }
            else{
                assert!(p.state() == ProcState::ACTIVE);
            }
        }
    }
    
    #[test]
    fn check_word(){
        let a_cases = vec![16, 16, 16, 16, 16]; // word size
        let b_cases = vec![11, 16, 20, -11, -20]; // test
        let expect = vec![false, true, true, false, true];
        
        let mut p = Proc::new(Mem::new());
        
        for ((a, b), err) in a_cases.iter().zip(b_cases.iter()).zip(expect.iter()){
            p.poke(0, *a);
            p.poke(1, *b);
            
            p.run(DirectOp::PFIX, 0x5);
            p.run(DirectOp::OPR, 0x6);
            
            if *err{
                assert!(p.state() == ProcState::HALTED);
            }
            else{
                assert!(p.state() == ProcState::ACTIVE);
            }
        }
    }
}

