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
use std::ptr::NonNull;
use std::vec::Vec;

#[cfg(feature = "alloc_trampoline")]
use skse64::trampoline::Trampoline;

use skse64::log::{skse_message, skse_fatal};
use skse64::reloc::RelocAddr;
use skse64::safe::{verify_flow, write_flow, Flow};
use skse64::plugin_api::Message;
use versionlib::VersionDb;
use racy_cell::RacyCell;

use crate::sig::{Signature, BinarySig};

pub use skse64::safe::Register;

/// Contains a version independent address ID for the specified skyrim versions.
pub enum GameLocation {
    Base { se: usize, ae: usize },
    Se { id: usize, offset: usize },
    Ae { id: usize, offset: usize },
    All { id_se: usize, offset_se: usize, id_ae: usize, offset_ae: usize}
}

/// Encodes the type of hook which is being used by a patch.
#[derive(Clone)]
pub enum Hook {
    None,

    #[cfg(feature = "alloc_trampoline")]
    Jump5 {
        entry: *const u8,
        trampoline: NonNull<UnsafeCell<usize>>
    },

    #[cfg(feature = "alloc_trampoline")]
    Call5(*const u8),

    #[cfg(feature = "alloc_trampoline")]
    Jump6 {
        entry: *const u8,
        trampoline: NonNull<UnsafeCell<usize>>
    },

    #[cfg(feature = "alloc_trampoline")]
    Call6(*const u8),

    DirectJump {
        entry: *const u8,
        trampoline: NonNull<UnsafeCell<usize>>
    },

    DirectCall(*const u8),

    Jump12 {
        entry: *const u8,
        clobber: Register,
        trampoline: NonNull<UnsafeCell<usize>>
    },

    Call12 {
        entry: *const u8,
        clobber: Register
    },

    Jump14 {
        entry: *const u8,
        trampoline: NonNull<UnsafeCell<usize>>
    },

    Call16(*const u8)
}

/// Describes a location in code to be parsed and acted on by the patcher.
pub enum Descriptor {
    Function {
        name: &'static str,
        loc: GameLocation,
        result: NonNull<UnsafeCell<usize>>
    },

    Object {
        name: &'static str,
        loc: GameLocation,
        result: NonNull<UnsafeCell<usize>>
    },

    Patch {
        name: &'static str,
        enabled: fn() -> bool,
        conflicts: Option<&'static str>,
        hook: Hook,
        loc: GameLocation,
        sig: Signature,
    }
}

/// Describes error reasons for why a descriptor result could not be located.
#[derive(Debug)]
enum DescriptorError {
    IncompatibleGameVersion,
    Disabled,
    Missing,
    Mismatch(Signature, BinarySig)
}

/// The result of an attempt to locate a descriptor.
type FindResult = Result<RelocAddr, DescriptorError>;

///
/// Contains an address retrieved by the patcher.
///
/// This structure is a transparent usize, as some results may be visible to ASM code.
///
#[repr(transparent)]
pub struct GameRef<T>(UnsafeCell<usize>, std::marker::PhantomData<T>);

/// Contains information about a patch necessary to verify its integrity.
struct PatchResult {
    name: &'static str,
    conflicts: &'static str,
    hook: Hook,
    loc: RelocAddr
}

/// Contains the set of patches installed by a call to apply().
struct PatchSet(Vec<PatchResult>);

impl GameLocation {
    /// Finds the game address specified by this location.
    fn find(
        &self,
        db: &VersionDb
    ) -> FindResult {
        let (id, offset) = self.get()?;
        if let Ok(ra) = db.find_addr_by_id(id) {
            Ok(ra + offset)
        } else {
            Err(DescriptorError::Missing)
        }
    }

    /// Checks if the game location is compatible with the running version.
    fn compatible(
        &self
    ) -> bool {
        self.get().is_ok()
    }

