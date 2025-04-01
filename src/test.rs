#[cfg(test)]
mod tests{
    use crate::{mem::Mem, proc::{DirectOp, Flag, Proc}};
    
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
}