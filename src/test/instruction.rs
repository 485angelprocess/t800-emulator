#[cfg(test)]
mod direct{
    use crate::{mem::Mem, proc::{DirectOp, ProcState, Flag, Proc}};
    
    #[test]
    fn load_constant(){
        let mut p = Proc::new(Mem::new());
        assert!(p.get_reg(0) == 0);
        let _ = p.run(DirectOp::LDC, 5);
        assert!(p.get_reg(0) == 5);
        assert!(p.get_reg(1) == 0);
        assert!(p.get_reg(2) == 0);
        let _ = p.run(DirectOp::LDC, 10);
        assert!(p.get_reg(0) == 10);
        assert!(p.get_reg(1) == 5);
        assert!(p.get_reg(2) == 0);
    }
    
    #[test]
    fn add_constant(){
        let mut p = Proc::new(Mem::new());
        
        // standard addition
        assert!(p.get_reg(0) == 0);
        p.set_reg(0, 5);
        let _ = p.run(DirectOp::LDC, 6);
        let _ = p.run(DirectOp::ADC, 3);
        assert!(p.get_reg(0) == 9);
        assert!(p.get_reg(1) == 5);
        
        // Overflow
        p.set_reg(0, 2_147_483_647);
        let _ = p.run(DirectOp::ADC, 3);
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
        
        let _ = p.run(DirectOp::LDL, 0);
        assert!(p.get_reg(0) == 9);
        
        let _ = p.run(DirectOp::LDL, 1);
        assert!(p.get_reg(0) == 11);
        assert!(p.get_reg(1) == 9);
        
        let _ = p.run(DirectOp::LDL, 2);
        assert!(p.get_reg(0) == 13);
        assert!(p.get_reg(1) == 11);
        assert!(p.get_reg(2) == 9);
    }
    
    #[test]
    fn store_local(){
        let mut m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        m.write(8, 10);
        m.write(12, 11);
        m.write(16, 13);
        
        p.set_workspace_pointer(8);
        
        p.set_reg(0, 15);
        let _ = p.run(DirectOp::STL, 0);
        assert!(m.read(8) == 15);
        
        let _ = p.run(DirectOp::STL, 1);
        assert!(m.read(12) == 15);
    }
    
    #[test]
    fn load_local_pointer(){
        let m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        p.set_workspace_pointer(1000);
        let _ = p.run(DirectOp::LDLP, 0);
        assert!(p.get_reg(0) == 1000);
        
        let _ = p.run(DirectOp::LDLP, 1);
        assert!(p.get_reg(0) == 1004);
        assert!(p.get_reg(1) == 1000);
    }

    #[test]
    fn load_non_local(){
        let mut m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        p.set_reg(0, 0x10004000);
        p.set_reg(1, 20);
        m.write(0x10004000, 5);
        m.write(0x10004004, 7);
     
        let _ = p.run(DirectOp::LDNL, 0);
        assert!(p.get_reg(0) == 5);
        assert!(p.get_reg(1) == 20);
        
        p.set_reg(0, 0x10004000);
        let _ = p.run(DirectOp::LDNL, 1);
        assert!(p.get_reg(0) == 7);
        assert!(p.get_reg(1) == 20);
    }
    
    #[test]
    fn store_non_local(){
        let m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        p.set_reg(0, 0x10004000);
        p.set_reg(1, 20);
        
        let _ = p.run(DirectOp::STNL, 0);
        assert!(p.get_reg(0) == 0x10004000);
        assert!(m.read(0x10004000) == 20);
        
        p.set_reg(0, 0x10004000);
        p.set_reg(1, 15);
        
        let _ = p.run(DirectOp::STNL, 1);
        assert!(p.get_reg(0) == 0x10004000);
        assert!(m.read(0x10004004) == 15);
    }

    #[test]
    fn load_non_local_pointer(){
        let m = Mem::new();
        let mut p = Proc::new(m.clone());
        
        p.set_reg(0, 8);
        p.set_reg(1, 10);
        let _ = p.run(DirectOp::LDNLP, 1);
        assert!(p.get_reg(0) == 12);
        assert!(p.get_reg(1) == 10);
    }

