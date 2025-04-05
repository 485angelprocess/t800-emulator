use std::{io, thread, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend}, layout::{Constraint, Direction, Layout, Rect}, style::Modifier, widgets::{Block, Borders, Paragraph, Widget}, Frame, Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{style::{Color, Style}, widgets::{Row, Table}};

/* Making a cute visualizer for processor state */
use crate::{mem::Mem, proc::Proc, DirectOp};
use crate::strings;

const COLOR_STANDARD: Color = Color::Rgb(255, 255, 255);
const COLOR_ACTIVE: Color = Color::Rgb(255, 100, 100);

struct StyleTag{
    color: Color
}

impl Default for StyleTag{
    fn default() -> Self {
        Self{
            color: COLOR_STANDARD
        }
    }
}

impl StyleTag{
    fn as_style(&self, active: bool) -> Style{
        
        if active{
            return Style::default().fg(COLOR_ACTIVE)
                .add_modifier(Modifier::SLOW_BLINK)
                .add_modifier(Modifier::BOLD);
        }
        return Style::default().fg(self.color);
    }
}

trait RowEntry{
    fn as_row(&self, number: usize, active: bool) -> Row;
}

struct Instruction{
    pub operation: DirectOp,
    pub value: i32,
    breakpoint: bool,
    style: StyleTag,
    // TODO add process identifier
    pub alias: Option<String>
}

impl Instruction{
    fn new(op: DirectOp, value: i32) -> Self{
        Self{
            operation: op,
            value: value,
            breakpoint: false,
            style: StyleTag::default(),
            alias: None
        }
    }
}

impl RowEntry for Instruction{
    fn as_row(&self, number: usize, active: bool) -> Row{
        let mut v = Vec::new();
        
        v.push(format!("{:#02X}", number));
        
        if self.breakpoint{
            v.push("b".to_string());
        }
        else{
            v.push(" ".to_string());
        }
        
        v.push(strings::direct_op_short_name(self.operation));
        v.push(format!("{:#01X}", self.value));
        
        // Additional column
        if let Some(alias) = &self.alias{
            v.push(alias.clone());
        }
        
        Row::new(v)
                .style(self.style.as_style(active))
    }
}

impl Instruction{
    fn set_alias(&mut self, s: String){
        self.alias = Some(s);
    }
    
    fn toggle_breakpont(&mut self){
        self.breakpoint = !self.breakpoint;
    }
}

struct ScrollTable<T:RowEntry>{
    pub display_pointer: usize,
    pub active_pointer: usize,
    pub footer: bool,
    pub height: usize,
    pub contents: Vec<T>,
}

impl<T:RowEntry> Default for ScrollTable<T>{
    fn default() -> Self {
        Self{
            display_pointer: 0,
            active_pointer: 0,
            height: 20,
            footer: false,
            contents: Vec::new()
        }
    }
}

impl<T:RowEntry> ScrollTable<T>{
    fn as_rows(&self) -> Vec<Row>{
        let mut v = Vec::new();
        let max_value = self.display_pointer + self.height;
        for i in self.display_pointer..max_value{
            if i < self.contents.len(){
                v.push(self.contents[i].as_row(i, self.active_pointer == i));
            }
            else{
                if self.footer{
                    v.push(Row::new(vec!["end", "x"]).style(Style::default().fg(Color::Green)));
                }
                return v;
            }
        }
        v
    }
    pub fn as_table(&self) -> Table{
        Table::new(self.as_rows())
    }
    pub fn up(&mut self){
        if self.active_pointer > 0{
            self.active_pointer -= 1;
        }
    }
    pub fn down(&mut self){
        if self.active_pointer < self.contents.len() - 1{
            self.active_pointer += 1;
        }
    }
    pub fn entry(&mut self, index: usize) -> &mut T{
        &mut self.contents[index]
    }
    pub fn active(&mut self) -> &mut T{
        self.entry(self.active_pointer)
    }
}

struct IntEntry{
    label: String,
    pub value: i32
}

impl IntEntry{
    fn new(label: String) -> Self{
        Self{
            label: label,
            value: 0
        }
    }
}

impl RowEntry for IntEntry{
    fn as_row(&self, number: usize, active: bool) -> Row {
        Row::new(vec![
            self.label.clone(),
            format!("{:#08X}", self.value)
        ])
    }
}