    /// Gets the address independent location, if it is compatible with the running game version.
    fn get(
        &self
    ) -> Result<(usize, usize), DescriptorError> {
        let is_se = skse64::version::current_runtime() <= skse64::version::RUNTIME_VERSION_1_5_97;
        let id = match self {
            Self::Base { se, ae } => if is_se { Some((*se, 0)) } else { Some((*ae, 0)) },
            Self::Se { id, offset } => if is_se { Some((*id, *offset)) } else { None },
            Self::Ae { id, offset } => if is_se { None } else { Some((*id, *offset)) },
            Self::All { id_se, offset_se, id_ae, offset_ae } => if is_se {
                Some((*id_se, *offset_se))
            } else {
                Some((*id_ae, *offset_ae))
            }
        };
        id.ok_or(DescriptorError::IncompatibleGameVersion)
    }
}

impl std::fmt::Display for GameLocation {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>
    ) -> Result<(), std::fmt::Error> {
        let (id, offset) = self.get().unwrap();
        if offset == 0 {
            write!(f, "[ID: {}]", id)
        } else {
            write!(f, "([ID: {}] + {:#x})", id, offset)
        }
    }
}

impl Hook {
    /// Gets the trampoline allocation size of the hook.
    #[cfg(feature = "alloc_trampoline")]
    fn alloc_size(
        &self
    ) -> usize {
        match self {
            #[cfg(feature = "alloc_trampoline")]
            Hook::Jump5 { .. } | Hook::Call5(_) => 14,
            #[cfg(feature = "alloc_trampoline")]
            Hook::Jump6 { .. } | Hook::Call6(_) => 8,
            _ => 0
        }
    }

    /// Gets the "on-site" patch size of the hook.
    fn patch_size(
        &self
    ) -> usize {
        match self {
            #[cfg(feature = "alloc_trampoline")]
            Hook::Jump5 { .. } | Hook::Call5(_) => 5,
            #[cfg(feature = "alloc_trampoline")]
            Hook::Jump6 { .. } | Hook::Call6(_) => 6,
            Hook::None => 0,
            Hook::DirectJump { .. } | Hook::DirectCall(_) => 5,
            Hook::Jump12 { .. } | Hook::Call12 { .. } => 12,
            Hook::Jump14 { .. } => 14,
            Hook::Call16(_) => 16,
        }
    }

    /// Gets a pointer to the patches return trampoline, if it exists.
    fn trampoline(
        &self
    ) -> Option<NonNull<UnsafeCell<usize>>> {
        match self {
            #[cfg(feature = "alloc_trampoline")]
            Hook::Jump5  { trampoline, .. } | Hook::Jump6 { trampoline, .. } => {
                Some(*trampoline)
            },
            Hook::Jump12 { trampoline, .. } |
            Hook::Jump14 { trampoline, .. } |
            Hook::DirectJump { trampoline, .. } => {
                Some(*trampoline)
            },
            _ => None
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
            #[cfg(feature = "alloc_trampoline")]
            Self::Jump5 { entry, .. } => {
                skse64::trampoline::write_jump5(Trampoline::Global, addr, *entry as usize);
            },
            #[cfg(feature = "alloc_trampoline")]
            Self::Call5(entry) => {
                skse64::trampoline::write_call5(Trampoline::Global, addr, *entry as usize);
            },
            #[cfg(feature = "alloc_trampoline")]
            Self::Jump6 { entry, .. } => {
                skse64::trampoline::write_jump6(Trampoline::Global, addr, *entry as usize);
            },
            #[cfg(feature = "alloc_trampoline")]
            Self::Call6(entry) => {
                skse64::trampoline::write_call6(Trampoline::Global, addr, *entry as usize);
            },
            Self::Jump12 { entry, clobber, .. } => {
                write_flow(addr, *entry as usize, Flow::JumpRegAbsolute(*clobber)).unwrap();
            },
            Self::Call12 { entry, clobber, .. } => {
                write_flow(addr, *entry as usize, Flow::CallRegAbsolute(*clobber)).unwrap();
            },
            Self::Jump14 { entry, .. } => {
                write_flow(addr, *entry as usize, Flow::JumpAbsolute).unwrap();
            },
            Self::Call16(entry) => {
                write_flow(addr, *entry as usize, Flow::CallAbsolute).unwrap();
            },
            Self::DirectJump { entry, .. } => {
                write_flow(addr, *entry as usize, Flow::JumpRelative).unwrap();
            },
            Self::DirectCall(entry) => {
                write_flow(addr, *entry as usize, Flow::CallRelative).unwrap();
            },
            Self::None => panic!("Cannot install to a None hook!"),
        }
    }

