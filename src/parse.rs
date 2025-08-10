use crate::proc::DirectOp;

pub fn parse_op_from_hex(v: u8) -> (DirectOp, u8){
    let value = v & 0b1111;
    let o = v >> 4;
    
    let op  = match o{
        0x0 => DirectOp::JUMP,
        0x1 => DirectOp::LDLP,
        0x2 => DirectOp::PFIX,
        0x3 => DirectOp::LDNL,
        0x4 => DirectOp::LDC,
        0x5 => DirectOp::LDNLP,
        0x6 => DirectOp::NFIX,
        0x7 => DirectOp::LDL,
        0x8 => DirectOp::ADC,
        0x9 => DirectOp::CALL,
        0xA => DirectOp::CJ,
        0xB => DirectOp::AJW,
        0xC => DirectOp::EQC,
        0xD => DirectOp::STL,
        0xE => DirectOp::STNL,
        0xF => DirectOp::OPR,
        _ => panic!("Invalid op code {}", o)
    };
    
    (op, value)
}