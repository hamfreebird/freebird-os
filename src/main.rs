#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(freebird_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use freebird_os::println;
use freebird_os::task::{Task, executor::Executor, keyboard};
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use freebird_os::allocator;
    use freebird_os::memory::{self, BootInfoFrameAllocator};

    println!("Welcome to use Freebird OS");
    freebird_os::init();
    freebird_os::process::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    // 创建测试进程
    create_test_processes();

    // 打印所有进程信息
    print_process_info();

    // 启动异步执行器处理键盘输入
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.spawn(Task::new(process_demonstration()));

    // executor.run() 不会返回
    executor.run();
}

/// 创建测试进程
fn create_test_processes() {
    use freebird_os::process::ProcessBuilder;
    use x86_64::VirtAddr;

    println!("Creating test processes...");

    // 创建进程1：简单的计数器进程
    let proc1_entry = VirtAddr::new(0x10000);
    let builder1 = ProcessBuilder::new(proc1_entry, "counter_process").with_priority(1);

    match builder1.spawn() {
        Ok(pid) => println!("Created counter process with PID: {}", pid.as_u64()),
        Err(e) => println!("Failed to create counter process: {:?}", e),
    }

    // 创建进程2：空闲进程（除了idle进程外的另一个）
    let proc2_entry = VirtAddr::new(0x20000);
    let builder2 = ProcessBuilder::new(proc2_entry, "idle_worker")
        .with_priority(0)
        .with_argument(42); // 示例参数

    match builder2.spawn() {
        Ok(pid) => println!("Created idle worker process with PID: {}", pid.as_u64()),
        Err(e) => println!("Failed to create idle worker process: {:?}", e),
    }
}

/// 打印所有进程信息
fn print_process_info() {
    use freebird_os::process::SCHEDULER;

    println!("\n=== Process Information ===");
    let scheduler = SCHEDULER.lock();

    for (pid, process) in scheduler.processes.iter() {
        println!("PID: {}", pid.as_u64());
        println!("  Name: {}", process.name);
        println!("  State: {:?}", process.state);
        println!("  Priority: {}", process.priority);
        println!("  Parent PID: {:?}", process.parent_pid.map(|p| p.as_u64()));
        println!("  Exit Code: {:?}", process.exit_code);
        println!("  RIP: 0x{:016x}", process.context.rip);
        println!("  RSP: 0x{:016x}", process.context.rsp);
        println!("  CR3: 0x{:016x}", process.context.cr3);
        println!();
    }

    if let Some(current_pid) = scheduler.current_pid {
        println!("Current running process: PID {}", current_pid.as_u64());
    } else {
        println!("No process is currently running");
    }
    println!("Total processes: {}", scheduler.processes.len());
    println!("=== End Process Information ===\n");
}

/// 进程演示任务
async fn process_demonstration() {
    use freebird_os::println;
    use freebird_os::process::syscall::*;

    println!("Starting process demonstration...");

    // 演示系统调用
    println!("System call demonstration:");

    // 模拟系统调用处理
    let mut dummy_context = freebird_os::process::ProcessContext::new();

    // 测试 getpid 系统调用
    dummy_context.rax = SyscallNumber::GetPid as u64;
    syscall_handler(&mut dummy_context);
    println!("  getpid() returned: {}", dummy_context.rax);

    // 测试 exit 系统调用
    println!("  exit(42) would terminate the process");

    // 测试 yield 系统调用
    dummy_context.rax = SyscallNumber::Yield as u64;
    syscall_handler(&mut dummy_context);
    println!("  yield() would yield CPU to another process");

    // 演示协作式多任务
    println!("\nCooperative multitasking demonstration:");
    println!("  In a real system, processes would call yield() to voluntarily");
    println!("  give up CPU time to other processes.");
    println!("  The scheduler would then select the next ready process to run.");
    println!("  This enables fair sharing of CPU resources among processes.");

    // 演示简单的上下文切换概念
    println!("\n  Context switch overview:");
    println!("    1. Save current process registers");
    println!("    2. Load next process registers");
    println!("    3. Update CR3 (page table) if needed");
    println!("    4. Resume execution at new process's RIP");

    // 演示用户程序加载
    println!("\n=== User Program Loading Demonstration ===");
    use freebird_os::process::demo_user_program_loading;
    demo_user_program_loading();

    println!("Process demonstration completed");
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    freebird_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    freebird_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
