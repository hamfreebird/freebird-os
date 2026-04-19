use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    MemoryAllocationFailed,
    ProcessIdExists,
    ProcessNotFound,
    InvalidProcessState,
    PageMappingFailed,
    StackAllocationFailed,
    PageTableCloneFailed,
    InvalidArgument,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            ProcessError::ProcessIdExists => write!(f, "Process ID already exists"),
            ProcessError::ProcessNotFound => write!(f, "Process not found"),
            ProcessError::InvalidProcessState => write!(f, "Invalid process state"),
            ProcessError::PageMappingFailed => write!(f, "Page mapping failed"),
            ProcessError::StackAllocationFailed => write!(f, "Stack allocation failed"),
            ProcessError::PageTableCloneFailed => write!(f, "Page table clone failed"),
            ProcessError::InvalidArgument => write!(f, "Invalid argument"),
        }
    }
}
