#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// 新建，还未准备好运行
    New,
    /// 就绪，等待被调度执行
    Ready,
    /// 正在执行
    Running,
    /// 阻塞，等待事件（如I/O）
    Blocked,
    /// 终止，等待资源回收
    Terminated,
}