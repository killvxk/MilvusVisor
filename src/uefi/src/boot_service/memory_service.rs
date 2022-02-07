//!
//! Memory Allocation Services of Boot Service
//!

use super::super::EfiStatus;
use super::EfiBootServices;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
#[repr(C)]
#[allow(dead_code)]
pub enum EfiMemoryType {
    EfiReservedMemoryType,
    EfiLoaderCode,
    EfiLoaderData,
    EfiBootServicesCode,
    EfiBootServicesData,
    EfiRuntimeServicesCode,
    EfiRuntimeServicesData,
    EfiConventionalMemory,
    EfiUnusableMemory,
    EfiACPIReclaimMemory,
    EfiACPIMemoryNVS,
    EfiMemoryMappedIO,
    EfiMemoryMappedIOPortSpace,
    EfiPalCode,
    EfiPersistentMemory,
    EfiMaxMemoryType,
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u64)]
#[allow(dead_code)]
pub enum EfiMemoryAttribute {
    EfiMemoryUc = 0x0000000000000001,
    EfiMemoryWc = 0x0000000000000002,
    EfiMemoryWt = 0x0000000000000004,
    EfiMemoryWb = 0x0000000000000008,
    EfiMemoryUce = 0x0000000000000010,
    EfiMemoryWp = 0x0000000000001000,
    EfiMemoryRp = 0x0000000000002000,
    EfiMemoryXp = 0x0000000000004000,
    EfiMemoryNv = 0x0000000000008000,
    EfiMemoryMoreReliable = 0x0000000000010000,
    EfiMemoryRo = 0x0000000000020000,
    EfiMemorySp = 0x0000000000040000,
    EfiMemoryCpuCrypto = 0x0000000000080000,
    EfiMemoryRuntime = 0x8000000000000000,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
#[repr(C)]
#[allow(dead_code)]
pub enum EfiAllocateType {
    AllocateAnyPages,
    AllocateMaxAddress,
    AllocateAddress,
    MaxAllocateType,
}

#[derive(Clone)]
#[repr(C)]
pub struct EfiMemoryDescriptor {
    pub memory_type: EfiMemoryType,
    pub physical_start: usize,
    pub virtual_start: usize,
    pub number_of_pages: u64,
    pub attribute: u64,
}

#[derive(Clone, Debug)]
pub struct MemoryMapInfo {
    pub key: usize,
    pub num_of_entries: usize,
    pub actual_descriptor_size: usize,
    pub descriptor_address: usize,
}

pub fn alloc_pool(b_s: *const EfiBootServices, size: usize) -> Result<usize, EfiStatus> {
    let mut address: usize = 0usize;
    let status = unsafe {
        ((*b_s).allocate_pool)(EfiMemoryType::EfiLoaderData, size, &mut address as *mut _)
    };
    if status != EfiStatus::EfiSuccess {
        return Err(status);
    }
    Ok(address)
}

pub fn free_pool(b_s: *const EfiBootServices, address: usize) -> Result<(), EfiStatus> {
    let status = unsafe { ((*b_s).free_pool)(address) };
    if status != EfiStatus::EfiSuccess {
        return Err(status);
    }
    Ok(())
}

/// サイズ(ページ指定)を満たす中で最高位のアドレスを確保する。
///
/// [`border_address`]以下で条件に合うアドレスを確保する。
pub fn alloc_highest_memory(
    b_s: *const EfiBootServices,
    pages: usize,
    border_address: usize,
) -> Result<usize, EfiStatus> {
    let mut memory_address = border_address;
    let status = unsafe {
        ((*b_s).allocate_pages)(
            EfiAllocateType::AllocateMaxAddress,
            EfiMemoryType::EfiUnusableMemory,
            pages,
            &mut memory_address as *mut _,
        )
    };
    if status != EfiStatus::EfiSuccess {
        return Err(status);
    }
    Ok(memory_address)
}

/// メモリマップを取得する
///
/// メモリマップを取得する際に、[`alloc_pool`]を使用し、その中にメモリマップを格納する。
/// [`MemoryMapInfo::descriptor_address`]は[`free_pool`]で開放する。
pub fn get_memory_map(b_s: *const EfiBootServices) -> Result<MemoryMapInfo, EfiStatus> {
    let mut memory_map_size = 0;
    let mut map_key = 0usize;
    let mut actual_memory_descriptor_size = 0usize;
    let mut descriptor_version = 0u32;

    let result = unsafe {
        ((*b_s).get_memory_map)(
            &mut memory_map_size,
            core::ptr::null_mut(),
            &mut map_key,
            &mut actual_memory_descriptor_size,
            &mut descriptor_version,
        )
    };
    if result != EfiStatus::EfiBufferTooSmall {
        return Err(result);
    }
    /* MemoryMap may get bigger after alloc_pool */
    memory_map_size += actual_memory_descriptor_size << 2;
    let buffer = alloc_pool(b_s, memory_map_size)?;

    let result = unsafe {
        ((*b_s).get_memory_map)(
            &mut memory_map_size,
            buffer as *mut _,
            &mut map_key,
            &mut actual_memory_descriptor_size,
            &mut descriptor_version,
        )
    };
    if result != EfiStatus::EfiSuccess {
        let _ = free_pool(b_s, buffer);
        return Err(result);
    }
    return Ok(MemoryMapInfo {
        key: map_key,
        num_of_entries: memory_map_size / actual_memory_descriptor_size,
        actual_descriptor_size: actual_memory_descriptor_size,
        descriptor_address: buffer,
    });
}

impl core::fmt::Debug for EfiMemoryDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EfiMemoryDescriptor")
            .field("memory_type", &self.memory_type)
            .field(
                "physical_start",
                &format_args!("{:#X}", self.physical_start),
            )
            .field("virtual_start", &format_args!("{:#X}", self.virtual_start))
            .field("number_of_pages", &self.number_of_pages)
            .field("attribute", &format_args!("{:#X}", self.attribute))
            .finish()
    }
}