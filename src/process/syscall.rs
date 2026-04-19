use crate::println;
use crate::process::{ProcessContext, SCHEDULER, current_process};

#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum SyscallNumber {
    /// 创建新进程
    Fork = 1,
    /// 执行程序
    Exec = 2,
    /// 退出进程
    Exit = 3,
    /// 等待子进程
    Wait = 4,
    /// 进程休眠
    Sleep = 5,
    /// 获取进程ID
    GetPid = 6,
    /// 让出CPU
    Yield = 7,
}

/// 系统调用处理入口
pub extern "C" fn syscall_handler(context: &mut ProcessContext) {
    let syscall_num = context.rax as u64;
    let arg1 = context.rbx;
    let _arg2 = context.rcx;
    let _arg3 = context.rdx;

    match syscall_num {
        n if n == SyscallNumber::Exit as u64 => {
            let exit_code = arg1 as i32;
            exit_handler(exit_code);
        }
        n if n == SyscallNumber::GetPid as u64 => {
            getpid_handler(context);
        }
        n if n == SyscallNumber::Yield as u64 => {
            yield_handler(context);
        }
        n if n == SyscallNumber::Sleep as u64 => {
            let milliseconds = arg1 as u64;
            sleep_handler(context, milliseconds);
        }
        // 处理其他系统调用...
        _ => {
            // 无效系统调用
            context.rax = !0;
        }
    }
}

/// 退出进程处理器
fn exit_handler(exit_code: i32) -> ! {
    println!("Process calling exit with code: {}", exit_code);
    // 使用完整的 crate 路径调用 process_exit
    crate::process::process_exit(exit_code);
}

/// 获取进程ID处理器
fn getpid_handler(context: &mut ProcessContext) {
    if let Some(process) = current_process() {
        context.rax = process.pid.as_u64();
    } else {
        context.rax = 0;
    }
}

/// 让出CPU处理器
fn yield_handler(context: &mut ProcessContext) {
    use crate::process::ProcessState;

    println!("Process calling yield()");

    // 尝试切换到下一个进程（演示版本，不实际切换）
    let mut scheduler = SCHEDULER.lock();

    if let Some(current_pid) = scheduler.current_pid {
        // 将当前进程放回就绪队列
        if let Some(process) = scheduler.processes.get_mut(&current_pid) {
            process.state = ProcessState::Ready;
            // 使用调度器方法添加进程到就绪队列
            scheduler.add_to_ready_queue(current_pid);
        }

        // 调度下一个进程
        if let Some(next_pid) = scheduler.schedule() {
            // 返回成功代码
            context.rax = 0;

            // 执行上下文切换（在演示中跳过，只打印消息）
            println!(
                "  [Demo] Would switch to process PID: {}",
                next_pid.as_u64()
            );
            // let _ = scheduler.switch_to(next_pid);
            // 注意：switch_to 不会返回，除非发生错误
        } else {
            // 没有可调度的进程
            context.rax = 1;
        }
    } else {
        // 没有当前进程
        context.rax = 2;
    }
}

/// 休眠处理器
fn sleep_handler(context: &mut ProcessContext, milliseconds: u64) {
    use crate::process::ProcessState;

    println!("Process calling sleep({} ms)", milliseconds);

    let mut scheduler = SCHEDULER.lock();

    if let Some(current_pid) = scheduler.current_pid {
        if let Some(process) = scheduler.processes.get_mut(&current_pid) {
            // 将进程状态设置为阻塞
            process.state = ProcessState::Blocked;

            // TODO: 实现实际的计时器机制
            // 目前只是简单地将进程移出就绪队列
            // 在实际实现中，需要设置唤醒时间

            // 返回成功代码
            context.rax = 0;

            // 调度下一个进程（演示版本，不实际切换）
            if let Some(next_pid) = scheduler.schedule() {
                println!(
                    "  [Demo] Sleep: Would switch to process PID: {}",
                    next_pid.as_u64()
                );
                // let _ = scheduler.switch_to(next_pid);
            }
        } else {
            context.rax = 1; // 进程不存在
        }
    } else {
        context.rax = 2; // 没有当前进程
    }
}

/// 从用户空间调用yield的辅助函数
pub fn sys_yield() {
    // 模拟系统调用：将系统调用号放入RAX，然后调用syscall指令
    // 在内核空间中，我们可以直接调用处理器
    let mut dummy_context = ProcessContext::new();
    dummy_context.rax = SyscallNumber::Yield as u64;
    yield_handler(&mut dummy_context);
}

/// 从用户空间调用sleep的辅助函数
pub fn sys_sleep(milliseconds: u64) {
    let mut dummy_context = ProcessContext::new();
    dummy_context.rax = SyscallNumber::Sleep as u64;
    dummy_context.rbx = milliseconds;
    sleep_handler(&mut dummy_context, milliseconds);
}

/// 从用户空间调用exit的辅助函数
pub fn sys_exit(exit_code: i32) -> ! {
    exit_handler(exit_code);
}

/// 从用户空间调用getpid的辅助函数
pub fn sys_getpid() -> u64 {
    let mut dummy_context = ProcessContext::new();
    dummy_context.rax = SyscallNumber::GetPid as u64;
    getpid_handler(&mut dummy_context);
    dummy_context.rax
}