struct MemoryEntry{
    pub address: i32,
    pub value: i32
}

impl MemoryEntry{
    fn new(address: i32, value: i32) -> Self{
        Self{
            address: address,
            value: value
        }
    }
}

impl RowEntry for MemoryEntry{
    fn as_row(&self, number: usize, active: bool) -> Row {
        if active{
            return Row::new(vec![
                format!("{:#08X}", self.address),
                format!("{:#08X} <-- wp", self.value)
            ]).style(Style::default().fg(Color::Magenta));
        }
        Row::new(vec![
            format!("{:#08X}", self.address),
            format!("{:#08X}", self.value)
        ])
    }
}

struct ProcessMemory{
    pub table: ScrollTable<MemoryEntry>
}

impl ProcessMemory{
    fn new() -> Self{
        Self{
            table: ScrollTable::default()
        }
    }
    
    fn update(&mut self, p: &Proc, m: &Mem){
        let workspace = p.workspace_pointer();
        
        if workspace % 4 != 0{
            panic!("Invalid workspace value {}", workspace);
        }
        
        self.table.contents.clear();
        
        
        for i in 0..self.table.height{
            
            let offset = i as i32 - (self.table.height >> 1) as i32;
            let address = workspace + (offset << 2);
            
            if address >= 0{
                let entry = m.read(address);
                let me = MemoryEntry::new(address as i32, entry);
            
                self.table.contents.push(me);
                if address == workspace{
                    self.table.active_pointer = self.table.contents.len() - 1;
                }
            }
            
            
        }
    }
}


struct Stack{
    pub table: ScrollTable<IntEntry>
}

impl Stack{
    fn new() -> Self{
        let mut table = ScrollTable::default();
        table.contents = vec![
            IntEntry::new("A".to_string()),
            IntEntry::new("B".to_string()),
            IntEntry::new("C".to_string())
        ];
        Self{
            table: table
        }
    }

    fn update(&mut self, p: &Proc){
        self.table.contents[0].value = p.peek(0);
        self.table.contents[1].value = p.peek(1);
        self.table.contents[2].value = p.peek(2);
    }
}

pub struct ProcessorTui{
    proc: Proc,
    mem: Mem,
    instructions: ScrollTable<Instruction>,
    register: Stack,
    memdisplay: ProcessMemory
}

impl ProcessorTui{
    pub fn new() -> Self{
        let m = Mem::new();
        let p = Proc::new(m.clone());
        Self{
            mem: m.clone(),
            proc: p,
            instructions: ScrollTable::default(),
            register: Stack::new(),
            memdisplay: ProcessMemory::new()
        }
    }
    
    pub fn upload_instruction(&mut self, op: DirectOp, value: i32){
        self.instructions.contents.push(Instruction::new(op, value));
    }
    
    fn draw<B: Backend>(&self, f: &mut Frame<B>){
        
        let v_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                [
                    Constraint::Length(8),
                    Constraint::Length(1),
                    Constraint::Min(5)
                ]
            )
        .split(f.size());
        