    ///
    /// Verifies that the given patch was installed correctly.
    ///
    /// In order to use this function safely, the given address must be a part of the games code.
    ///
    unsafe fn verify(
        &self,
        addr: usize
    ) -> Result<(), ()> {
        match self {
            #[cfg(feature = "alloc_trampoline")]
            Self::Jump5 { .. } | Self::Call5(_) | Self::Jump6 { .. } | Self::Call5(_) => {
                todo!();
            },
            Self::Jump12 { entry, clobber, .. } => {
                verify_flow(addr, *entry as usize, Flow::JumpRegAbsolute(*clobber))
            },
            Self::Call12 { entry, clobber, .. } => {
                verify_flow(addr, *entry as usize, Flow::CallRegAbsolute(*clobber))
            },
            Self::Jump14 { entry, .. } => {
                verify_flow(addr, *entry as usize, Flow::JumpAbsolute)
            },
            Self::Call16(entry) => {
                verify_flow(addr, *entry as usize, Flow::CallAbsolute)
            },
            Self::DirectJump { entry, .. } => {
                verify_flow(addr, *entry as usize, Flow::JumpRelative)
            },
            Self::DirectCall(entry) => {
                verify_flow(addr, *entry as usize, Flow::CallRelative)
            },
            Self::None => Ok(()),
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
                // Incompatible game version needs to take priority, or we'll try to report on
                // a patch that should be invisible.
                if !loc.compatible() {
                    return Err(DescriptorError::IncompatibleGameVersion);
                }

                if !enabled() {
                    return Err(DescriptorError::Disabled);
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
            },
            Err(DescriptorError::IncompatibleGameVersion) => (), // Invalid, so we ignore it.
        }
    }

