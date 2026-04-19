//! 用户程序加载器模块
//!
//! 负责将用户程序加载到内存中，设置内存映射，并创建可调度的进程。
//! 当前支持简单的原始二进制格式，未来可扩展支持ELF等格式。

use crate::process::builder::ProcessBuilder;
use crate::process::error::ProcessError;
use crate::process::memory::MemoryPermissions;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use x86_64::VirtAddr;

/// 加载器错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderError {
    /// 无效的程序格式
    InvalidFormat,
    /// 内存分配失败
    MemoryAllocationFailed,
    /// 内存映射失败
    MemoryMappingFailed,
    /// 程序太大，无法加载
    ProgramTooLarge,
    /// 无效的入口点
    InvalidEntryPoint,
    /// 页表设置失败
    PageTableSetupFailed,
    /// 不支持的程序格式
    UnsupportedFormat,
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoaderError::InvalidFormat => write!(f, "Invalid program format"),
            LoaderError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            LoaderError::MemoryMappingFailed => write!(f, "Memory mapping failed"),
            LoaderError::ProgramTooLarge => write!(f, "Program too large"),
            LoaderError::InvalidEntryPoint => write!(f, "Invalid entry point"),
            LoaderError::PageTableSetupFailed => write!(f, "Page table setup failed"),
            LoaderError::UnsupportedFormat => write!(f, "Unsupported program format"),
        }
    }
}

impl From<LoaderError> for ProcessError {
    fn from(error: LoaderError) -> Self {
        match error {
            LoaderError::MemoryAllocationFailed => ProcessError::MemoryAllocationFailed,
            LoaderError::MemoryMappingFailed => ProcessError::PageMappingFailed,
            _ => ProcessError::InvalidArgument,
        }
    }
}

/// 程序段信息
#[derive(Debug, Clone)]
pub struct ProgramSegment {
    /// 虚拟地址起始位置
    pub virtual_address: VirtAddr,
    /// 段大小（字节）
    pub size: usize,
    /// 段数据
    pub data: Vec<u8>,
    /// 访问权限
    pub permissions: MemoryPermissions,
    /// 是否可执行
    pub executable: bool,
}

/// 程序头信息
#[derive(Debug, Clone)]
pub struct ProgramHeader {
    /// 程序入口点（虚拟地址）
    pub entry_point: VirtAddr,
    /// 代码段
    pub code_segment: Option<ProgramSegment>,
    /// 数据段
    pub data_segment: Option<ProgramSegment>,
    /// 所需的栈大小
    pub stack_size: usize,
    /// 程序名称
    pub name: &'static str,
}

/// 原始二进制程序信息
#[derive(Debug, Clone)]
pub struct RawBinary {
    /// 程序数据
    pub data: Vec<u8>,
    /// 加载地址（虚拟地址）
    pub load_address: VirtAddr,
    /// 入口点偏移（相对于加载地址）
    pub entry_offset: usize,
    /// 程序名称
    pub name: &'static str,
}

impl RawBinary {
    /// 创建一个新的原始二进制程序
    pub fn new(
        data: Vec<u8>,
        load_address: VirtAddr,
        entry_offset: usize,
        name: &'static str,
    ) -> Self {
        RawBinary {
            data,
            load_address,
            entry_offset,
            name,
        }
    }

    /// 从静态数据创建原始二进制程序
    pub fn from_static(
        data: &'static [u8],
        load_address: VirtAddr,
        entry_offset: usize,
        name: &'static str,
    ) -> Self {
        RawBinary {
            data: data.to_vec(),
            load_address,
            entry_offset,
            name,
        }
    }

    /// 转换为程序头信息
    pub fn to_program_header(&self) -> ProgramHeader {
        let entry_point = self.load_address + self.entry_offset as u64;

        // 创建单个段，包含所有数据（代码+数据）
        let segment = ProgramSegment {
            virtual_address: self.load_address,
            size: self.data.len(),
            data: self.data.clone(),
            permissions: MemoryPermissions {
                readable: true,
                writable: true,
                executable: true,
            },
            executable: true,
        };

        ProgramHeader {
            entry_point,
            code_segment: Some(segment),
            data_segment: None,
            stack_size: 4096 * 4, // 默认16KB栈
            name: self.name,
        }
    }
}

/// 用户程序加载器
pub struct UserProgramLoader;

impl UserProgramLoader {
    /// 创建一个新的用户程序加载器
    pub fn new() -> Self {
        UserProgramLoader
    }

    /// 加载原始二进制程序并创建进程
    pub fn load_raw_binary(&self, binary: &RawBinary) -> Result<ProcessBuilder, LoaderError> {
        let header = binary.to_program_header();
        self.load_from_header(&header)
    }

