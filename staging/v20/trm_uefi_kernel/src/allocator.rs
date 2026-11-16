//! Global allocator backed by EFI Boot Services AllocatePool.
//! Required by ported intelligence modules (Vec/String/format!) in no_std UEFI.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// Boot Services pointer captured at efi_main entry. Must be set before any
/// allocation happens (i.e. first thing in efi_main).
static mut BOOT_SERVICES: *mut r_efi::efi::BootServices = ptr::null_mut();

/// Initialize the allocator with the EFI Boot Services table.
/// SAFETY: call exactly once, at the very start of efi_main, before any Vec/String use.
pub unsafe fn init(bs: *mut r_efi::efi::BootServices) {
    BOOT_SERVICES = bs;
}

struct EfiAllocator;

unsafe impl GlobalAlloc for EfiAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            return layout.align() as *mut u8;
        }

        let bs = BOOT_SERVICES;
        if bs.is_null() {
            return ptr::null_mut();
        }

        // Over-allocate to satisfy any alignment (e.g. 16, 32, 64-byte for SIMD)
        // and reserve space for the original allocation address.
        let align = layout.align().max(core::mem::size_of::<usize>());
        let total_size = layout.size() + align + core::mem::size_of::<usize>();

        let mut raw_buf: *mut core::ffi::c_void = ptr::null_mut();
        let status = ((*bs).allocate_pool)(
            r_efi::efi::LOADER_DATA,
            total_size,
            &mut raw_buf,
        );

        if status != r_efi::efi::Status::SUCCESS || raw_buf.is_null() {
            return ptr::null_mut();
        }

        let raw_addr = raw_buf as usize;
        let unaligned_addr = raw_addr + core::mem::size_of::<usize>();
        let mask = align - 1;
        let aligned_addr = (unaligned_addr + mask) & !mask;
        let aligned_ptr = aligned_addr as *mut u8;

        // Store original raw allocation pointer right before aligned buffer
        *((aligned_ptr as *mut usize).sub(1)) = raw_addr;

        aligned_ptr
    }

    unsafe fn dealloc(&self, ptr_: *mut u8, layout: Layout) {
        if layout.size() == 0 || ptr_.is_null() {
            return;
        }

        let bs = BOOT_SERVICES;
        if bs.is_null() {
            return;
        }

        let raw_addr = *((ptr_ as *mut usize).sub(1));
        if raw_addr != 0 {
            ((*bs).free_pool)(raw_addr as *mut core::ffi::c_void);
        }
    }
}

#[global_allocator]
static ALLOCATOR: EfiAllocator = EfiAllocator;

/// Returns true if the EFI BootServices allocator pool has been initialized.
pub fn is_allocator_initialized() -> bool {
    unsafe { !BOOT_SERVICES.is_null() }
}

/// Returns allocator state code (1 = active, 0 = uninitialized).
pub fn get_allocator_status() -> u64 {
    if is_allocator_initialized() { 1 } else { 0 }
}

