use crate::process::context::ProcessContext;
use crate::process::memory::MemoryRegion;
use crate::process::stack::KernelStack;
use crate::process::state::ProcessState;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::{PhysAddr, VirtAddr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(pub u64);

impl ProcessId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1); // PID 1通常是内核进程
        ProcessId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(Clone)]
pub struct Process {
    /// 进程ID
    pub pid: ProcessId,
    /// 进程状态
    pub state: ProcessState,
    /// 保存的CPU上下文
    pub context: ProcessContext,
    /// 内核栈
    pub kernel_stack: KernelStack,
    /// 用户栈顶指针（如果有用户空间）
    pub user_stack: Option<VirtAddr>,
    /// 页表地址（CR3值）
    pub page_table: PhysAddr,
    /// 进程优先级（调度用）
    pub priority: u8,
    /// 父进程ID
    pub parent_pid: Option<ProcessId>,
    /// 退出码
    pub exit_code: Option<i32>,
    /// 进程名称（用于调试）
    pub name: &'static str,
    /// 内存映射信息
    pub memory_regions: Vec<MemoryRegion>,
}
