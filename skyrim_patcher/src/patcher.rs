//!
//! @file patcher.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Locates and applies pre-defined patches to game functions and objects.
//! @bug No known bugs.
//!
//! This file includes the patcher implementation, which reads in arrays of descriptor
//! from the skyrim and patches modules, and then applies them to the game. A descriptor
//! is either the location of a game function/object or a modification to a game function.
//!
//! Note that descriptors which modify the games code must also provide a signature of the code
//! they expect to be at the modification site for the length of the patch, to ensure the
//! mod is doing the intended modification on every version.
//!

use std::cell::UnsafeCell;

use skse64::log::skse_message;
use skse64::trampoline::Trampoline;
use skse64::reloc::RelocAddr;
use versionlib::VersionDb;

use crate::sig::{Signature, BinarySig};

/// Tracks a location in the skyrim game binary.
#[allow(dead_code)]
pub enum GameLocation {
    Offset {
        base: RelocAddr,
        offset: usize
    },

    Id {
        id: usize,
        offset: usize
    }
}

/// Encodes the type of hook which is being used by a patch.
#[allow(dead_code)]
pub enum Hook {
    None,
    Jump5(HookFn),
    Jump6(HookFn),
    DirectJump(HookFn),
    Call5(HookFn),
    Call6(HookFn),
    DirectCall(HookFn)
}

/// Describes a location in code to be parsed and acted on by the patcher.
pub enum Descriptor {
    Function {
        name: &'static str,
        loc: GameLocation,
        result: GameRefResult
    },

    Object {
        name: &'static str,
        loc: GameLocation,
        result: GameRefResult
    },

    Patch {
        name: &'static str,
        enabled: fn() -> bool,
        hook: Hook,
        loc: GameLocation,
        sig: Signature,
        trampoline: Option<GameRefResult>
    }
}

/// Describes error reasons for why a descriptor result could not be located.
enum DescriptorError {
    Disabled,
    Missing,
    Mismatch(Signature, BinarySig)
}

/// The result of an attempt to locate a descriptor.
type FindResult = Result<RelocAddr, DescriptorError>;

/// Stores a function which is used by a hook.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct HookFn(*const u8);

///
/// Contains an address retrieved by the patcher.
///
/// This structure is a transparent usize, as some results may be visible to ASM code.
///
#[repr(transparent)]
pub struct GameRef<T>(UnsafeCell<usize>, std::marker::PhantomData<T>);

/// Contains a pointer to the unsafe cell of a RelocAddr, to be written back to by the patcher.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct GameRefResult(*mut UnsafeCell<usize>);

