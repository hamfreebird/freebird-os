use crate::process::error::ProcessError;
use crate::process::{ProcessId, SCHEDULER};
use alloc::vec::Vec;
use x86_64::VirtAddr;

struct MemorySpace {
    size: usize,
}

impl MemorySpace {
    fn new(_size: usize) -> Result<Self, ProcessError> {
        Ok(MemorySpace { size: 4096 * 4 })
    }
}

struct ProcessStack;

impl ProcessStack {
    fn new() -> Result<Self, ProcessError> {
        Ok(ProcessStack)
    }
}

pub struct ProcessBuilder {
    entry_point: VirtAddr,
    name: &'static str,
    priority: u8,
    memory_size: usize,
    arguments: Vec<u64>,
    environment: Vec<&'static str>,
}

impl ProcessBuilder {
    /// 创建一个新的进程构建器
    pub fn new(entry_point: VirtAddr, name: &'static str) -> Self {
        ProcessBuilder {
            entry_point,
            name,
            priority: 1,
            memory_size: 4096 * 4, // 16KB 默认内存
            arguments: Vec::new(),
            environment: Vec::new(),
        }
    }

    /// 设置进程优先级
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// 设置进程内存大小
    pub fn with_memory_size(mut self, memory_size: usize) -> Self {
        self.memory_size = memory_size;
        self
    }

    /// 添加进程参数
    pub fn with_argument(mut self, argument: u64) -> Self {
        self.arguments.push(argument);
        self
    }

    /// 添加环境变量
    pub fn with_environment(mut self, env: &'static str) -> Self {
        self.environment.push(env);
        self
    }

    /// 复制参数到用户空间（占位符实现）
    fn copy_arguments_to_user_space(&self, _pid: ProcessId) -> Result<(), ProcessError> {
        // TODO: 实现真正的参数复制到用户空间
        // 目前还没有用户空间内存管理，所以返回成功
        Ok(())
    }

    /// 创建并启动新进程
    pub fn spawn(self) -> Result<ProcessId, ProcessError> {
        let mut scheduler = SCHEDULER.lock();

        // 1. 创建进程结构
        let pid = scheduler.create_process(self.entry_point, self.name)?;

        // 2. 分配内存空间
        let _memory_space = MemorySpace::new(self.memory_size)?;

        // 3. 设置初始堆栈
        let _stack = ProcessStack::new()?;

        // 4. 复制参数到用户空间
        self.copy_arguments_to_user_space(pid)?;

        // 5. 设置入口点
        let process = scheduler.get_process_mut(pid).unwrap();
        process.context.rip = self.entry_point.as_u64();

        // 6. 添加到就绪队列
        let _ = scheduler.wake_process(pid);

        Ok(pid)
    }
}