        let h_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30)  
            ].as_ref()
        )
        .split(v_layout[2]);
        
        let commands = Table::new(vec![
            Row::new(vec!["[s]", "step program"]),
            Row::new(vec!["[c]", "reset program"]),
            Row::new(vec!["[b]", "set breakpoint"]),
            Row::new(vec!["[B]", "clear breakpoint"]),
            Row::new(vec!["[r]", "run program"])
        ]).block(Block::default().title("Commands").borders(Borders::ALL))
                        .widths(&[Constraint::Length(3), Constraint::Length(32)])
                        .column_spacing(1);
        
        f.render_widget(commands, v_layout[0]);
        
        let p = Paragraph::new(format!("Program Counter: {:#04X}", self.proc.program_counter()));
            
        f.render_widget(p, v_layout[1]);
            
        // Instructions
        let inst = self.instructions.as_table()
                        .block(Block::default().title("Instructions").borders(Borders::ALL))
                        .widths(&[Constraint::Length(5),  // Line number
                                    Constraint::Length(2),  // breakpoint
                                    Constraint::Length(5),  // operation
                                    Constraint::Length(5), // value
                                    Constraint::Length(12)])
                        .column_spacing(1);
            
        f.render_widget(inst, h_layout[0]);
        
        // Register update
        let reg = self.register.table.as_table()
                        .block(Block::default().title("Registers").borders(Borders::ALL))
                        .widths(&[Constraint::Length(2), Constraint::Length(8)])
                        .column_spacing(1);
        
        f.render_widget(reg, h_layout[1]);
        
        let mem = self.memdisplay.table.as_table()
                                .block(Block::default().title("Memory Stack").borders(Borders::ALL))
                                .widths(&([Constraint::Length(10), Constraint::Length(15)]))
                                .column_spacing(1);
    
        f.render_widget(mem, h_layout[2]);
        
    }
    
    fn update_alias(&mut self){
        let c = &mut self.instructions.contents;
        for i in 0..c.len(){
            if c[i].operation == DirectOp::OPR{
                if i > 0{
                    let mut prefix = 0;
                    if c[i-1].operation == DirectOp::PFIX{
                        prefix = c[i-1].value;
                        c[i - 1].set_alias("┑".to_string());
                    }
                    let value = c[i].value;
                    // TODO get actual name
                    c[i].set_alias(format!("┛ 0x2{:01X} 0xF{:01X}", prefix, value));
                }
            }
        }
    }
    
    fn update(&mut self){
        let inst = &mut self.instructions.contents;
        
        // update program counter highlight
        let pc_p = self.proc.program_counter() as usize;
        
        for i in 0..inst.len(){
            if i == pc_p{
                inst[i].style.color = Color::Green;
            }
            else{
                inst[i].style.color = COLOR_STANDARD;
            }
        }
        if pc_p >= self.instructions.contents.len(){
            self.instructions.footer = true;
        }
        else{
            self.instructions.footer = false;
        }
        
        // update register values
        self.register.update(&self.proc);
        self.memdisplay.update(&self.proc, &self.mem);
    }
    
    /// Clear process registers and program counter
    fn clear(&mut self){
        self.proc.reset(0x8000);
        self.update();
    }
    
    fn step(&mut self){
        let pc = self.proc.program_counter();
        
        let inst = &mut self.instructions.contents;
        
        if pc < inst.len() as i32{
            self.proc.run(inst[pc as usize].operation, inst[pc as usize].value);
        }
        
        self.update();
    }
    
    fn run_checked(&mut self, pc: usize) -> bool{
        let op = self.instructions.contents[pc].operation;
        let value = self.instructions.contents[pc].value;
        if let Err(e) = self.proc.run(op, value){
            // TODO print error
            return false;
        }
        return true;
    }
    
    /// Run program from cleared state 
    fn run_program(&mut self){
        self.clear();
        let inst = &mut self.instructions.contents;
        
        let len = inst.len();
        
        for _ in 0..500{ // dont run infinitely is all
            let pc = self.proc.program_counter();
            if pc < len as i32{
                if self.instructions.contents[pc as usize].breakpoint{
                    self.update();
                    return;
                }
                
                let result = self.run_checked(pc as usize);
                if !result{
                    return;
                }
                
            }
            else{
                self.update();
                return;
            }
        }
        self.update();
    }
    
    fn run_app<B:Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()>{
        
        self.proc.set_workspace_pointer(0x8000);
        
        self.update();
        self.update_alias();
        
        
        
        loop{
            terminal.draw(|f| self.draw(f))?;
            
            if let Event::Key(key) = event::read()? {
                match key.kind{
                    KeyEventKind::Release => {
                        match key.code{
                            KeyCode::Up => self.instructions.up(),
                            KeyCode::Down => self.instructions.down(),
                            KeyCode::Char('b') => self.instructions.active().breakpoint = true,
                            KeyCode::Char('B') => self.instructions.active().breakpoint = false,
                            KeyCode::Char('s') => self.step(),
                            KeyCode::Char('c') => self.clear(),
                            KeyCode::Char('r') => self.run_program(),
                            KeyCode::Char('q') => {
                                return Ok(())
                            }
                            _ => ()
                        }
                    },
                    _ => ()
                }
                
            }
            
            
        }
    }
    
    pub fn run(&mut self) -> Result<(), io::Error>{
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
    
        let res = self.run_app(&mut terminal);
    
        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        if let Err(err) = res {
            println!("{:?}", err)
        }
    
        Ok(())
    }
}