impl GameLocation {
    /// Finds the game address specified by this location.
    fn find(
        &self,
        db: &VersionDb
    ) -> FindResult {
        match self {
            Self::Offset { base, offset } => {
                if let Ok(id) = db.find_id_by_addr(*base) {
                    skse_message!("Offset {:#x} has ID {}", base.offset(), id);
                    Ok(*base + *offset)
                } else {
                    Err(DescriptorError::Missing)
                }
            },
            Self::Id { id, offset } => {
                if let Ok(ra) = db.find_addr_by_id(*id) {
                    Ok(ra + *offset)
                } else {
                    Err(DescriptorError::Missing)
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
                if *offset == 0 {
                    write!(f, "[BASE: {:#x}]", base.offset())
                } else {
                    write!(f, "([BASE: {:#x}] + {:#x})", base.offset(), offset)
                }
            },
            Self::Id { id, offset } => {
                if *offset == 0 {
                    write!(f, "[ID: {}]", id)
                } else {
                    write!(f, "([ID: {}] + {:#x})", id, offset)
                }
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

    ///
    /// Installs the given patch.
    ///
    /// In order to use this function safely, the given address must be the correct
    /// location for this patch to be installed to.
    ///
    unsafe fn install(
        &self,
        addr: usize
    ) {
        match self {
            Self::Jump5(hook) => {
                skse64::trampoline::write_jump5(Trampoline::Global, addr, hook.addr());
            },
            Self::Call5(hook) => {
                skse64::trampoline::write_call5(Trampoline::Global, addr, hook.addr());
            },
            Self::Jump6(hook) => {
                skse64::trampoline::write_jump6(Trampoline::Global, addr, hook.addr());
            },
            Self::Call6(hook) => {
                skse64::trampoline::write_call6(Trampoline::Global, addr, hook.addr());
            },
            Self::DirectJump(hook) => {
                skse64::safe::write_jump(addr, hook.addr()).unwrap();
            },
            Self::DirectCall(hook) => {
                skse64::safe::write_call(addr, hook.addr()).unwrap();
            },
            Self::None => panic!("Cannot install to a None hook!")
        }
    }
}

impl Descriptor {
    /// Finds the address and verifies its signature, if applicable.
    fn find(
        &self,
        db: &VersionDb
    ) -> FindResult {
        match self {
            Self::Object { loc, .. } => loc.find(db),
            Self::Function { loc, .. } => loc.find(db),
            Self::Patch { enabled, loc, sig, .. } => {
                if !enabled() {
                    return Err(DescriptorError::Disabled)
                }

                let addr = loc.find(db)?;
                unsafe {
                    // SAFETY: We know addr is in the skyrim binary, since it came from the db.
                    sig.check(addr.addr()).map_err(|e| DescriptorError::Mismatch(*sig, e))?;
                }
                Ok(addr)
            }
        }
    }

    /// Reports the results of an attempt to find a descriptor.
    fn report(
        &self,
        res: &FindResult
    ) {
        match res {
            Ok(addr) => {
                skse_message!(
                    "[SUCCESS] {} is at offset {:#x}",
                    self,
                    addr.offset()
                );
            },
            Err(DescriptorError::Disabled) => {
                skse_message!("[SKIPPED] {} is disabled", self);
            },
            Err(DescriptorError::Missing) => {
                skse_message!("[FAILURE] {} was not in the version database!", self);
            },
            Err(DescriptorError::Mismatch(sig, bsig)) => {
                skse_message!(
                    "[FAILURE] {} at offset {:#x} did not match the expected code signature!",
                    self,
                    bsig.reloc().offset()
                );
                skse_message!("\\------> [EXPECTED] {}", sig);
                skse_message!(" \\-----> [FOUND...] {}", bsig);
            }
        }
    }

    /// Gets the number of bytes in the game code this patch expects to alter.
    fn size(
        &self
    ) -> usize {
        match self {
            Self::Patch { sig, .. } => sig.len(),
            _ => 0
        }
    }

    /// Gets the hook for this patch.
    fn hook<'a>(
        &'a self
    ) -> Option<&'a Hook> {
        match self {
            Self::Patch { hook, .. } => Some(hook),
            _ => None
        }
    }

    /// Checks if the given patch is disabled.
    fn disabled(
        &self
    ) -> bool {
        match self {
            Self::Patch { enabled, .. } => !enabled(),
            _ => false
        }
    }
}

impl std::fmt::Display for Descriptor {
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

impl HookFn {
    ///
    /// Creates a new hook function type.
    ///
    /// In order to use this function safely, the function type
    /// must be a valid extern "system" fn.
    ///
    pub const unsafe fn new(
        func: *const u8
    ) -> Self {
        Self(func)
    }

    /// Gets the underlying address of the hook function.
    fn addr(
        self
    ) -> usize {
        self.0 as usize
    }
}

impl<T> GameRef<T> {
    /// Creates a new game reference structure.
    pub const fn new() -> Self {
        assert!(std::mem::size_of::<T>() == std::mem::size_of::<usize>());
        Self(UnsafeCell::new(0), std::marker::PhantomData)
    }

    ///
    /// Gets a pointer to the underlying unsafe cell.
    ///
    /// This pointer must only be used at patch initialization time. Doing otherwise
    /// will cause races/undefined behavior within get().
    ///
    pub const fn inner(
        &self
    ) -> GameRefResult {
        GameRefResult(&self.0 as *const _ as *mut _)
    }

    /// Reads the contained address. Only legal if the address has been set through inner().
    pub fn get(
        &self
    ) -> T {
        assert!(std::mem::size_of::<T>() == std::mem::size_of::<usize>());

        // SAFETY: We know that T is of the correct size for this transmute.
        unsafe {
            let addr = *self.0.get();
            assert!(addr != 0);
            ::std::mem::transmute_copy::<usize, T>(&addr)
        }
    }
}

impl GameRefResult {
    ///
    /// Updates the address of a RelocAddr.
    ///
    /// In order to use this function safely, the caller must ensure that the given address
    /// points to a valid type T.
    ///
    unsafe fn write(
        &self,
        a: usize
    ) {
        // SAFETY: We know this came from a RelocAddr, so the pointer is valid.
        assert!(!self.0.is_null());
        let res = (*self.0).get();
        assert!(*res == 0);
        assert!(a != 0);
        *res = a;
    }
}

// SAFETY: The patcher is protected by the single initialization of skse.
unsafe impl Sync for HookFn {}
unsafe impl<T> Sync for GameRef<T> {}
unsafe impl Sync for GameRefResult {}

/// Locates any game functions/objects, and applies any code patches.
pub fn apply<const NUM_PATCHES: usize>(
    patches: [&Descriptor; NUM_PATCHES]
) -> Result<(), ()> {
    let db = VersionDb::new(None);
    let mut res_addrs: [usize; NUM_PATCHES] = [0; NUM_PATCHES];
    let mut alloc_size: usize = 0;

    skse_message!("--------------------- Skyrim Patcher 1.0.1 ---------------------");

    // Attempt to locate all of the patch signatures.
    let mut fails = 0;
    for (i, sig) in patches.iter().enumerate() {
        let res = sig.find(&db);
        sig.report(&res);

        match res {
            Ok(addr) => {
                res_addrs[i] = addr.addr();
                if let Some(h) = sig.hook() {
                    alloc_size += h.alloc_size();
                    assert!(h.patch_size() <= sig.size());
                }
            },
            Err(DescriptorError::Disabled) => (),
            _ => {
                fails += 1;
            }
        }
    }


    if fails > 0 {
        skse_message!("[FAILURE] Could not locate every game signature!");
        return Err(())
    }

    // Allocate our branch trampoline.
    if alloc_size > 0 {
        // SAFETY: We're not giving an image base, so this is actually safe.
        unsafe { skse64::trampoline::create(Trampoline::Global, alloc_size, None) };
        skse_message!(
            "[SUCCESS] Created branch trampoline buffer with a size of {} bytes",
            alloc_size
        );
    } else {
        skse_message!("[SKIPPED] All patches are disabled.");
    }

    // Install our patches.
    for (i, sig) in patches.iter().enumerate() {
        if sig.disabled() { continue; }

        let hook_size = sig.hook().map(|h| h.patch_size()).unwrap_or(0);
        let ret_addr = res_addrs[i] + hook_size;
        match sig {
            Descriptor::Patch { trampoline, hook, .. } => {
                unsafe {
                    // SAFETY: We will ensure our return address is valid by writing NOPS to any
                    //         bytes that are part of the patch and after the return address.
                    if let Some(t) = trampoline {
                        t.write(ret_addr);
                    }
                    hook.install(res_addrs[i]);
                }
            },
            Descriptor::Function { result, .. } | Descriptor::Object { result, .. } => {
                unsafe {
                    // SAFETY: The version DB ensures that we have the write object for
                    //         the requested ID.
                    result.write(res_addrs[i]);
                }
            }
        }

        unsafe {
            // SAFETY: We have matched signatures to ensure our patch is valid.
            let remain = sig.size() - hook_size;
            if remain > 0 {
                skse64::safe::use_region(ret_addr, remain, || {
                    ::std::ptr::write_bytes::<u8>(ret_addr as *mut u8, 0x90, remain);
                });
            }
        }
    }

    skse_message!("[SUCCESS] Applied game patches.");
    skse_message!("----------------------------------------------------------------");

    Ok(())
}
