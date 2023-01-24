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

use crate::safe::Signature;

/// Tracks a location in the skyrim game binary.
pub enum GameLocation {
    Id {
        id: u32,
        offset: usize
    },

    Offset {
        base: usize,
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
pub struct RelocAddr<T>(usize, std::marker::PhantomData<T>);

// TODO
pub unsafe fn apply(
    _version: SkseVersion
) -> Result<(), ()> {
    Ok(())
}
