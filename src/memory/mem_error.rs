use super::Memory;
use std::error::Error;
use std::fmt;

#[derive(PartialEq, Debug)]
pub enum MemoryOperation {
    Read,
    Write,
    Load,
}

#[derive(Debug)]
pub struct MemoryError {
    operation: MemoryOperation,
    address: usize,
    access_size: usize,
    msg: String,
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "address: ${:x}, msg: {})", self.address, self.msg)
    }
}

impl From<std::io::Error> for MemoryError {
    fn from(err: std::io::Error) -> Self {
        MemoryError {
            operation: MemoryOperation::Load,
            address: 0,
            access_size: 0,
            msg: format!("Load error: {}", err.to_string()),
        }
    }
}

/**
 * check memory boundaries
 */
pub fn check_address(
    mem: &impl Memory,
    address: usize,
    access_size: usize,
    op: MemoryOperation,
) -> Result<bool, MemoryError> {
    if address + access_size > mem.size() {
        let e = MemoryError {
            operation: MemoryOperation::Read,
            address: address,
            access_size: access_size,
            msg: format!("Error, max memory size={}", mem.size()),
        };
        return Err(e);
    }
    Ok(true)
}
