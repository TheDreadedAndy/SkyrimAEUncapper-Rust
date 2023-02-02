//!
//! @file reloc.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes a type to disambiguate between offsets and addresses.
//! @bug No known bugs.
//!

extern "system" {
    fn SKSE64_Reloc__base__() -> usize;
}

/// Holds a game address, which can be accessed by offset or address.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelocAddr(usize);

impl RelocAddr {
    /// Gets the base address of the skyrim binary.
    pub fn base() -> usize {
        // SAFETY: Not actually unsafe lol.
        unsafe { SKSE64_Reloc__base__() }
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