    #[test]
    fn adjust_workspace(){
        let mut p = Proc::new(Mem::new());
        p.set_workspace_pointer(1000);
        assert!(p.workspace_pointer() == 1000);
        let _ = p.run(DirectOp::AJW, -2);
        assert!(p.workspace_pointer() == 992);
        let _ = p.run(DirectOp::AJW, 2);
        assert!(p.workspace_pointer() == 1000);
    }
    
    #[test]
    fn prefix(){
        let mut p = Proc::new(Mem::new());
        
        let _ = p.run(DirectOp::PFIX, 4);
        let _ = p.run(DirectOp::PFIX, 3);
        let _ = p.run(DirectOp::PFIX, 2);
        let _ = p.run(DirectOp::LDC, 1);
        assert!(p.get_reg(0) == 0x4321);
        
        let _ = p.run(DirectOp::NFIX, 1);
        let _ = p.run(DirectOp::LDC, 1);
        println!("Value of A is {}", p.get_reg(0));
        assert!(p.get_reg(0) == -31);
    }

    #[test]
    fn jump(){
        let mut p = Proc::new(Mem::new());
        let _ = p.run(DirectOp::JUMP, 2);
        assert!(p.program_counter() == 3);
        let _ = p.run(DirectOp::JUMP, 1);
        assert!(p.program_counter() == 5);
        assert!(p.state() == ProcState::IDLE);
    }
    
    #[test]
    fn conditional(){
        let mut p = Proc::new(Mem::new());
        
        // Run three programs and don't jump
        let _ = p.run(DirectOp::LDC, 1);
        let _ = p.run(DirectOp::EQC, 1);
        assert!(p.get_reg(0) == 1);
        let _ = p.run(DirectOp::CJ, 4);
        assert!(p.program_counter() == 3);
        
        // Run three instruction and then jump
        let _ = p.run(DirectOp::LDC, 2);
        let _ = p.run(DirectOp::EQC, 1);
        assert!(p.get_reg(0) == 0);
        assert!(p.program_counter() == 5);
        let _ = p.run(DirectOp::CJ, 4);
        println!("Program counter {}", p.program_counter());
        assert!(p.program_counter() == 10);
    }
}

mod indirect{

    use crate::{mem::Mem, proc::{DirectOp, ProcState, Flag, Proc}};
    
    #[test]
    fn reverse(){
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 11);
        p.set_reg(1, 13);
        // run reverse
        let _ = p.run(DirectOp::OPR, 0);
        
