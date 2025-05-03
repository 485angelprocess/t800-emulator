/* Used for special workspace location management */
use crate::mem::Mem;

pub struct EventState{}

pub const NOT_PROCESS_P: i32 = -1; // Pretty sure it's -1
pub const MOST_NEG: i32 = i32::MIN;

impl EventState{
    pub const ENABLING: i32 = i32::MIN + 1;
    pub const WAITING: i32  = i32::MIN + 2;
    pub const READY: i32    = i32::MIN + 3;
}

pub struct ProcPriority{}

impl ProcPriority{
    pub const HIGH: i32 = 1;
    pub const LOW: i32 = 0;
}

pub struct WorkspaceCache{
    mem: Mem
}

impl WorkspaceCache{
    const GUARD: i32 = 0;
    const IPTR: i32 = -4; // Instruction pointer
    const LINK: i32 = -8; // Workspace descriptior of next process in queue
    const STATE: i32= -12; // Flag indicating alternation state
    const TLINK: i32= -16; // Time value reached flag
    const TIME: i32 = -20; // time process watiing to awaken at
    
    pub fn new(mem: Mem) -> Self{
        Self{
            mem: mem
        }
    }
    
    /// Set guard offset
    /// Holds the offset from the end of the alternation
    /// or the instruction after ALTEND
    /// it is initialized as -1 by an ALTWT instruction
    pub fn set_guard_offset(&mut self, wp: i32, offset: i32){
        self.mem.write(wp + WorkspaceCache::GUARD, offset);
    }
    
    /// Get the offset from the end of an alternation
    /// -1 indicates that no message has arrived yet
    pub fn get_guard_offset(&self, wp: i32) -> i32{
        self.mem.read(wp + WorkspaceCache::GUARD)
    }
    
    /// Set the instruction pointer of a descheduled process
    pub fn set_iptr(&mut self, wp: i32, ip: i32){
        self.mem.write(wp + WorkspaceCache::IPTR, ip);
    }
    
    /// Get the instruction pointer of a descheduleued process
    pub fn get_iptr(&self, wp: i32) -> i32{
        self.mem.read(wp + WorkspaceCache::IPTR)
    }
    
    /// Set the workspace descriptor of the next process in quque
    /// This is part of a workspace linked list
    /// a workspace descriptor is the workspace pointer, with the low bit representing the priority
    /// 1 for low priority, 0 for high priority
    pub fn set_link(&mut self, wp: i32, wpd: i32){
        self.mem.write(wp + WorkspaceCache::LINK, wpd);
    }
    
    /// Get the workspace descriptor for the next process in queue
    pub fn get_link(&self, wp: i32) -> i32{
        self.mem.read(wp + WorkspaceCache::LINK)
    }
    
    /// Set state flag
    /// Indicates the state of the alternation
    pub fn set_state(&mut self, wp: i32, state: i32){
        self.mem.write(wp + WorkspaceCache::STATE, state);
    }
    
    /// Get state flag
    pub fn get_state(&self, wp: i32) -> i32{
        self.mem.read(wp + WorkspaceCache::STATE)
    }
    
    /// Tlink is a flag used in the implementation of timer guards
    /// It has two values:
    ///     TimeSet.p = 0x80000001; timer set
    ///     TimeNotSet.p = 0x80000002; timer not set
    pub fn set_tlink(&mut self, wp: i32, flag: i32){
        self.mem.write(wp + WorkspaceCache::TLINK, flag);
    }
    
    /// Get tlink flag
    pub fn get_tlink(&self, wp: i32) -> i32{
        self.mem.read(wp + WorkspaceCache::TLINK)
    }
    
    /// Set Time.s
    /// Contains the time a process is waiting before it times out and contineus
    /// used in timer alternations TALT and TALTWT
    pub fn set_time(&mut self, wp: i32, time: i32){
        self.mem.write(wp + WorkspaceCache::TIME, time);
    }
    
    /// Get time s,
    /// Time a process is waiting for
    pub fn get_time(&self, wp: i32) -> i32{
        self.mem.read(wp + WorkspaceCache::TIME)
    }
}