use std::{any::Any, collections::HashMap};

use crate::proc::{DirectOp, Proc};

fn prefix_constant(op: u8, v: i32) -> Vec<u8>{
    if v < 16 && v >= 0{
        return vec![(op << 4) + (v as u8)];
    }
    else if v >= 16{
        let mut p = Vec::new();
        for fix in prefix_constant(0x2, v >> 4){
            p.push(fix);
        }
        p.push((op << 4) + ((v & 0xF) as u8));
        return p
    }
    else{
        let mut p = Vec::new();
        for fix in prefix_constant(0x6, (!v) >> 4){
            p.push(fix);
        }
        p.push((op << 4) + ((v & 0xF) as u8));
        return p
    }
}

enum Token{
    Op( (String, i32) ),
    OpPromise( (String, String) ),
    IndirectOp(String),
    Label(String)
}

struct Reader<'a>{
    words: Vec<&'a str>,
    pos: usize
}

impl<'a> Reader<'a>{
    fn done(&self) -> bool{
        self.words.len() <= self.pos
    }
    
    fn finish(&mut self){
        self.pos = self.words.len()
    }
    
    fn get(&self) -> String{
        self.words[self.pos].to_string()
    }
    
    fn next(&mut self){
        self.pos += 1;
    }
}

pub struct Assemble{
    labels: HashMap<String, i32>,
    op: HashMap<String, u8>,
    iop: HashMap<String, usize>,
    line_number: usize
}

impl Assemble{
    pub fn new() -> Self{
        Self{
            labels: HashMap::new(),
            op: HashMap::new(),
            iop: HashMap::new(),
            line_number: 0
        }
    }
    
    pub fn setup(&mut self, proc: &Proc){
        self.define_op("j", 0x0);
        self.define_op("ldlp", 0x1);
        self.define_op("pfix", 0x2);
        self.define_op("ldnl", 0x3);
        self.define_op("ldc", 0x4);
        self.define_op("ldnlp", 0x5);
        self.define_op("nfix", 0x6);
        self.define_op("ldl", 0x7);
        self.define_op("adc", 0x8);
        self.define_op("call", 0x9);
        self.define_op("cj", 0xA);
        self.define_op("ajw", 0xB);
        self.define_op("eqc", 0xC);
        self.define_op("stl", 0xD);
        self.define_op("stnl", 0xE);
        self.define_op("opr", 0xF);
        
        for (op, value) in proc.get_indirect_ops(){
            self.define_iop(op.clone(), value);
        }
    }
    
    fn define_op(&mut self, word: &str, value: u8){
        self.op.insert(word.to_string(), value);
    }
    
    fn define_iop(&mut self, word: String, value: usize){
        self.iop.insert(word, value);
    }
    
    fn load_token(&mut self, t: &Token) -> Option<Vec<u8>>{
        match t{
            Token::Label(label) => {
                self.labels.insert(label.clone(), self.line_number as i32);
                None
            },
            Token::Op((op, v)) => {
                let opcode = self.op[op];
                Some(prefix_constant(opcode, *v))
            },
            Token::IndirectOp(op) => {
                let opcode = 0xF;
                let v = self.iop[op];
                Some(prefix_constant(opcode, v as i32))
            },
            Token::OpPromise( (op, label) ) => {
                let opcode = self.op[op];
                let c = self.labels[label];
                Some(prefix_constant(opcode, c))
            }
        }
    }
    
    pub fn read_line(&mut self, line: &str) -> Option<Vec<u8>>{
        match self.tokenize(line.to_string()){
            Some(tokens) => {
                let mut ops = Vec::new();
                for t in tokens{
                    if let Some(prefixed_op) = self.load_token(&t){
                        for o in prefixed_op{
                            self.line_number += 1;
                            ops.push(o);
                        }
                    }
                }
                if ops.len() > 0{
                    Some(ops)
                }
                else{
                    None
                }
            },
            None => None
        }
    }
    
    fn read(&self, reader: &mut Reader) -> Option<Token>{
        let w = reader.get();
        if w.ends_with(":"){
            // Label
            return Some(Token::Label(w));
        }
        if self.op.contains_key(&w){
            reader.next();
            let v = reader.get();
            return match v.parse::<i32>(){
                Ok(num) => Some(Token::Op( (w, num) )),
                Err(e) => {
                    if self.labels.contains_key(&v){
                        let label_address = self.labels[&v];
                        Some(Token::Op( (w, label_address) ))
                    }
                    else{
                        Some(Token::OpPromise((w, v)))
                    }
                }
            };
        }
        match w.as_str(){
            ";" =>  {reader.finish(); return None;},
            _ => {
                Some(Token::IndirectOp(w.clone()))
            }
        }
    }
    
    fn tokenize(&self, line: String) -> Option<Vec<Token>>{
        // Split by spaces
        let line_trimmed = line.trim().to_string();
        let words: Vec<&str> = line_trimmed.split(" ").collect();
        
        if words.len() > 0{
            let mut reader = Reader{
                words: words,
                pos: 0
            };
            
            let mut tokens = Vec::new();
            
            while !reader.done(){
                match self.read(&mut reader){
                    Some(t) => tokens.push(t),
                    _ => ()
                };
                reader.next();
            }
            
            return Some(tokens);
        }
        None
    }
}