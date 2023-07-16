//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Reimplements the Skyrim SE/AE versionlibdb header.
//! @bug No known bugs.
//!

use std::vec::Vec;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

use crate::skse64::version::{SkseVersion, RUNTIME_VERSION_1_6_317};
use crate::skse64::reloc::RelocAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// An item in the version database, which holds its ID and the address that maps to it.
pub struct DatabaseItem {
    pub id   : usize,
    pub addr : RelocAddr
}

/// A version database, which allows for offsets/ids to be searched for by each other.
pub struct VersionDb {
    by_id   : Vec<DatabaseItem>,
    version : SkseVersion
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

impl VersionDb {
    /// Attempts to create a new version database, loading it with the specified version
    pub fn new(
        version: SkseVersion
    ) -> Self {
        //
        // Note that we hard-code the build number to 0, as Bethesda doesn't use it.
        //
        // The SKSE64 team uses it to denote which store the game was obtained from, so
        // we can't just pull it from our version structure.
        //
        Self::new_from_path(&std::path::PathBuf::from(format!(
            "Data\\SKSE\\Plugins\\{}-{}-{}-{}-0.bin",
            if version < RUNTIME_VERSION_1_6_317 { "version" } else { "versionlib" },
            version.major(),
            version.minor(),
            version.build()
        )))
    }

    /// Creates a version database from the given path, setting the version based on the file.
    pub fn new_from_path(
        path: &std::path::Path
    ) -> Self {
        let     f     = &mut std::fs::File::open(path).unwrap();
        let mut by_id = Vec::new();

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
        let format = Self::read::<u32>(f); // File format.
        assert!((format == 1) || (format == 2));
        let version = SkseVersion::new(
            Self::read::<u32>(f), // major
            Self::read::<u32>(f), // minor
            Self::read::<u32>(f), // build/revision
            Self::read::<u32>(f)  // sub
        );
        let mod_len = Self::read::<u32>(f); // Module name length
        let pos = f.seek(SeekFrom::Current(0)).unwrap();
        assert!(f.seek(SeekFrom::Current(mod_len as i64)).unwrap() == pos + (mod_len as u64));
        let (ptr_size, addr_count) = (Self::read::<u32>(f) as usize, Self::read::<u32>(f));

        // The previous ID/offset are necessary to parse the database, and are initialized to zero.
        let (mut pid, mut poffset) = (0, 0);
        for _ in 0..addr_count {
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
            let control = Self::read::<u8>(f);
            assert!(control & 0x08 == 0);

            // SAFETY: This is the defined encoding of the control byte.
            //         The enum is sized to always be in range.
            let (id_enc, offset_enc) = unsafe {(
                std::mem::transmute::<u8, AddrEncoding>(control & 0x07),
                std::mem::transmute::<u8, AddrEncoding>((control >> 4) & 0x07)
            )};

            let id     = id_enc.read(f, pid);
            let offset = if (control & 0x80) != 0 /* is the offset by pointer? */ {
                offset_enc.read(f, poffset / ptr_size) * ptr_size
            } else {
                offset_enc.read(f, poffset)
            };

            let index = by_id.binary_search_by(|lhs: &DatabaseItem| lhs.id.cmp(&id)).unwrap_err();
            by_id.insert(index, DatabaseItem {
                id,
                addr: RelocAddr::from_offset(offset)
            });

            pid = id;
            poffset = offset;
        }

        Self { by_id, version }
    }

    /// Gets the version that is currently loaded into the database.
    pub fn loaded_version(
        &self
    ) -> SkseVersion {
        self.version
    }

    /// Attempts to find the offset of the given address independent id.
    pub fn find_addr_by_id(
        &self,
        id: usize
    ) -> Result<RelocAddr, ()> {
        match self.by_id.binary_search_by(|lhs| lhs.id.cmp(&id)) {
            Ok(index) => Ok(self.by_id[index].addr),
            Err(_)    => Err(())
        }
    }

    /// Returns a reference to the underlying database.
    pub fn as_vec(
        &self
    ) -> &Vec<DatabaseItem> {
        &self.by_id
    }

    /// Read T from file.
    fn read<T: Unsigned>(
        f: &mut File
    ) -> T {
        let mut b: [u8; size_of::<u64>()] = [0; size_of::<u64>()];
        assert!(f.read(b.split_at_mut(size_of::<T>()).0).unwrap() == size_of::<T>());
        // SAFETY: We only read integer types, and ensure that the buffer is the right size.
        unsafe { std::ptr::read_unaligned(b.as_ptr() as *mut T) }
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
            Self::Raw64      => VersionDb::read::<u64>(f) as usize,
            Self::Raw32      => VersionDb::read::<u32>(f) as usize,
            Self::Raw16      => VersionDb::read::<u16>(f) as usize,
            Self::Inc        => prev + 1,
            Self::PosDelta8  => prev + (VersionDb::read::<u8>(f) as usize),
            Self::NegDelta8  => prev - (VersionDb::read::<u8>(f) as usize),
            Self::PosDelta16 => prev + (VersionDb::read::<u16>(f) as usize),
            Self::NegDelta16 => prev - (VersionDb::read::<u16>(f) as usize)
        }
    }
}