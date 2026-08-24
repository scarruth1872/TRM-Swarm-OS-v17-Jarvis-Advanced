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
        let bs = BOOT_SERVICES;
        if bs.is_null() {
            return ptr::null_mut();
        }
        let mut buf: *mut core::ffi::c_void = ptr::null_mut();
        let status = ((*bs).allocate_pool)(
            r_efi::efi::LOADER_DATA,
            layout.size(),
            &mut buf,
        );
        if status == r_efi::efi::Status::SUCCESS {
            buf as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr_: *mut u8, _layout: Layout) {
        let bs = BOOT_SERVICES;
        if !bs.is_null() && !ptr_.is_null() {
            ((*bs).free_pool)(ptr_ as *mut core::ffi::c_void);
        }
    }
}

#[global_allocator]
static ALLOCATOR: EfiAllocator = EfiAllocator;
