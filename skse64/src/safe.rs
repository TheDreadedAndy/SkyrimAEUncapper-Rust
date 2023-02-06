//!
//! @file safe.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes the SKSE safe-write functions.
//! @bug No known bugs
//!

use core::slice;
use core::ffi::c_void;
use core::mem::size_of;

use windows_sys::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE};

/// The maximum patch size. Chosen as our largest patch size is 16 (call absolute).
const MAX_PATCH_SIZE: usize = 16;

/// Represents an assembled x86-64 patch to be written.
struct Patch {
    addr: usize,
    buf: [u8; MAX_PATCH_SIZE],
    len: usize
}

/// Encodes a x86-64 +rq register index.
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Register {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    Rbx = 3,
    Rsp = 4,
    Rbp = 5,
    Rsi = 6,
    Rdi = 7
}

/// Encodes the addressing mode of an instruction, or its opcode.
enum Encoding {
    JumpNear,
    CallRelative,
    JumpRelative,
    CallIndirect,
    JumpIndirect,
    CallReg(Register),
    JumpReg(Register),
    MoveImmQReg(Register),
    RelativeD(usize),
    AbsoluteSH(i8),
    AbsoluteD(u32),
    AbsoluteQ(u64)
}

///
/// An enumeration of the different types of control flow which can be written.
///
/// Indirect calling methods require an address to write the trampoline to.
///
pub enum Flow {
    CallRelative,
    JumpRelative,
    CallAbsolute,
    JumpAbsolute,
    CallRegAbsolute(Register),
    JumpRegAbsolute(Register),
    CallIndirect(usize),
    JumpIndirect(usize)
}

impl Patch {
    /// Generates a patch to be applied to the given address from the given encoding.
    fn new(
        addr: usize,
        chunks: &[Encoding]
    ) -> Result<Self, ()> {
        let mut this = Self {
            addr,
            buf: [0; MAX_PATCH_SIZE],
            len: 0
        };

        for chunk in chunks.iter() {
            match chunk {
                Encoding::JumpNear => this.append(&[0xeb]),
                Encoding::CallRelative => this.append(&[0xe8]),
                Encoding::JumpRelative => this.append(&[0xe9]),
                Encoding::CallIndirect => this.append(&[0xff, 0x15]),
                Encoding::JumpIndirect => this.append(&[0xff, 0x25]),
                Encoding::CallReg(reg) => this.append(&[0xff, 0xd0 + (*reg as u8)]),
                Encoding::JumpReg(reg) => this.append(&[0xff, 0xe0 + (*reg as u8)]),
                Encoding::MoveImmQReg(reg) => this.append(&[0x48, 0xb8 + (*reg as u8)]),
                Encoding::RelativeD(target) => {
                    let rel: i32 = (
                        target.wrapping_sub(addr + this.len + size_of::<i32>())
                    ).try_into().map_err(|_| ())?;

                    unsafe {
                        this.append(slice::from_raw_parts(
                            &rel as *const i32 as *const u8,
                            size_of::<i32>()
                        ));
                    }
                },
                Encoding::AbsoluteSH(h) => this.append(&[*h as u8]),
                Encoding::AbsoluteD(d) => unsafe {
                    this.append(
                        slice::from_raw_parts(d as *const u32 as *const u8, size_of::<u32>())
                    );
                },
                Encoding::AbsoluteQ(q) => unsafe {
                    this.append(
                        slice::from_raw_parts(q as *const u64 as *const u8, size_of::<u64>())
                    );
                }
            }
        }

        Ok(this)
    }

    /// Appends the given bytes to the patch buffer.
    fn append(
        &mut self,
        s: &[u8]
    ) {
        self.buf.split_at_mut(self.len).1.split_at_mut(s.len()).0.copy_from_slice(s);
        self.len += s.len();
    }

    /// Applies the given patch, protecting the region.
    unsafe fn apply(
        self
    ) {
        use_region(self.addr, self.len, || {
            self.apply_unchecked();
        });
    }

    /// Applies the given patch without protecting the region.
    unsafe fn apply_unchecked(
        self
    ) {
        std::ptr::copy(self.buf.as_ptr(), self.addr as *mut u8, self.len);
    }
}

impl Flow {
    ///
    /// Creates a new patch from the invoking flow, address, and target.
    ///
    /// If the flow is an indirect, the target is written to the trampoline on success.
    ///
    fn as_patch(
        &self,
        addr: usize,
        target: usize
    ) -> Result<Patch, ()> {
        let patch = match self {
            Flow::CallRelative => Patch::new(addr,  &[
                Encoding::CallRelative, Encoding::RelativeD(target)
            ]),

            Flow::JumpRelative => Patch::new(addr, &[
                Encoding::JumpRelative, Encoding::RelativeD(target)
            ]),

            Flow::CallAbsolute => Patch::new(addr, &[
                Encoding::CallIndirect, Encoding::AbsoluteD(0x02),
                Encoding::JumpNear, Encoding::AbsoluteSH(0x08),
                Encoding::AbsoluteQ(target as u64)
            ]),
            Flow::JumpAbsolute => Patch::new(addr, &[
                Encoding::JumpIndirect, Encoding::AbsoluteD(0),
                Encoding::AbsoluteQ(target as u64)
            ]),
            Flow::CallRegAbsolute(reg) => Patch::new(addr, &[
                Encoding::MoveImmQReg(*reg), Encoding::AbsoluteQ(target as u64),
                Encoding::CallReg(*reg)
            ]),
            Flow::JumpRegAbsolute(reg) => Patch::new(addr, &[
                Encoding::MoveImmQReg(*reg), Encoding::AbsoluteQ(target as u64),
                Encoding::JumpReg(*reg)
            ]),
            Flow::CallIndirect(trampoline) => Patch::new(addr, &[
                Encoding::CallIndirect, Encoding::RelativeD(*trampoline)
            ]),
            Flow::JumpIndirect(trampoline) => Patch::new(addr, &[
                Encoding::JumpIndirect, Encoding::RelativeD(*trampoline)
            ])
        }?;

        match self {
            Flow::CallIndirect(t) | Flow::JumpIndirect(t) => unsafe {
                std::ptr::write_unaligned(*t as *mut usize, target);
            }
            _ => ()
        }

        Ok(patch)
    }
}

/// Temporarily marks the given memory region for read/write, then calls the given fn.
pub unsafe fn use_region(
    addr: usize,
    size: usize,
    func: impl FnOnce()
) {
    let mut old_prot: u32 = 0;
    VirtualProtect(addr as *const c_void, size, PAGE_EXECUTE_READWRITE, &mut old_prot);
    func();
    VirtualProtect(addr as *const c_void, size, old_prot, &mut old_prot);
}

///
/// Writes the given instruction type to the given address, changing RIP to the given target.
///
/// This function may be called on code which is not currently marked read/write.
///
/// In order to use this function safely, the given address must be in the skyrim binary.
///
pub unsafe fn write_flow(
    addr: usize,
    target: usize,
    flow: Flow
) -> Result<(), ()> {
    flow.as_patch(addr, target)?.apply();
    Ok(())
}

///
/// Implementation of write_flow() which requires the address region to already be read/write.
///
pub unsafe fn write_flow_unchecked(
    addr: usize,
    target: usize,
    flow: Flow
) -> Result<(), ()> {
    flow.as_patch(addr, target)?.apply_unchecked();
    Ok(())
}
