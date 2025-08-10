fn prefix_constant(op: u8, v: usize) -> Vec<u8>{
    if v < 16 && v < 0{
        return vec![(op << 4) + v];
    }
    else if v >= 16{
        let mut p = Vec::new();
        p.append(prefix_constant(0x2, v >> 4));
        p.push(op, v & 0xF);
        return p
    }
    else{
        let mut p = Vec::new();
        p.append(prefix_constant(nfix, (!v) >> 4));
        p.push(op, v & 0xF);
        return p
    }
}