    /// 从程序头信息加载程序
    pub fn load_from_header(&self, header: &ProgramHeader) -> Result<ProcessBuilder, LoaderError> {
        // 验证入口点
        if header.entry_point.as_u64() == 0 {
            return Err(LoaderError::InvalidEntryPoint);
        }

        // 检查程序大小（简单限制）
        let total_size: usize = header.code_segment.as_ref().map(|s| s.size).unwrap_or(0)
            + header.data_segment.as_ref().map(|s| s.size).unwrap_or(0);

        if total_size > 1024 * 1024 {
            // 1MB限制
            return Err(LoaderError::ProgramTooLarge);
        }

        // 创建进程构建器
        let builder = ProcessBuilder::new(header.entry_point, header.name);

        // 设置内存大小（程序大小 + 栈大小）
        let total_memory = total_size + header.stack_size;
        let builder = builder.with_memory_size(total_memory);

        // 注意：实际的程序加载和内存映射需要在进程创建后完成
        // 当前实现中，ProcessBuilder只设置基本信息
        // 实际的内存复制和页表设置需要在进程初始化时完成

        Ok(builder)
    }

    /// 创建简单的测试程序
    pub fn create_test_program(name: &'static str) -> RawBinary {
        // 创建一个简单的汇编程序，执行以下操作：
        // 1. 调用getpid系统调用
        // 2. 调用yield系统调用
        // 3. 无限循环

        // 注意：这是x86_64机器码，使用原始字节
        // 由于我们还没有完整的汇编器，这里使用占位符数据
        // 实际实现中应该使用真正的汇编代码

        let test_code = vec![
            0x48, 0xC7, 0xC0, 0x06, 0x00, 0x00, 0x00, // mov rax, 6 (getpid系统调用号)
            0x0F, 0x05, // syscall
            0x48, 0xC7, 0xC0, 0x07, 0x00, 0x00, 0x00, // mov rax, 7 (yield系统调用号)
            0x0F, 0x05, // syscall
            0xEB, 0xF6, // jmp $-8 (无限循环)
        ];

        RawBinary::new(
            test_code,
            VirtAddr::new(0x400000), // 用户空间典型加载地址
            0,                       // 从开始处执行
            name,
        )
    }

    /// 创建简单的退出程序
    pub fn create_exit_program(name: &'static str, exit_code: u8) -> RawBinary {
        // 创建调用exit系统调用的程序
        let exit_code = exit_code as u32;

        let exit_code_bytes = vec![
            0x48,
            0xC7,
            0xC0,
            0x03,
            0x00,
            0x00,
            0x00, // mov rax, 3 (exit系统调用号)
            0x48,
            0xC7,
            0xC3, // mov rbx, exit_code
            (exit_code & 0xFF) as u8,
            ((exit_code >> 8) & 0xFF) as u8,
            ((exit_code >> 16) & 0xFF) as u8,
            ((exit_code >> 24) & 0xFF) as u8,
            0x0F,
            0x05, // syscall
        ];

        RawBinary::new(exit_code_bytes, VirtAddr::new(0x400000), 0, name)
    }
}

/// 演示用户程序加载功能
pub fn demo_user_program_loading() {
    use crate::println;

    println!("\n=== User Program Loading Demonstration ===");

    // 创建加载器
    let loader = UserProgramLoader::new();

    // 创建测试程序
    let test_program = UserProgramLoader::create_test_program("test_user_program");
    println!("Created test program: {}", test_program.name);
    println!(
        "  Load address: 0x{:016x}",
        test_program.load_address.as_u64()
    );
    println!(
        "  Entry point: 0x{:016x}",
        test_program.load_address.as_u64() + test_program.entry_offset as u64
    );
    println!("  Program size: {} bytes", test_program.data.len());

    // 创建退出程序
    let exit_program = UserProgramLoader::create_exit_program("exit_program", 42);
    println!("\nCreated exit program: {}", exit_program.name);
    println!("  Exit code: 42");

    // 尝试加载测试程序
    match loader.load_raw_binary(&test_program) {
        Ok(_builder) => {
            println!("\nSuccessfully loaded program into process builder");
            println!("  Process name: {}", test_program.name);
            println!(
                "  Entry point: 0x{:016x}",
                test_program.load_address.as_u64() + test_program.entry_offset as u64
            );

            // 在实际系统中，这里会使用builder.spawn()来创建进程
            println!("  (In a real system, would call builder.spawn() here)");
        }
        Err(e) => {
            println!("\nFailed to load program: {}", e);
        }
    }

    println!("\nLoading process overview:");
    println!("  1. Parse program format (raw binary, ELF, etc.)");
    println!("  2. Allocate virtual memory for code, data, and stack");
    println!("  3. Set up page table entries with appropriate permissions");
    println!("  4. Copy program data into allocated memory");
    println!("  5. Set up initial stack with arguments and environment");
    println!("  6. Create process control block (PCB)");
    println!("  7. Add process to scheduler's ready queue");

    println!("\n=== End User Program Loading Demonstration ===");
}
