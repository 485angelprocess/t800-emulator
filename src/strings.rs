use crate::proc::DirectOp;

pub fn direct_op_short_name(op: DirectOp) -> String{
    let name =match op{
        DirectOp::JUMP => "jump",
        DirectOp::CJ => "cj",
        DirectOp::LDLP => "ldlp",
        DirectOp::PFIX => "pfix",
        DirectOp::LDNL => "ldnl",
        DirectOp::LDC => "ldc",
        DirectOp::LDNLP => "ldnlp",
        DirectOp::NFIX => "nfix",
        DirectOp::LDL => "ldl",
        DirectOp::AJW => "ajw",
        DirectOp::CALL => "call",
        DirectOp::EQC => "eqc",
        DirectOp::STL => "stl",
        DirectOp::ADC => "adc",
        DirectOp::STNL => "stnl",
        DirectOp::OPR => "opr"
    };
    name.to_string()
}