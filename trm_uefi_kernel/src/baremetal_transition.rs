//! ExitBootServices transition: the point at which firmware hands off and our
//! own GDT/IDT/TSS can legally be loaded.
//!
//! BEFORE this call the firmware owns the descriptor tables; reloading them
//! would break firmware callbacks. AFTER it, boot services are gone: no console,
//! no file I/O, no return. This is a ONE-WAY transition and the caller must
//! confirm it.

use r_efi::efi;

/// Memory map buffer size to start with.
const INITIAL_MAP_BYTES: usize = 4096;

/// Exit UEFI boot services.
///
/// Returns true only when the firmware actually released control. On false the
/// caller is still in boot services and may continue normally.
pub unsafe fn exit_boot_services(image_handle: efi::Handle, bs: *mut efi::BootServices) -> bool {
    let b = &*bs;
    let mut size = INITIAL_MAP_BYTES;
    let mut map_key: usize = 0;
    let mut desc_size: usize = 0;
    let mut desc_ver: u32 = 0;
    let mut map: *mut core::ffi::c_void = core::ptr::null_mut();

    let mut st = (b.allocate_pool)(efi::LOADER_DATA, size, &mut map);
    if st.is_error() { return false; }

    // GetMemoryMap reports the required size when the buffer is too small, so
    // grow and retry rather than guessing.
    loop {
        st = (b.get_memory_map)(
            &mut size,
            map as *mut efi::MemoryDescriptor,
            &mut map_key,
            &mut desc_size,
            &mut desc_ver,
        );
        if st == efi::Status::SUCCESS { break; }
        let _ = (b.free_pool)(map);
        size += INITIAL_MAP_BYTES;
        st = (b.allocate_pool)(efi::LOADER_DATA, size, &mut map);
        if st.is_error() { return false; }
    }

    // Exit. The firmware invalidates the map key if anything allocated since we
    // read it, so on failure refresh the map and retry once - the documented
    // two-attempt pattern.
    st = (b.exit_boot_services)(image_handle, map_key);
    if st.is_error() {
        let mut s2 = size;
        let mut k2: usize = 0;
        let r = (b.get_memory_map)(&mut s2, map as *mut efi::MemoryDescriptor, &mut k2, &mut desc_size, &mut desc_ver);
        if r.is_error() { return false; }
        st = (b.exit_boot_services)(image_handle, k2);
    }
    st == efi::Status::SUCCESS
}

/// Load our own GDT, IDT and TSS, then enable interrupts.
///
/// Only legal once `exit_boot_services` has returned true.
pub unsafe fn install_tables(gdt: &crate::apic_core_binding::GdtPointer, idt: &crate::interrupt_stubs::IdtPointer, tss_selector: u16) {
    crate::interrupt_stubs::load_gdt(gdt);
    crate::interrupt_stubs::load_idt(idt);
    crate::interrupt_stubs::load_tss(tss_selector);
    crate::interrupt_stubs::enable_interrupts();
}