        assert!(p.get_reg(0) == 13);
        assert!(p.get_reg(1) == 11);
    }
    
    #[test]
    fn add(){
        // A = A + B,
        // If overflow set error
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 5);
        p.set_reg(1, 7);
        p.set_reg(2, 9);
        
        let _ = p.run(DirectOp::PFIX, 0);
        let _ = p.run(DirectOp::OPR, 5);
        
        assert!(p.get_reg(0) == 12);
        assert!(p.get_reg(1) == 9);
        
        assert!(p.flag(Flag::ERROR) == false);
        
        // Overflow
        p.set_reg(0, 2_147_483_647);
        let _ = p.run(DirectOp::PFIX, 0);
        let _ = p.run(DirectOp::OPR, 5);
        
        assert!(p.flag(Flag::ERROR) == true);
    }
    
    #[test]
    fn alt(){
        let mut p = Proc::new(Mem::new());
        
        let _ = p.run(DirectOp::PFIX, 4);
        let _ = p.run(DirectOp::OPR, 3);
        
    }
    
    #[test]
    fn and(){
        // Perform logical and of register a and b
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 0b011111);
        p.set_reg(1, 0b101010);
        p.set_reg(2, 15);
        
        let _ = p.run(DirectOp::PFIX, 0x4);
        let _ = p.run(DirectOp::OPR, 0xB);
        
        assert!(p.get_reg(0) == 0b001010);
        // Copies C into B
        assert!(p.get_reg(1) == 15);
    }
    
    #[test]
    fn byte_count(){
        // Test BCNT
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 5);
        p.set_reg(1, 11);
        
        // 0x23 0xF4
        let _ = p.run(DirectOp::PFIX, 0x3);
        let _ = p.run(DirectOp::OPR, 0x4);
        
        assert!(p.get_reg(0) == 20);
        assert!(p.get_reg(1) == 11);
    }
    
    #[test]
    fn bit_counts(){
        // Test BITCNT 
        // 0x27 F6
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 0b10101);
        p.set_reg(1, 7);
        p.set_reg(2, 11);
        
        let _ = p.run(DirectOp::PFIX, 0x7);
        let _ = p.run(DirectOp::OPR, 0x6);
        
        assert!(p.get_reg(0) == 10);
        assert!(p.get_reg(1) == 11);
    }
    
    #[test]
    fn bit_rev_n_bits(){
        // Test bit reverse
        // reverses n bits of B
        let mut p = Proc::new(Mem::new());
        p.set_reg(0, 4); // number of bits to reverse
        p.set_reg(1, 0b1010_0101);
        p.set_reg(2, 13);
        
        let _ = p.run(DirectOp::PFIX, 0x7);
        let _ = p.run(DirectOp::OPR, 0x8);
        
        assert!(p.get_reg(0) == 0b1010_1010);
        assert!(p.get_reg(1) == 13);
    }
    
    #[test]
    fn bit_rev_word(){
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 0x12345678);
        
        let _ = p.run(DirectOp::PFIX, 0x7);
        let _ = p.run(DirectOp::OPR, 0x7);
        
        assert!(p.get_reg(0) == 0x1E6A2C48);
    }
    
    #[test]
    fn bsub(){
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 13);
        p.set_reg(1, 15);
        p.set_reg(2, 17);
        let _ = p.run(DirectOp::OPR, 0x2);
        
        assert!(p.get_reg(0) == 28);
        assert!(p.get_reg(1) == 17);
    }
    
    #[test]
    fn ccnt1(){
        let mut p = Proc::new(Mem::new());
        
        
        // Case B == 0, sets error
        p.set_reg(0, 10);
        p.set_reg(1, 0);
        p.set_reg(2, 5);
        
        let _ = p.run(DirectOp::PFIX, 0x4);
        let _ = p.run(DirectOp::OPR, 0xD);
        
        assert!(p.get_reg(0) == 0);
        assert!(p.get_reg(1) == 5);
        
        assert!(p.flag(Flag::ERROR) == true);
        p.clear();
        
        // Case B < A and B > 0, error is not set
        p.set_reg(0, 10);
        p.set_reg(1, 1);
        p.set_reg(2, 5);
        
        let _ = p.run(DirectOp::PFIX, 0x4);
        let _ = p.run(DirectOp::OPR, 0xD);
        
        assert!(p.get_reg(0) == 1);
        assert!(p.get_reg(1) == 5);
        
        assert!(p.flag(Flag::ERROR) == false);
        p.clear();
        
        // Case B > A, error is set
        p.set_reg(0, 10);
        p.set_reg(1, 100);
        p.set_reg(2, 5);
        
        let _ = p.run(DirectOp::PFIX, 0x4);
        let _ = p.run(DirectOp::OPR, 0xD);
        
        assert!(p.get_reg(0) == 100);
        assert!(p.get_reg(1) == 5);
        
        assert!(p.flag(Flag::ERROR) == true);
        p.clear();
        
        p.set_reg(0, -12);
        p.set_reg(1, 1);
        p.set_reg(2, 5);
        
        let _ = p.run(DirectOp::PFIX, 0x4);
        let _ = p.run(DirectOp::OPR, 0xD);
        
        assert!(p.get_reg(0) == 1);
        assert!(p.get_reg(1) == 5);
        
        assert!(p.flag(Flag::ERROR) == false);
        p.clear();
    }
    
    #[test]
    fn clrhalterr(){
        let mut p = Proc::new(Mem::new());
        
        // Case B == 0, sets error
        p.set_reg(0, 10);
        p.set_reg(1, 0);
        p.set_reg(2, 5);
        
        let _ = p.run(DirectOp::PFIX, 0x4);
        let _ = p.run(DirectOp::OPR, 0xD);
        
        assert!(p.get_reg(0) == 0);
        assert!(p.get_reg(1) == 5);
        
        assert!(p.state() == ProcState::HALTED);
        p.clear();
        
        // Case B == 0, sets error
        p.set_reg(0, 10);
        p.set_reg(1, 0);
        p.set_reg(2, 5);
        
        let _ = p.run(DirectOp::PFIX, 0x2);
        let _ = p.run(DirectOp::OPR, 0x7);
        
        let _ = p.run(DirectOp::PFIX, 0x4);
        let _ = p.run(DirectOp::OPR, 0xD);
        
        assert!(p.get_reg(0) == 0);
        assert!(p.get_reg(1) == 5);
        
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
            p.set_reg(0, a);
            p.set_reg(1, b);
            
            let _ = p.run(DirectOp::PFIX, 0x4);
            let _ = p.run(DirectOp::OPR, 0xC);
            
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
            p.set_reg(0, *a);
            p.set_reg(1, *b);
            
            let _ = p.run(DirectOp::PFIX, 0x1);
            let _ = p.run(DirectOp::OPR, 0x3);
            
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
            p.set_reg(0, *a);
            p.set_reg(1, *b);
            
            let _ = p.run(DirectOp::PFIX, 0x5);
            let _ = p.run(DirectOp::OPR, 0x6);
            
            if *err{
                assert!(p.state() == ProcState::HALTED);
            }
            else{
                assert!(p.state() == ProcState::ACTIVE);
            }
        }
    }
    
    #[test]
    fn diff(){
        // Gets the different of B - A, and pops C into V
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 11);
        p.set_reg(1, 13);
        p.set_reg(2, 16);
        
        let _ = p.run(DirectOp::OPR, 0x4);
        
        assert!(p.get_reg(0) == 2);
        assert!(p.get_reg(1) == 16);
    }
    
    #[test]
    fn div(){
        // Divide B by A (B / A)
        
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 2);
        p.set_reg(1, 8);
        p.set_reg(2, 16);
        
        let _ = p.run(DirectOp::PFIX, 0x2);
        let _ = p.run(DirectOp::OPR, 0xC);
        
        assert!(p.get_reg(0) == 4);
        assert!(p.get_reg(1) == 16);
        
        p.set_reg(0, 0);
        p.set_reg(1, 8);
        p.set_reg(2, 16);
        
        let _ = p.run(DirectOp::PFIX, 0x2);
        let _ = p.run(DirectOp::OPR, 0xC);
        
        assert!(p.state() == ProcState::HALTED);
    }
    
    #[test]
    fn disc(){
        // Disable channel
        panic!("Disable channel test not set up");
    }
    
    #[test]
    fn diss(){
        // Disable Skip
        panic!("Diss test not set up");
        
        // Need to get to ENBS
        // Disable skip disables a skip for a guard that was previously initialized with ENBS. A contains an offset to the guard to
        //disable and B contains a flag that allows it to decide whether to select this guard or not. C is popped into B
    }
    
    #[test]
    fn dist(){
        // Disable timer
        // Disables timer guard, 
        panic!("Disable timer not set");
    }
    
    #[test]
    fn dup(){
        // Duplicate stack
        let mut p = Proc::new(Mem::new());
        
        p.set_reg(0, 15);
        p.set_reg(1, 13);
        p.set_reg(2, 11);
    
        let _ = p.run(DirectOp::PFIX, 0x5);
        let _ = p.run(DirectOp::OPR, 0xA);
        
        // Duplicates A and pushes back into B, B goes into C
        assert!(p.get_reg(0) == 15);
        assert!(p.get_reg(1) == 15);
        assert!(p.get_reg(2) == 13);
    }
}

