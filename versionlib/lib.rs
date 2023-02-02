//!
//! @file lib.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief Reimplements the Skyrim AE versionlibdb header.
//! @bug No known bugs.
//!

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

use skse64::version::SkseVersion;
use skse64::reloc::RelocAddr;

/// A version database, which allows for offsets/ids to be searched for by each other.
pub struct VersionDb {
    by_id: HashMap<usize, usize>,
    by_offset: HashMap<usize, usize>,
    version: SkseVersion
}

///
/// An enumeration used to encode how the data in an address is stored in the database.
///
/// This enumeration will be constructed directly from data read in from the database.
///
#[derive(Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)] // Transmutes don't count as usage.
enum AddrEncoding {
    Raw64 = 0,
    Raw32 = 7,
    Raw16 = 6,
    Inc = 1,
    PosDelta8 = 2,
    NegDelta8 = 3,
    PosDelta16 = 4,
    NegDelta16 = 5
}

// Trait used to ensure read only works on unsigned ints.
trait Unsigned {}
impl Unsigned for u8 {}
impl Unsigned for u16 {}
impl Unsigned for u32 {}
impl Unsigned for u64 {}

impl VersionDb {
    /// Attempts to create a new version database, loading it with the specified version
    /// (or the current version, if none is provided).
    pub fn new(
        version: Option<SkseVersion>
    ) -> Self {
        let mut by_id = HashMap::<usize, usize>::new();
        let mut by_offset = HashMap::<usize, usize>::new();
        let version = version.unwrap_or(skse64::version::current_runtime());

        let mut f = std::fs::File::open(PathBuf::from(format!(
            "Data\\SKSE\\Plugins\\versionlib-{}-{}-{}-{}.bin",
            version.major(),
            version.minor(),
            version.build(),
            version.runtime_type()
        ))).unwrap();

        let (ptr_size, addr_count) = Self::parse_header(&mut f);
        let (mut pid, mut poffset) = (0, 0);
        for _ in 0..addr_count {
            let (id, offset) = Self::parse_addr(&mut f, pid, poffset, ptr_size);

            assert!(by_id.insert(id, offset).is_none());
            assert!(by_offset.insert(offset, id).is_none());

            pid = id;
            poffset = offset;
        }

        Self {
            by_id,
            by_offset,
            version
        }
    }

    /// Gets the version that is currently loaded into the database.
    pub fn loaded_version(
        &self
    ) -> SkseVersion {
        self.version
    }

    /// Attempts to find the address independent id for the given offset.
    pub fn find_id_by_addr(
        &self,
        addr: RelocAddr
    ) -> Result<usize, ()> {
        self.by_offset.get(&addr.offset()).map(|id| *id).ok_or(())
    }

    /// Attempts to find the offset of the given address independent id.
    pub fn find_addr_by_id(
        &self,
        id: usize
    ) -> Result<RelocAddr, ()> {
        self.by_id.get(&id).map(|o| RelocAddr::from_offset(*o)).ok_or(())
    }

    ///
    /// Parses the header of a version database file.
    ///
    /// The version db file format seems to be as follows:
    /// - Each binary begins with a u32 version, where 2 is AE.
    /// - After that, there is a (major, minor, build, sub) u32 tuple. This can be skipped.
    /// - The version tuple is followed by a u32 module name string len, between 0 and 0x10000.
    /// - This string length is followed by exactly len many bytes encoding the name.
    /// - Next, there is a u32 encoding a pointer size for the file.
    /// - After that, there is a u32 count for the number of addresses in the database.
    /// - The remainder of the database is the addresses contained within it.
    ///
    fn parse_header(
        f: &mut File
    ) -> (u32, u32) {
        assert!(Self::read::<u32>(f) == 2); // version
        Self::skip(f, (size_of::<u32>() * 4) as u32); // Runtime version
        let mod_len = Self::read::<u32>(f); // Module name length
        Self::skip(f, mod_len); // Module name.
        let ptr_size = Self::read::<u32>(f);
        let addr_count = Self::read::<u32>(f);
        (ptr_size, addr_count)
    }

    ///
    /// Parses an address in the version database.
    ///
    /// Each address seems to be encoded as follows:
    /// - First, is a control byte encoding two 3-bit values denoting an item type.
    ///   The msb of the control byte determines if offset calculations should use
    ///   the previous offset (0) or the poffset/ptr_size (1). We call this modified
    ///   offset "tpoffset".
    /// - Then, the encoded data. Relative control encoding is applied to pid/tpoffset.
    ///   If the high byte of the control bit was set, the resulting offset is later
    ///   multiplied by pointer size (equiv, each delta is multiplied by pointer size and
    ///   we can just use poffset).
    ///
    fn parse_addr(
        f: &mut File,
        pid: usize,
        poffset: usize,
        ptr_size: u32
    ) -> (usize, usize) {
        // SAFETY: This is the defined encoding of the control byte. The enum is sized to always
        //         be in range.
        let control = Self::read::<u8>(f);
        let offset_ptr = control & 0x80 > 0;
        let id_enc = unsafe { std::mem::transmute::<u8, AddrEncoding>(control & 0x07) };
        let offset_enc = unsafe { std::mem::transmute::<u8, AddrEncoding>((control >> 4) & 0x07) };

        let offset_mult = if offset_ptr { ptr_size } else { 1 };
        let id = id_enc.read(f, pid, 1);
        let offset = offset_enc.read(f, poffset, offset_mult);
        (id, offset)
    }

    /// Read T from file.
    fn read<T: Unsigned>(
        f: &mut File
    ) -> T {
        let mut b: [u8; size_of::<u64>()] = [0; size_of::<u64>()];
        assert!(f.read(b.split_at_mut(size_of::<T>()).0).unwrap() == size_of::<T>());
        unsafe {
            // SAFETY: We only read integer types, and ensure that the buffer is the right size.
            std::ptr::read_unaligned(b.as_ptr() as *mut T)
        }
    }

    /// Skips bytes in a file.
    fn skip(
        f: &mut File,
        n: u32
    ) {
        let pos = f.seek(SeekFrom::Current(0)).unwrap();
        assert!(f.seek(SeekFrom::Current(n as i64)).unwrap() == pos + (n as u64));
    }
}

impl AddrEncoding {
    /// Uses an address encoding to read in new data from the file, returning the result.
    fn read(
        self,
        f: &mut File,
        prev: usize,
        mult: u32
    ) -> usize {
        let mult = mult as usize;
        match self {
            Self::Raw64 => VersionDb::read::<u64>(f) as usize,
            Self::Raw32 => VersionDb::read::<u32>(f) as usize,
            Self::Raw16 => VersionDb::read::<u16>(f) as usize,
            Self::Inc => prev + 1,
            Self::PosDelta8 => prev + ((VersionDb::read::<u8>(f) as usize) * mult),
            Self::NegDelta8 => prev - ((VersionDb::read::<u8>(f) as usize) * mult),
            Self::PosDelta16 => prev + ((VersionDb::read::<u16>(f) as usize) * mult),
            Self::NegDelta16 => prev - ((VersionDb::read::<u16>(f) as usize) * mult)
        }
    }
}
