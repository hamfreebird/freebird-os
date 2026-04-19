pub mod builder;
pub mod context;
pub mod error;
pub mod loader;
pub mod memory;
pub mod process;
pub mod scheduler;
pub mod stack;
pub mod state;
pub mod switch_asm;
pub mod syscall;

pub use self::{
    builder::ProcessBuilder,
    context::ProcessContext,
    loader::{LoaderError, RawBinary, UserProgramLoader, demo_user_program_loading},
    memory::{MemoryPermissions, MemoryRegion},
    process::{Process, ProcessId},
    scheduler::{Scheduler, setup_scheduler_timer},
    stack::KernelStack,
    state::ProcessState,
    syscall::{SyscallNumber, syscall_handler},
};
use crate::println;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

/// 初始化进程系统
pub fn init() {
    println!("Initializing process system...");
    let idle_pid = SCHEDULER
        .lock()
        .create_idle_process()
        .expect("Failed to create idle process");
    println!("Created idle process with PID: {}", idle_pid.0);
    setup_scheduler_timer();
}

pub fn current_process() -> Option<Process> {
    SCHEDULER.lock().current_process().cloned()
}

pub fn process_exit(exit_code: i32) -> ! {
    let mut scheduler = SCHEDULER.lock();
    if let Some(current_pid) = scheduler.current_pid {
        if let Some(process) = scheduler.processes.get_mut(&current_pid) {
            process.exit_code = Some(exit_code);
            process.state = ProcessState::Terminated;
            println!("Process {} exited with code {}", current_pid.0, exit_code);
        }
    }
    scheduler.schedule();
    crate::hlt_loop();
}
