use x86_64::{PhysAddr, VirtAddr};

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// 虚拟地址起始
    pub virtual_start: VirtAddr,
    /// 物理地址起始
    pub physical_start: PhysAddr,
    /// 区域大小（字节）
    pub size: usize,
    /// 访问权限：可读、可写、可执行
    pub permissions: MemoryPermissions,
    /// 是否已分配
    pub allocated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryPermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}