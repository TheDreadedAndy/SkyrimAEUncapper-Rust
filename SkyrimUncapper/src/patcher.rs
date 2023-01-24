//!
//! @file patcher.rs
//! @author Andrew Spaulding (Kasplat)
//! @author Vadfromnu
//! @author Kassent
//! @brief Locates and applies pre-defined patches to game functions and objects.
//! @bug No known bugs.
//!
//! TODO
//!

use std::cell::UnsafeCell;

use skse64::version::SkseVersion;
use skse64::errors::skse_assert;
use skse64::log::skse_message;
use versionlib::VersionDb;

use crate::safe::Signature;

/// Tracks a location in the skyrim game binary.
pub enum GameLocation {
    Offset {
        base: usize,
        offset: usize
    },

    Id {
        id: usize,
        offset: usize
    }
}

/// Encodes the type of hook which is being used by a patch.
pub enum Hook {
    None,
    Jump5(usize),
    Jump6(usize),
    DirectJump(usize),
    Call5(usize),
    Call6(usize),
    DirectCall(usize)
}

/// Describes a location in code to be parsed and acted on by the patcher.
pub enum RelocPatch {
    Function {
        name: &'static str,
        loc: GameLocation,
        result: *mut UnsafeCell<usize>
    },

    Object {
        name: &'static str,
        loc: GameLocation,
        result: *mut UnsafeCell<usize>
    },

    Patch {
        name: &'static str,
        enabled: fn() -> bool,
        hook: Hook,
        loc: GameLocation,
        sig: Signature,
        trampoline: Option<*mut UnsafeCell<usize>>
    }
}

///
/// Contains an address retrieved by the patcher.
///
/// This structure is a transparent usize, as some results may be visible to ASM code.
///
#[repr(transparent)]
pub struct RelocAddr<T>(UnsafeCell<usize>, std::marker::PhantomData<T>);

impl GameLocation {
    /// Finds the game offset specified by this location.
    fn find(
        &self,
        db: &VersionDb
    ) -> Result<usize, ()> {
        match self {
            Self::Offset { base, offset } => {
                if let Ok(id) = db.find_id_by_offset(*base) {
                    skse_message!("Offset {:#x} has ID {}", base, id);
                    Ok(base + offset)
                } else {
                    Err(())
                }
            },
            Self::Id { id, offset } => {
                if let Ok(base) = db.find_offset_by_id(*id) {
                    Ok(base + offset)
                } else {
                    Err(())
                }
            }
        }
    }
}

impl std::fmt::Display for GameLocation {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>
    ) -> Result<(), std::fmt::Error> {
        match self {
            Self::Offset { base, offset } => {
                write!(f, "([BASE: {:#x}] + {:#x})", base, offset)
            },
            Self::Id { id, offset } => {
                write!(f, "([ID: {}] + {:#x})", id, offset)
            }
        }
    }
}

impl Hook {
    /// Gets the trampoline allocation size of the hook.
    fn alloc_size(
        &self
    ) -> usize {
        match self {
            Hook::Jump5(_) | Hook::Call5(_) => 14,
            Hook::Jump6(_) | Hook::Call6(_) => 8,
            _ => 0
        }
    }

    /// Gets the "on-site" patch size of the hook.
    fn patch_size(
        &self
    ) -> usize {
        match self {
            Hook::None => 0,
            Hook::Jump5(_) | Hook::Call5(_) | Hook::DirectJump(_) | Hook::DirectCall(_) => 5,
            Hook::Jump6(_) | Hook::Call6(_) => 6
        }
    }
}

impl<T> RelocAddr<T> {
    /// Creates a new relocatable address.
    pub const fn new() -> Self {
        assert!(std::mem::size_of::<T>() == std::mem::size_of::<usize>());
        Self(UnsafeCell::new(0), std::marker::PhantomData)
    }

    ///
    /// Gets a pointer to the underlying unsafe cell.
    ///
    /// This pointer must only be used at patch initialization time. Doing otherwise
    /// will cause races/undefined behavior within get().
    pub const fn inner(
        &self
    ) -> *mut UnsafeCell<usize> {
        &self.0 as *const _ as *mut _
    }

    /// Reads the contained address. Only legal if the address has been set through inner().
    pub fn get(
        &self
    ) -> T {
        skse_assert!(std::mem::size_of::<T>() == std::mem::size_of::<usize>());

        // SAFETY: We know that T is of the correct size for this transmute.
        unsafe {
            let addr = *self.0.get();
            skse_assert!(addr != 0);
            ::std::mem::transmute_copy::<usize, T>(&addr)
        }
    }
}

// Lie.
unsafe impl<T> Sync for RelocAddr<T> {}

// TODO
pub unsafe fn apply(
    _version: SkseVersion
) -> Result<(), ()> {
    Ok(())
}
