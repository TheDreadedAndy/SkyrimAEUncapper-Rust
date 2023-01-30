//!
//! @file safe.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes the SKSE safe-write functions.
//! @bug No known bugs
//!

use core::ffi::c_int;

extern "system" {
    fn SKSE64_SafeWrite__virtual_protect__(
        addr: usize,
        size: usize,
        new_prot: u32,
        old_prot: *mut u32
    );
    fn SKSE64_SafeWrite__safe_write_jump__(src: usize, dst: usize) -> c_int;
    fn SKSE64_SafeWrite__safe_write_call__(src: usize, dst: usize) -> c_int;
}

/// Temporarily marks the given memory region for read/write, then calls the given fn.
pub unsafe fn use_region(
    addr: usize,
    size: usize,
    func: impl FnOnce()
) {
    const PAGE_EXECUTE_READWRITE: u32 = 0x40;
    let mut old_prot: u32 = 0;
    SKSE64_SafeWrite__virtual_protect__(addr, size, PAGE_EXECUTE_READWRITE, &mut old_prot);
    func();
    SKSE64_SafeWrite__virtual_protect__(addr, size, old_prot, &mut old_prot);
}

/// Writes a 5-byte jump to the given address.
pub unsafe fn write_jump(
    src: usize,
    dst: usize
) -> Result<(), ()> {
    if SKSE64_SafeWrite__safe_write_jump__(src, dst) >= 0 {
        Ok(())
    } else {
        Err(())
    }
}

/// Writes a 5-byte call to the given address.
pub unsafe fn write_call(
    src: usize,
    dst: usize
) -> Result<(), ()> {
    if SKSE64_SafeWrite__safe_write_call__(src, dst) >= 0 {
        Ok(())
    } else {
        Err(())
    }
}
