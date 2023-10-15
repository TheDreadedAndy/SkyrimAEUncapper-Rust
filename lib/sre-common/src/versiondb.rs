//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Reimplements the Skyrim SE/AE versionlibdb header.
//! @bug No known bugs.
//!

use core::ffi::CStr;
use core::fmt::Write;
use core::mem::size_of;

use cstd::io::{File, Seek};

use crate::skse64::version::{SkseVersion, RUNTIME_VERSION_1_6_317};
use crate::skse64::reloc::RelocAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// An item in the version database, which holds its ID and the address that maps to it.
pub struct DatabaseItem {
    pub id   : usize,
    pub addr : RelocAddr
}

/// A file stream for iterating over the items in a version database.
///
/// Additionally contains the previous ID/offset and the pointer size of the database, which are
/// necessary to correctly parse it.
pub struct VersionDbStream {
    file              : File,
    prev_id           : usize,
    prev_offset       : usize,
    ptr_size          : usize,
    remaining_entries : usize,
}

/// An enumeration used to encode how the data in an address is stored in the database.
///
/// This enumeration will be constructed directly from data read in from the database.
#[derive(Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)] // Transmutes don't count as usage.
enum AddrEncoding {
    Raw64      = 0,
    Raw32      = 7,
    Raw16      = 6,
    Inc        = 1,
    PosDelta8  = 2,
    NegDelta8  = 3,
    PosDelta16 = 4,
    NegDelta16 = 5
}

// Trait used to ensure VersionDb::read only works on unsigned ints.
trait Unsigned {}
impl Unsigned for u8  {}
impl Unsigned for u16 {}
impl Unsigned for u32 {}
impl Unsigned for u64 {}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl VersionDbStream {
    /// Attempts to create a new version database, loading it with the specified version
    pub fn new(
        version: SkseVersion
    ) -> Self {
        // Large enough to hold a path for any valid game version.
        const PATH_SIZE: usize = 256;
        let mut buf = core_util::StringBuffer::<PATH_SIZE>::new();

        //
        // Note that we hard-code the build number to 0, as Bethesda doesn't use it.
        //
        // The SKSE64 team uses it to denote which store the game was obtained from, so
        // we can't just pull it from our version structure.
        //
        buf.write_fmt(format_args!(
            "Data\\SKSE\\Plugins\\{}-{}-{}-{}-0.bin",
            if version < RUNTIME_VERSION_1_6_317 { "version" } else { "versionlib" },
            version.major(),
            version.minor(),
            version.build()
        )).unwrap();

        Self::new_from_path(buf.as_c_str())
    }

    /// Creates a version database from the given path, setting the version based on the file.
    pub fn new_from_path(
        path: &CStr
    ) -> Self {
        let mut f = File::open(path, core_util::cstr!("rb")).unwrap();

        //
        // Parses the header of a version database file.
        //
        // The version db file format seems to be as follows:
        // - Each binary begins with a u32 version, where 1 is SE and 2 is AE.
        // - After that, there is a (major, minor, build, sub) u32 tuple. This can be skipped.
        // - The version tuple is followed by a u32 module name string len, between 0 and 0x10000.
        // - This string length is followed by exactly len many bytes encoding the name.
        // - Next, there is a u32 encoding a pointer size for the file.
        // - After that, there is a u32 count for the number of addresses in the database.
        // - The remainder of the database is the addresses contained within it.
        //
        let format = Self::read::<u32>(&mut f); // File format.
        assert!((format == 1) || (format == 2));

        f.seek(Seek::Current((size_of::<u32>() * 4) as i64)).unwrap();

        let mod_len = Self::read::<u32>(&mut f); // Module name length
        f.seek(Seek::Current(mod_len as i64)).unwrap();

        let (ptr_size, addr_count) = (
            Self::read::<u32>(&mut f) as usize,
            Self::read::<u32>(&mut f) as usize
        );

        Self { file: f, ptr_size, prev_id: 0, prev_offset: 0, remaining_entries: addr_count }
    }

    /// Read T from file.
    fn read<T: Unsigned>(
        f: &mut File
    ) -> T {
        let mut b: [u8; size_of::<u64>()] = [0; size_of::<u64>()];
        assert!(f.read(b.split_at_mut(size_of::<T>()).0).unwrap() == size_of::<T>());
        // SAFETY: We only read integer types, and ensure that the buffer is the right size.
        unsafe { core::ptr::read_unaligned(b.as_ptr() as *mut T) }
    }
}

impl Iterator for VersionDbStream {
    type Item = DatabaseItem;

    fn next(
        &mut self
    ) -> Option<Self::Item> {
        if self.remaining_entries == 0 {
            return None;
        }
        self.remaining_entries -= 1;

        //
        // Parses an address in the version database.
        //
        // Each address seems to be encoded as follows:
        // - First, is a control byte encoding two 3-bit values denoting an item type.
        //   The msb of the control byte determines if offset calculations should use
        //   the previous offset (0) or the poffset/ptr_size (1). We call this modified
        //   offset "tpoffset".
        // - Then, the encoded data. Relative control encoding is applied to pid/tpoffset.
        //   If the high byte of the control bit was set, the resulting offset is later
        //   multiplied by pointer size (equiv, each delta is multiplied by pointer size and
        //   we can just use poffset).
        //
        let control = Self::read::<u8>(&mut self.file);
        assert!(control & 0x08 == 0);

        // SAFETY: This is the defined encoding of the control byte.
        //         The enum is sized to always be in range.
        let (id_enc, offset_enc) = unsafe {(
            core::mem::transmute::<u8, AddrEncoding>(control & 0x07),
            core::mem::transmute::<u8, AddrEncoding>((control >> 4) & 0x07)
        )};

        self.prev_id     = id_enc.read(&mut self.file, self.prev_id);
        self.prev_offset = if (control & 0x80) != 0 /* is the offset by pointer? */ {
            offset_enc.read(&mut self.file, self.prev_offset / self.ptr_size) * self.ptr_size
        } else {
            offset_enc.read(&mut self.file, self.prev_offset)
        };

        Some(DatabaseItem { id: self.prev_id, addr: RelocAddr::from_offset(self.prev_offset) })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl AddrEncoding {
    /// Uses an address encoding to read in new data from the file, returning the result.
    fn read(
        self,
        f: &mut File,
        prev: usize
    ) -> usize {
        match self {
            Self::Raw64      => VersionDbStream::read::<u64>(f) as usize,
            Self::Raw32      => VersionDbStream::read::<u32>(f) as usize,
            Self::Raw16      => VersionDbStream::read::<u16>(f) as usize,
            Self::Inc        => prev + 1,
            Self::PosDelta8  => prev + (VersionDbStream::read::<u8>(f) as usize),
            Self::NegDelta8  => prev - (VersionDbStream::read::<u8>(f) as usize),
            Self::PosDelta16 => prev + (VersionDbStream::read::<u16>(f) as usize),
            Self::NegDelta16 => prev - (VersionDbStream::read::<u16>(f) as usize)
        }
    }
}
