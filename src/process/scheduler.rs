use crate::process::error::ProcessError;
use crate::process::switch_asm::switch_context;
use crate::process::{Process, ProcessContext, ProcessId, ProcessState};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec;
use alloc::vec::Vec;
use x86_64::{PhysAddr, VirtAddr};

pub struct Scheduler {
    /// 就绪队列（按优先级分组）
    ready_queue: VecDeque<ProcessId>,
    /// 阻塞队列（等待I/O等）
    blocked_queue: VecDeque<ProcessId>,
    /// 所有进程的映射表
    pub processes: BTreeMap<ProcessId, Process>,
    /// 当前运行的进程
    pub current_pid: Option<ProcessId>,
    /// 空闲进程（当没有其他进程运行时执行）
    idle_process: ProcessId,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            ready_queue: VecDeque::new(),
            blocked_queue: VecDeque::new(),
            processes: BTreeMap::new(),
            current_pid: None,
            idle_process: ProcessId(0),
        }
    }

    /// 添加进程到就绪队列
    pub fn add_to_ready_queue(&mut self, pid: ProcessId) {
        self.ready_queue.push_back(pid);
    }

    /// 检查就绪队列是否包含指定进程
    pub fn ready_queue_contains(&self, pid: ProcessId) -> bool {
        self.ready_queue.contains(&pid)
    }

    /// 创建新进程
    pub fn create_process(
        &mut self,
        entry_point: VirtAddr,
        name: &'static str,
    ) -> Result<ProcessId, ProcessError> {
        // 分配PID
        let pid = ProcessId::new();

        // 创建内核栈
        let kernel_stack = self.allocate_kernel_stack()?;

        // 初始化进程上下文
        let mut context = ProcessContext::new();
        context.rip = entry_point.as_u64();
        context.rsp = kernel_stack.as_u64();

        // 创建进程
        let process = Process {
            pid,
            state: ProcessState::Ready,
            context,
            kernel_stack,
            user_stack: None,
            page_table: Self::clone_current_page_table(),
            priority: 1,
            parent_pid: self.current_pid,
            exit_code: None,
            name,
            memory_regions: Vec::new(),
        };

        // 添加到进程表
        self.processes.insert(pid, process);
        self.ready_queue.push_back(pid);

        Ok(pid)
    }

    /// 调度下一个进程运行
    pub fn schedule(&mut self) -> Option<ProcessId> {
        if let Some(current_pid) = self.current_pid.take() {
            // 保存当前进程状态
            if let Some(process) = self.processes.get_mut(&current_pid) {
                if process.state == ProcessState::Running {
                    process.state = ProcessState::Ready;
                    self.ready_queue.push_back(current_pid);
                }
            }
        }

        // 选择下一个进程（简单轮转调度）
        if let Some(next_pid) = self.ready_queue.pop_front() {
            if let Some(process) = self.processes.get_mut(&next_pid) {
                process.state = ProcessState::Running;
                self.current_pid = Some(next_pid);
                return Some(next_pid);
            }
        }

        // 没有就绪进程，返回空闲进程
        Some(self.idle_process)
    }

    pub fn create_idle_process(&mut self) -> Result<ProcessId, ProcessError> {
        // 创建空闲进程，当没有其他进程运行时执行
        let pid = ProcessId::new();
        let kernel_stack = self.allocate_kernel_stack()?;
        let mut context = ProcessContext::new();
        context.rip = Self::idle_task as *const () as u64;
        context.rsp = kernel_stack.as_u64();

        let process = Process {
            pid,
            state: ProcessState::Ready,
            context,
            kernel_stack,
            user_stack: None,
            page_table: Self::clone_current_page_table(),
            priority: 0,
            parent_pid: None,
            exit_code: None,
            name: "idle",
            memory_regions: vec![],
        };
        self.processes.insert(pid, process);
        self.idle_process = pid;
        Ok(pid)
    }

    extern "C" fn idle_task() -> ! {
        use x86_64::instructions::hlt;
        loop {
            hlt();
        } // 使用HLT指令等待中断
    }

    fn allocate_kernel_stack(&self) -> Result<crate::process::stack::KernelStack, ProcessError> {
        const KERNEL_STACK_SIZE: usize = 4096 * 8; // 32KB
        crate::process::stack::KernelStack::allocate(KERNEL_STACK_SIZE)
    }

    fn clone_current_page_table() -> PhysAddr {
        use x86_64::registers::control::Cr3;
        let (current_frame, _) = Cr3::read();
        current_frame.start_address()
    }

    pub fn wake_process(&mut self, pid: ProcessId) -> Result<(), ProcessError> {
        if let Some(process) = self.processes.get_mut(&pid) {
            if process.state == ProcessState::Blocked {
                process.state = ProcessState::Ready;
                self.ready_queue.push_back(pid);
                Ok(())
            } else {
                Err(ProcessError::InvalidProcessState)
            }
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }

    pub fn get_process_mut(&mut self, pid: ProcessId) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }

    pub fn get_process(&self, pid: ProcessId) -> Option<&Process> {
        self.processes.get(&pid)
    }

    pub fn current_process(&self) -> Option<&Process> {
        self.current_pid.and_then(|pid| self.processes.get(&pid))
    }

    pub fn switch_to(&mut self, next_pid: ProcessId) -> Result<(), ProcessError> {
        use core::ptr;

        // 检查下一个进程是否存在
        if !self.processes.contains_key(&next_pid) {
            return Err(ProcessError::ProcessNotFound);
        }

        // 如果下一个进程就是当前进程，不需要切换
        if let Some(current_pid) = self.current_pid {
            if current_pid == next_pid {
                return Ok(());
            }
        }

        // 准备上下文指针
        let mut current_context_ptr: *mut ProcessContext = ptr::null_mut();
        let mut next_context_ptr: *mut ProcessContext = ptr::null_mut();

        // 第一步：处理当前进程（如果存在）
        if let Some(current_pid) = self.current_pid {
            if let Some(current_process) = self.processes.get_mut(&current_pid) {
                // 保存当前进程的栈指针到其上下文中
                unsafe {
                    core::arch::asm!("mov [{}], rsp", in(reg) &mut current_process.context.rsp);
                }
                current_context_ptr = &mut current_process.context;

                // 更新当前进程状态（如果不是空闲进程）
                if current_pid != self.idle_process {
                    current_process.state = ProcessState::Ready;
                    if !self.ready_queue.contains(&current_pid) {
                        self.ready_queue.push_back(current_pid);
                    }
                }
            }
        }

        // 第二步：处理下一个进程
        if let Some(next_process) = self.processes.get_mut(&next_pid) {
            next_process.state = ProcessState::Running;
            next_context_ptr = &mut next_process.context;
        }

        // 更新当前进程ID
        self.current_pid = Some(next_pid);

        // 第三步：执行上下文切换
        unsafe {
            // 调用汇编实现的上下文切换函数
            switch_context(
                &mut current_context_ptr as *mut *mut ProcessContext,
                next_context_ptr,
            );
        }

        // 这段代码不会执行，因为上下文切换会跳转到新进程
        unreachable!();
    }
}

/// 设置调度器定时器
pub fn setup_scheduler_timer() {
    use x86_64::instructions::port::Port;

    // 配置 PIT 定时器（Programmable Interval Timer）
    // 频率：100Hz (10ms间隔)
    const PIT_FREQUENCY: u32 = 1193182;
    const TIMER_FREQUENCY: u32 = 100; // 100Hz
    let divisor = PIT_FREQUENCY / TIMER_FREQUENCY;

    unsafe {
        // 设置PIT控制字
        // 0x36 = 通道0，模式3，二进制
        Port::<u8>::new(0x43).write(0x36u8);

        // 设置除数
        let mut divisor_port = Port::<u8>::new(0x40);
        divisor_port.write((divisor & 0xFF) as u8);
        divisor_port.write(((divisor >> 8) & 0xFF) as u8);

        // 取消屏蔽PIC上的定时器中断（IRQ0）
        // 主PIC的数据端口是0x21
        let mut pic_master_data = Port::<u8>::new(0x21);
        let current_mask = pic_master_data.read();
        // 清除IRQ0位（位0）以启用定时器中断
        let new_mask = current_mask & !(1 << 0);
        pic_master_data.write(new_mask);
    }
}
