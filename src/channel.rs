

struct Link<T: Default>{
    tx_buffer: Vec<T>,
    rx_buffer: Vec<T>,
    delay: usize
}

/// Transputer to transputer channel
struct Channel{
    
}