    /// Creates a patch result for the descriptor, if the descriptor is a patch.
    fn patch_result(
        &self,
        addr: RelocAddr
    ) -> Option<PatchResult> {
        match self {
            Self::Patch { name, hook, conflicts, .. } => {
                Some(PatchResult {
                    name: *name,
                    hook: hook.clone(),
                    conflicts: conflicts.unwrap_or("None"),
                    loc: addr
                })
            },
            _ => None
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
            Self::Patch { enabled, loc, .. } => !enabled() || !loc.compatible(),
            Self::Function { loc, .. } | Self::Object { loc, .. } => !loc.compatible()
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
    ) -> NonNull<UnsafeCell<usize>> {
        unsafe { NonNull::new_unchecked(&self.0 as *const _ as *mut _) }
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

impl PatchResult {
    /// Verifies that the given patch is installed to the game code.
    fn verify(
        &self
    ) -> Result<(), ()> {
        unsafe {
            // SAFETY: We give this hook the address it was installed to.
            self.hook.verify(self.loc.addr())
        }
    }
}

impl PatchSet {
    /// Verifies that the given patch set has correctly been installed.
    pub fn verify(
        &self
    ) {
        let mut fails = 0;
        for patch in self.0.iter() {
            if let Err(_) = patch.verify() {
                skse_message!("[ERROR] Patch {} has been clobbered!", patch.name);
                skse_fatal!(
                    "The integrity checker has determined that the patch {} was \
                     partially or completely overwritten by a conflicting plugin. \
                     This is fatal. Please disable the conflicting plugin or modify \
                     the INI file to disable this patch.\n\n\
                     Known conflicts: {}",
                    patch.name, patch.conflicts => window
                );

                fails += 1;
            }
        }

        if fails > 0 {
            panic!("Patch integrity verification found one or more installation errors");
        }
    }
}

// SAFETY: The patcher is protected by the single initialization of skse.
unsafe impl Sync for Hook {}
unsafe impl Sync for Descriptor {}
unsafe impl<T> Sync for GameRef<T> {}

/// Uses the version database to locate the patches that the user requested be installed.
fn locate_patches<const NUM_PATCHES: usize>(
    patches: &[&Descriptor]
) -> Result<([usize; NUM_PATCHES], Vec<PatchResult>, usize), ()> {
    let db = VersionDb::new(None);
    let mut res_addrs: [usize; NUM_PATCHES] = [0; NUM_PATCHES];
    let mut installed_patches: Vec<PatchResult> = Vec::new();

    let mut _alloc_size: usize = 0;

    // Attempt to locate all of the patch signatures.
    let mut fails = 0;
    for (i, sig) in patches.iter().enumerate() {
        let res = sig.find(&db);
        sig.report(&res);

        match res {
            Ok(addr) => {
                assert!(sig.hook().map(|h| h.patch_size() <= sig.size()).unwrap_or(true));
                res_addrs[i] = addr.addr();

                #[cfg(feature = "alloc_trampoline")]
                if let Some(h) = sig.hook() {
                    _alloc_size += h.alloc_size();
                }

                if let Some(patch_result) = sig.patch_result(addr) {
                    installed_patches.push(patch_result);
                }
            },
            Err(DescriptorError::Disabled) | Err(DescriptorError::IncompatibleGameVersion) => (),
            _ => {
                fails += 1;
            }
        }
    }

    if fails == 0 {
        Ok((res_addrs, installed_patches, _alloc_size))
    } else {
        Err(())
    }
}

/// Installs the set of previously located patches.
fn install_patches(
    patches: &[&Descriptor],
    res_addrs: &[usize],
    set: PatchSet
) {
    // Install our patches.
    for (i, sig) in patches.iter().enumerate() {
        if sig.disabled() { continue; }

        let hook_size = sig.hook().map(|h| h.patch_size()).unwrap_or(0);
        let ret_addr = res_addrs[i] + hook_size;
        match sig {
            Descriptor::Patch { hook, .. } => {
                unsafe {
                    // SAFETY: We will ensure our return address is valid by writing NOPS to any
                    //         bytes that are part of the patch and after the return address.
                    if let Some(t) = hook.trampoline() {
                        *(t.as_ref().get()) = ret_addr;
                    }
                    hook.install(res_addrs[i]);
                }
            },
            Descriptor::Function { result, .. } | Descriptor::Object { result, .. } => {
                unsafe {
                    // SAFETY: The version DB ensures that we have the write object for
                    //         the requested ID.
                    *(result.as_ref().get()) = res_addrs[i];
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

    // Register the patches to be verified later.
    static INSTALLED_PATCHES: RacyCell<Vec<PatchSet>> = RacyCell::new(Vec::new());
    static DO_ONCE: RacyCell<bool> = RacyCell::new(true);
    unsafe {
        // SAFETY: SKSE plugin loading is single threaded, so its safe to mutate here.
        (*INSTALLED_PATCHES.get()).push(set);
        if *DO_ONCE.get() {
            *DO_ONCE.get() = false;

            //
            // Verify the patch integrity just before the main window opens.
            //
            // Some plugins (e.g. Experience, I think) will apply patches during the post-load
            // phase, so we can't actually panic there.
            //
            skse64::event::register_listener(Message::SKSE_POST_POST_LOAD, |_| {
                for set in (*INSTALLED_PATCHES.get()).drain(0..) {
                    set.verify();
                }
            });
        }
    }
}

/// Locates any game functions/objects, and applies any code patches.
pub fn apply<const NUM_PATCHES: usize>(
    patches: [&Descriptor; NUM_PATCHES]
) -> Result<(), ()> {
    skse_message!(
        "--------------------- Skyrim Patcher {} ---------------------",
        env!("CARGO_PKG_VERSION")
    );

    let (res_addrs, to_install, _alloc_size) = locate_patches::<NUM_PATCHES>(&patches).map_err(|_| {
        skse_message!("[FAILURE] Could not locate every game signature!");
        skse_message!("----------------------------------------------------------------");
    })?;

    // Allocate our branch trampoline.
    #[cfg(feature = "alloc_trampoline")]
    if _alloc_size > 0 {
        // SAFETY: We're not giving an image base, so this is actually safe.
        unsafe { skse64::trampoline::create(Trampoline::Global, _alloc_size, None) };
        skse_message!(
            "[SUCCESS] Created branch trampoline buffer with a size of {} bytes",
            _alloc_size
        );
    } else {
        skse_message!("[SKIPPED] No patches require a branch trampoline allocation");
    }

    install_patches(&patches, &res_addrs, PatchSet(to_install));

    skse_message!("[SUCCESS] Applied game patches.");
    skse_message!("----------------------------------------------------------------");
    Ok(())
}
