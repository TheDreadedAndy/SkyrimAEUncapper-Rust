//!
//! @file safe.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes the SKSE safe-write functions.
//! @bug No known bugs
//!

use core::ffi::c_void;
use core::mem::size_of;

use windows_sys::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE};

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
    CallIndirect(usize),
    JumpIndirect(usize)
}

/// Encodes the addressing mode of an instruction.
enum AddrMode {
    Relative(usize),
    Absolute(usize)
}

impl Flow {
    /// Gets the opcode encoding for the given flow type.
    fn opcode(
        &self
    ) -> &'static [u8] {
        match self {
            Self::CallRelative => &[0xe8],
            Self::JumpRelative => &[0xe9],
            // call 0x02(%rip); jmp 0x08 (over absolute).
            Self::CallAbsolute => &[0xff, 0x15, 0x02, 0x00, 0x00, 0x00, 0xeb, 0x08],
            Self::CallIndirect(_) => &[0xff, 0x15],
            Self::JumpAbsolute => &[0xff, 0x25, 0x00, 0x00, 0x00, 0x00],
            Self::JumpIndirect(_) => &[0xff, 0x25]
        }
    }

    /// Gets the size of the patch to be inserted at addr.
    fn patch_size(
        &self
    ) -> usize {
        self.opcode().len() + match self {
            Self::CallRelative |
            Self::JumpRelative |
            Self::CallIndirect(_) |
            Self::JumpIndirect(_) => size_of::<i32>(),

            Self::JumpAbsolute |
            Self::CallAbsolute => size_of::<usize>()
        }
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
    let mut ret: Result<(), ()> = Err(());
    use_region(addr, flow.patch_size(), || {
        ret = write_flow_unchecked(addr, target, flow);
    });
    return ret;
}

///
/// Implementation of write_flow() which requires the address region to already be read/write.
///
pub unsafe fn write_flow_unchecked(
    addr: usize,
    target: usize,
    flow: Flow
) -> Result<(), ()> {
    match flow {
        Flow::CallRelative |
        Flow::JumpRelative => {
            write_flow_instr_unchecked(addr, flow.opcode(), AddrMode::Relative(target))
        },

        Flow::CallAbsolute |
        Flow::JumpAbsolute => {
            write_flow_instr_unchecked(addr, flow.opcode(), AddrMode::Absolute(target))
        },

        Flow::CallIndirect(trampoline) |
        Flow::JumpIndirect(trampoline) => {
            write_flow_instr_unchecked(addr, flow.opcode(), AddrMode::Relative(target))?;
            std::ptr::write_unaligned(trampoline as *mut usize, target);
            Ok(())
        }
    }
}

///
/// Writes a control flow operation, with one 32-bit signed address
/// offset and a second, optional, absolute address.
///
/// Returns the number of bytes written.
///
unsafe fn write_flow_instr_unchecked(
    addr: usize,
    opcode: &[u8],
    mode: AddrMode
) -> Result<(), ()> {
    std::ptr::copy(opcode.as_ptr(), addr as *mut u8, opcode.len());

    match mode {
        AddrMode::Relative(target) => {
            let rel: i32 = (
                target - (addr + opcode.len() + size_of::<i32>())
            ).try_into().map_err(|_| ())?;
            std::ptr::write_unaligned((addr + opcode.len()) as *mut i32, rel);

        },
        AddrMode::Absolute(target) => {
            std::ptr::write_unaligned((addr + opcode.len()) as *mut usize, target);
        }
    }

    Ok(())
}
