//!
//! @file reloc.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes a type to disambiguate between offsets and addresses.
//! @bug No known bugs.
//!

use later::Later;

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;

/// Holds a game address, which can be accessed by offset or address.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RelocAddr(usize);

/// Holds the base address of the skyrim binary.
static BASE_ADDR: Later<usize> = Later::new();

impl RelocAddr {
    #[doc(hidden)]
    pub fn init_manager() {
        BASE_ADDR.init(unsafe { GetModuleHandleA(std::ptr::null_mut()) as usize });
    }

    /// Gets the base address of the skyrim binary.
    pub fn base() -> usize {
        if BASE_ADDR.is_init() { *BASE_ADDR } else { 0x140000000 }
    }

    /// Creates a reloc addr from an offset.
    pub const fn from_offset(
        offset: usize
    ) -> Self {
        Self(offset)
    }

    /// Creates a reloc addr from an address.
    pub fn from_addr(
        addr: usize
    ) -> Self {
        assert!(Self::base() <= addr);
        Self(addr - Self::base())
    }

    /// Gets the underlying offset of the RelocAddr.
    pub const fn offset(
        self
    ) -> usize {
        self.0
    }

    /// Gets the actual address of the RelocAddr.
    pub fn addr(
        self
    ) -> usize {
        Self::base() + self.0
    }
}

impl std::ops::Add<usize> for RelocAddr {
    type Output = Self;
    fn add(
        self,
        rhs: usize
    ) -> Self::Output {
        Self(self.0 + rhs)
    }
}
