//!
//! @file patcher.rs
//! @author Andrew Spaulding (Kasplat)
//! @author Vadfromnu
//! @author Kassent
//! @brief Locates and applies pre-defined patches to game functions and objects.
//! @bug No known bugs.
//!
//! This file includes the patcher implementation, which reads in arrays of patches
//! from the skyrim and patches modules, and then applies them to the game. Note
//! that even though I say "patches" here, I really mean any relocatable function
//! or object as well, which is what the "patches" in the skyrim module are.
//!
//! Note that patches which modify the games code must also provide a signature of the code
//! they expect to be at the modification site for the length of the patch, to ensure the
//! mod is doing the intended modification on every version.
//!

use std::cell::UnsafeCell;

use skse64::errors::{skse_assert, skse_halt};
use skse64::log::{skse_message, skse_error};
use versionlib::VersionDb;

use crate::safe::Signature;
use crate::skyrim::GAME_SIGNATURES;

/// Tracks a location in the skyrim game binary.
#[allow(dead_code)]
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
        result: RelocResult
    },

    Object {
        name: &'static str,
        loc: GameLocation,
        result: RelocResult
    },

    Patch {
        name: &'static str,
        enabled: fn() -> bool,
        hook: Hook,
        loc: GameLocation,
        sig: Signature,
        trampoline: Option<RelocResult>
    }
}

/// Describes error reasons for why a patch could not be located.
enum PatchError {
    Disabled,
    Missing,
    Mismatch
}

/// The result of an attempt to locate a patch.
type PatchResult = Result<usize, PatchError>;

///
/// Contains an address retrieved by the patcher.
///
/// This structure is a transparent usize, as some results may be visible to ASM code.
///
#[repr(transparent)]
pub struct RelocAddr<T>(UnsafeCell<usize>, std::marker::PhantomData<T>);

/// Contains a pointer to the unsafe cell of a RelocAddr, to be written back to by the patcher.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct RelocResult(*mut UnsafeCell<usize>);

impl GameLocation {
    /// Finds the game offset specified by this location.
    fn find(
        &self,
        db: &VersionDb
    ) -> PatchResult {
        match self {
            Self::Offset { base, offset } => {
                if let Ok(id) = db.find_id_by_offset(*base) {
                    skse_message!("Offset {:#x} has ID {}", base, id);
                    Ok(base + offset)
                } else {
                    Err(PatchError::Missing)
                }
            },
            Self::Id { id, offset } => {
                if let Ok(base) = db.find_offset_by_id(*id) {
                    Ok(base + offset)
                } else {
                    Err(PatchError::Missing)
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

impl RelocPatch {
    /// Finds the patch and verifies its signature, if applicable.
    fn find(
        &self,
        db: &VersionDb
    ) -> PatchResult {
        match self {
            Self::Object { loc, .. } => loc.find(db),
            Self::Function { loc, .. } => loc.find(db),
            Self::Patch { enabled, loc, sig, .. } => {
                if !enabled() {
                    return Err(PatchError::Disabled)
                }

                let addr = loc.find(db)?;
                unsafe {
                    // SAFETY: We know addr is in the skyrim binary, since it came from the db.
                    sig.check(addr).map_err(|_| PatchError::Mismatch)?;
                }
                Ok(addr)
            }
        }
    }

    /// Reports the results of an attempt to find a signature.
    fn report(
        &self,
        res: &PatchResult
    ) {
        match res {
            Ok(addr) => {
                skse_message!("[SUCCESS] {} is at offset {:#x}", self, addr);
            },
            Err(PatchError::Disabled) => {
                skse_message!("[SKIPPED] {} is disabled", self);
            },
            Err(PatchError::Missing) => {
                skse_error!("[FAILURE] {} was not in the version database!", self);
            },
            Err(PatchError::Mismatch) => {
                skse_error!("[FAILURE] {} did not match the expected code signature!", self);
            }
        }
    }

    /// Gets the result structure for this patch.
    fn result(
        &self
    ) -> RelocResult {
        match self {
            Self::Object { result, .. } => *result,
            Self::Function { result, .. } => *result,
            Self::Patch { .. } => skse_halt!("Cannot get the result field of a patch type!")
        }
    }
}

impl std::fmt::Display for RelocPatch {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>
    ) -> Result<(), std::fmt::Error> {
        match self {
            Self::Object { name, loc, .. } => {
                write!(f, "Object {} {}", name, loc)
            },
            Self::Function { name, loc, .. } => {
                write!(f, "Function {} {}", name, loc)
            },
            Self::Patch { name, loc, .. } => {
                write!(f, "Patch {} {}", name, loc)
            }
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
    ) -> RelocResult {
        RelocResult(&self.0 as *const _ as *mut _)
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

impl RelocResult {
    ///
    /// Updates the address of a RelocAddr.
    ///
    /// In order to use this function safely, the caller must ensure that the given address
    /// points to a valid type T.
    ///
    pub unsafe fn write(
        &self,
        a: usize
    ) {
        // SAFETY: We know this came from a RelocAddr, so the pointer is valid.
        skse_assert!(!self.0.is_null());
        let res = (*self.0).get();
        skse_assert!(*res == 0);
        skse_assert!(a != 0);
        *res = a;
    }
}

// Lie.
unsafe impl<T> Sync for RelocAddr<T> {}
unsafe impl Sync for RelocResult {}

/// Locates any game functions/objects, and applies any code patches.
pub fn apply() -> Result<(), ()> {
    let db = VersionDb::new(None);

    // Locate any game signatures for objects/functions we call.
    let mut fails = 0;
    for sig in GAME_SIGNATURES.iter() {
        let res = sig.find(&db);
        sig.report(&res);

        if let Ok(addr) = res {
            // SAFETY: The version DB ensures we obtained the correct object.
            unsafe { sig.result().write(addr); }
        } else {
            fails += 1;
        }
    }

    if fails == 0 {
        Ok(())
    } else {
        Err(())
    }
}
