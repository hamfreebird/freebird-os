use crate::process::ProcessContext;
use crate::process::error::ProcessError;
use alloc::alloc::alloc_zeroed;
use core::alloc::Layout;
use x86_64::VirtAddr;

const PAGE_SIZE: usize = 4096;

#[derive(Clone)]
pub struct KernelStack {
    /// 栈底（高地址）
    bottom: VirtAddr,
    /// 栈顶（低地址）
    top: VirtAddr,
    /// 栈大小
    size: usize,
}

impl KernelStack {
    /// 为进程分配内核栈
    pub fn allocate(size: usize) -> Result<Self, ProcessError> {
        // 从内核堆中分配内存
        let layout = Layout::from_size_align(size, PAGE_SIZE)
            .map_err(|_| ProcessError::MemoryAllocationFailed)?;

        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            return Err(ProcessError::MemoryAllocationFailed);
        }

        let bottom = VirtAddr::from_ptr(ptr).align_up(PAGE_SIZE as u64);
        let top = bottom + size;

        Ok(KernelStack { bottom, top, size })
    }

    /// 设置栈上的初始上下文
    pub fn setup_initial_context(&mut self, entry_point: VirtAddr) -> ProcessContext {
        // 在栈顶预留空间保存初始寄存器值
        let mut context = ProcessContext::new();
        context.rip = entry_point.as_u64();
        context.rsp = self.top.as_u64() - 8; // 预留返回地址空间
        context.rbp = context.rsp;
        context
    }

    pub fn top(&self) -> VirtAddr {
        self.top
    }
    pub fn bottom(&self) -> VirtAddr {
        self.bottom
    }
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn as_u64(&self) -> u64 {
        self.top.as_u64()
    }
}
