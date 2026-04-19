#[repr(C)]
#[derive(Debug, Clone)]
pub struct ProcessContext {
    // 通用寄存器
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // 栈指针和指令指针
    pub rsp: u64,
    pub rip: u64,

    // 标志寄存器
    pub rflags: u64,

    // 页表基址（CR3）
    pub cr3: u64,
}

impl ProcessContext {
    pub fn new() -> Self {
        ProcessContext {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rsp: 0, rip: 0,
            rflags: 0x200, // 启用中断标志
            cr3: 0,
        }
    }

    pub fn from_current() -> Self {
        // 使用内联汇编保存当前寄存器状态
        let mut context = Self::new();
        unsafe {
            core::arch::asm!(
            "mov {}, rax", out(reg) context.rax,
            // ... 保存所有寄存器
            options(nostack)
            );
            core::arch::asm!("mov {}, cr3", out(reg) context.cr3);
        }
        context
    }
}
