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

use core::cell::UnsafeCell;
use core::ptr::NonNull;
use core::slice;
use core::ffi::c_void;
use core::mem::size_of;
use alloc::vec::Vec;

use sre_common::versiondb::{VersionDbStream, DatabaseItem};
use core_util::RacyCell;
use core_util::attempt;

use windows_sys::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE};

use crate::version;
use crate::plugin_api;
use crate::log::{skse_message, skse_fatal};
use crate::reloc::RelocAddr;
use crate::plugin_api::Message;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Code injection definitions
////////////////////////////////////////////////////////////////////////////////////////////////////
//
// These definitions are used to inject our patch hooks into the skyrim game code.
//
// Note that the injection implementations themselves were originally wrappers around the skse
// code, but were eventually completely rewritten into the current implementation, which takes
// a much different approach to assembling the required the instructions.

/// Encodes a x86-64 +rq register index.
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Register {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    Rbx = 3,
    Rsp = 4,
    Rbp = 5,
    Rsi = 6,
    Rdi = 7
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Signature definitions
////////////////////////////////////////////////////////////////////////////////////////////////////

///
/// Used to match code to pre-defined signatures.
///
/// This enumeration is used in the system that ensures that, regardless of game version, the
/// intended code is being overwritten.
///
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Opcode {
    Code(u8),
    Any
}

/// Identifies a distinct string of binary code within the skyrim binary.
#[derive(Copy, Clone, Debug)]
pub struct Signature(&'static [Opcode]);

/// Generates a new signature out of hex digits and question marks.
#[macro_export]
macro_rules! signature {
    ( $($sig:tt),+; $size:literal ) => {{
        let psize = [ $($crate::signature!(@munch $sig)),* ].len();
        $crate::core::assert!($size == psize, "Patch size is incorrect.");
        $crate::patcher::Signature::new(&[ $($crate::signature!(@munch $sig)),* ])
    }};

    ( @munch $op:literal ) => { $crate::patcher::Opcode::Code($op) };
    ( @munch ? )           => { $crate::patcher::Opcode::Any       };
}
pub use signature;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Helper to print a signature in the games code.
#[derive(Copy, Clone, Debug)]
struct BinarySig {
    addr: RelocAddr,
    len: usize
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Patcher definitions
////////////////////////////////////////////////////////////////////////////////////////////////////

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
}

/// An object in Skyrim's code to be located by the patcher, and modified if necessary.
pub enum DescriptorObject {
    Function(NonNull<UnsafeCell<usize>>),
    Global(NonNull<UnsafeCell<usize>>),
    Patch {
        enabled: fn() -> bool,
        conflicts: Option<&'static str>,
        hook: Hook,
        sig: Signature,
    }
}

/// Describes a named location in the games code to be found and used by the patcher.
pub struct Descriptor {
    pub name: &'static str,
    pub loc: GameLocation,
    pub object: DescriptorObject
}

///
/// Contains an address retrieved by the patcher.
///
/// This structure is a transparent usize, as some results may be visible to ASM code.
///
#[repr(transparent)]
pub struct GameRef<T>(UnsafeCell<usize>, core::marker::PhantomData<T>);

////////////////////////////////////////////////////////////////////////////////////////////////////
// Patcher implementation
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Maintains the current state of a patch/object that is in the process of being located/installed.
#[derive(Copy, Clone)]
enum DescriptorState {
    None,
    Incompatible,
    Disabled,
    Unresolved(usize, usize),
    Mismatch(Signature, BinarySig),
    Resolved(RelocAddr)
}

/// Contains information about a patch necessary to verify its integrity.
struct PatchResult {
    name: &'static str,
    conflicts: &'static str,
    hook: Hook,
    loc: RelocAddr
}

/// Contains the set of patches installed by a call to apply().
struct PatchSet(Vec<PatchResult>);

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Locates any game functions/objects, and applies any code patches.
pub fn apply<const NUM_PATCHES: usize>(
    patches: [&Descriptor; NUM_PATCHES]
) -> Result<(), ()> {
    skse_message!(
        "--------------------- Skyrim Patcher {} ---------------------",
        env!("CARGO_PKG_VERSION")
    );

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Initialize descriptor state
    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Before we can apply any patches to the game code, we must determine what set of patch
    // descriptors are valid for us to use for the current game version. We also need to check if
    // any of the patches in the list have been disabled at runtime.

    let mut desc_state : [DescriptorState; NUM_PATCHES] = [DescriptorState::None; NUM_PATCHES];
    let mut unresolved : usize                          = 0;

    for (i, descriptor) in patches.iter().enumerate() {
        desc_state[i] = if let Ok((id, offset)) = descriptor.loc.get() {
            DescriptorState::Unresolved(id, offset)
        } else {
            DescriptorState::Incompatible
        };

        if let DescriptorObject::Patch { enabled, .. } = descriptor.object {
            desc_state[i] = if enabled() { desc_state[i] } else { DescriptorState::Disabled };
        }

        if let DescriptorState::Unresolved(..) = desc_state[i] {
            unresolved += 1;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Resolve patches
    ////////////////////////////////////////////////////////////////////////////////////////////////

    let mut installed_patches = PatchSet(Vec::new());

    // Attempt to locate all of the patch signatures.
    for DatabaseItem { id, addr } in VersionDbStream::new(version::current_runtime()) {
        for i in 0..desc_state.len() {
            if let DescriptorState::Unresolved(desc_id, desc_offset) = desc_state[i] {
                if id != desc_id { continue; }
                unresolved -= 1;

                let addr = addr + desc_offset;
                if let DescriptorObject::Patch { hook, sig, conflicts, .. } = &patches[i].object {
                    // SAFETY: We know addr is in the skyrim binary, since it came from the db.
                    if let Err(mismatch) = unsafe { sig.check(addr.addr()) } {
                        desc_state[i] = DescriptorState::Mismatch(*sig, mismatch);
                        continue;
                    }

                    // Add the patch to the list of successfully installed patches.
                    installed_patches.0.push(PatchResult {
                        name: patches[i].name,
                        hook: hook.clone(),
                        conflicts: conflicts.unwrap_or("None"),
                        loc: addr
                    });
                }
                desc_state[i] = DescriptorState::Resolved(addr);
            }
        }

        // Stop streaming the database once we've found all we need.
        if unresolved == 0 { break; }
    }

    let mut failed = false;
    for i in 0..NUM_PATCHES {
        match desc_state[i] {
            DescriptorState::Resolved(addr) => {
                skse_message!( "[SUCCESS] {} is at offset {:#x}", patches[i], addr.offset());
            },
            DescriptorState::Disabled => {
                failed = true;
                skse_message!("[SKIPPED] {} is disabled", patches[i]);
            },
            DescriptorState::Unresolved(..) => {
                failed = true;
                skse_message!("[FAILURE] {} was not in the version database!", patches[i]);
            },
            DescriptorState::Mismatch(sig, bsig) => {
                failed = true;
                skse_message!(
                    "[FAILURE] {} at offset {:#x} did not match the expected code signature!",
                    patches[i],
                    bsig.addr.offset()
                );
                skse_message!("\\------> [EXPECTED] {}", sig);
                skse_message!(" \\-----> [FOUND...] {}", bsig);
            },
            _ => ()
        }
    }

    if failed {
        skse_message!("[FAILURE] Could not locate every game signature!");
        skse_message!("----------------------------------------------------------------");
        return Err(());
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Install patches
    ////////////////////////////////////////////////////////////////////////////////////////////////

    for i in 0..NUM_PATCHES {
        let addr = if let DescriptorState::Resolved(a) = desc_state[i] { a } else { continue; };

        unsafe {
            match &patches[i].object {
                DescriptorObject::Patch { hook, sig, .. } => {
                    let hook_size = hook.patch_size();
                    let ret_addr = addr.addr() + hook_size;

                    // SAFETY: We will ensure our return address is valid by writing NOPS to any
                    //         bytes that are part of the patch and after the return address.
                    match hook {
                        Hook::Jump12 { trampoline, .. } |
                        Hook::DirectJump { trampoline, .. } => {
                            *(trampoline.as_ref().get()) = ret_addr;
                        },
                        _ => ()
                    }

                    // SAFETY: We have matched signatures to ensure our patch is valid.
                    use_region(addr.addr(), sig.len(), || {
                        if let Some(asm_hook) = hook.get_install_asm(addr.addr()).unwrap() {
                                core::ptr::copy(asm_hook.buf.as_ptr(), asm_hook.addr as *mut u8,
                                                asm_hook.len);
                        }
                        core::ptr::write_bytes::<u8>(ret_addr as *mut u8, 0x90 /* NOP */,
                                                     sig.len() - hook_size);
                    });
                },

                DescriptorObject::Function(result) | DescriptorObject::Global(result) => {
                    // SAFETY: The version DB ensures that we have the write object for
                    //         the requested ID.
                    *(result.as_ref().get()) = addr.addr();
                }
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Integrity check patches
    ////////////////////////////////////////////////////////////////////////////////////////////////

    static INSTALLED_PATCHES: RacyCell<Vec<PatchSet>> = RacyCell::new(Vec::new());
    static DO_ONCE: RacyCell<bool> = RacyCell::new(true);
    unsafe {
        // SAFETY: SKSE plugin loading is single threaded, so its safe to mutate here.
        (*INSTALLED_PATCHES.get()).push(installed_patches);
        if *DO_ONCE.get() {
            *DO_ONCE.get() = false;

            //
            // Verify the patch integrity just before the main window opens.
            //
            // Some plugins (e.g. Experience, I think) will apply patches during the post-load
            // phase, so we can't actually panic there.
            //
            plugin_api::register_listener(Message::SKSE_POST_POST_LOAD, |_| {
                for set in (*INSTALLED_PATCHES.get()).drain(0..) {
                    set.verify();
                }
            });
        }
    }

    skse_message!("[SUCCESS] Applied game patches.");
    skse_message!("----------------------------------------------------------------");
    Ok(())
}

/// Flattens multiple arrays of patches into a single array.
pub fn flatten_patch_groups<const N: usize>(
    groups: &[&'static [Descriptor]]
) -> [&'static Descriptor; N] {
    let mut res = core::mem::MaybeUninit::<[&Descriptor; N]>::uninit();

    let mut i = 0;
    for g in groups.iter() {
        for d in g.iter() {
            unsafe { (*res.as_mut_ptr())[i] = d; }
            i += 1;
        }
    }

    assert!(i == N);
    unsafe {
        res.assume_init()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl GameLocation {
    /// Gets the id/offset for the running version of skyrim, if it is available.
    fn get(
        &self
    ) -> Result<(usize, usize), ()> {
        let is_se = version::current_runtime() <= version::RUNTIME_VERSION_1_5_97;
        match *self {
            Self::Base { se, .. }               if is_se  => Ok((se, 0)),
            Self::Base { ae, .. }               if !is_se => Ok((ae, 0)),
            Self::Se   { id, offset }           if is_se  => Ok((id, offset)),
            Self::Ae   { id, offset }           if !is_se => Ok((id, offset)),
            Self::All  { id_se, offset_se, .. } if is_se  => Ok((id_se, offset_se)),
            Self::All  { id_ae, offset_ae, .. } if !is_se => Ok((id_ae, offset_ae)),
            _                                             => Err(())
        }
    }
}

impl core::fmt::Display for GameLocation {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>
    ) -> Result<(), core::fmt::Error> {
        let (id, offset) = self.get().unwrap();
        if offset == 0 {
            write!(f, "[ID: {}]", id)
        } else {
            write!(f, "([ID: {}] + {:#x})", id, offset)
        }
    }
}

impl Hook {
    /// Gets the "on-site" patch size of the hook.
    fn patch_size(
        &self
    ) -> usize {
        match self {
            Hook::None => 0,
            Hook::DirectJump { .. } | Hook::DirectCall(_) => 5,
            Hook::Jump12 { .. } | Hook::Call12 { .. } => 12
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
        if let Some(asm_hook) = self.get_install_asm(addr)? {
            let mut ret = Err(());
            use_region(asm_hook.addr, asm_hook.len, || {
                let patch = asm_hook.buf.split_at(asm_hook.len).0;
                let code = core::slice::from_raw_parts(asm_hook.addr as *mut u8, asm_hook.len);
                ret = if code == patch { Ok(()) } else { Err(()) };
            });
            return ret;
        } else {
            Ok(())
        }
    }

    /// Gets the assembly hook assocaited with the invoking hook.
    fn get_install_asm(
        &self,
        addr: usize
    ) -> Result<Option<AssemblyHook>, ()> {
        Ok(match self {
            Self::Jump12 { entry, clobber, .. } => Some(AssemblyHook::new(addr, &[
                Encoding::MoveImmQReg(*clobber), Encoding::AbsoluteQ(*entry as u64),
                Encoding::JumpReg(*clobber)
            ])?),

            Self::Call12 { entry, clobber, .. } => Some(AssemblyHook::new(addr, &[
                Encoding::MoveImmQReg(*clobber), Encoding::AbsoluteQ(*entry as u64),
                Encoding::CallReg(*clobber)
            ])?),

            Self::DirectJump { entry, .. } => Some(AssemblyHook::new(addr, &[
                Encoding::JumpRelative, Encoding::RelativeD(*entry as usize)
            ])?),

            Self::DirectCall(entry) => Some(AssemblyHook::new(addr,  &[
                Encoding::CallRelative, Encoding::RelativeD(*entry as usize)
            ])?),

            _ => None
        })
    }
}

impl core::fmt::Display for Descriptor {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>
    ) -> Result<(), core::fmt::Error> {
        match self.object {
            DescriptorObject::Global { .. }   => write!(f, "Global {} {}", self.name, self.loc),
            DescriptorObject::Function { .. } => write!(f, "Function {} {}", self.name, self.loc),
            DescriptorObject::Patch { .. }    => write!(f, "Patch {} {}", self.name, self.loc)
        }
    }
}

impl<T> GameRef<T> {
    /// Creates a new game reference structure.
    pub const fn new() -> Self {
        assert!(core::mem::size_of::<T>() == core::mem::size_of::<usize>());
        Self(UnsafeCell::new(0), core::marker::PhantomData)
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
        assert!(core::mem::size_of::<T>() == core::mem::size_of::<usize>());

        // SAFETY: We know that T is of the correct size for this transmute.
        unsafe {
            let addr = *self.0.get();
            assert!(addr != 0);
            core::mem::transmute_copy::<usize, T>(&addr)
        }
    }
}

impl PatchResult {
    /// Verifies that the given patch is installed to the game code.
    fn verify(
        &self
    ) -> Result<(), ()> {
        // SAFETY: We give this hook the address it was installed to.
        unsafe { self.hook.verify(self.loc.addr()) }
    }
}

impl PatchSet {
    /// Verifies that the given patch set has correctly been installed.
    fn verify(
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Signature checking implementation
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Signature {
    /// Creates a new signature structure.
    pub const fn new(
        sig: &'static [Opcode]
    ) -> Self {
        Self(sig)
    }

    ///
    /// Checks the given signature against the given memory location.
    ///
    /// In order to use this function safely, the address range specified must be
    /// a valid part of the Skyrim binary.
    ///
    unsafe fn check(
        &self,
        a: usize
    ) -> Result<(), BinarySig> {
        assert!(a != 0);
        if self.len() == 0 { return Ok(()); }

        let mut diff = 0;
        use_region(a, self.len(), || {
            for (i, op) in self.0.iter().enumerate() {
                if let Opcode::Code(b) = *op {
                    diff += if b == *(a as *mut u8).add(i) { 0 } else { 1 };
                }
            }
        });

        if diff > 0 {
            Err(BinarySig {
                addr: RelocAddr::from_addr(a),
                len: self.len()
            })
        } else {
            Ok(())
        }
    }

    /// Checks how long the signature is.
    fn len(
        &self
    ) -> usize {
        self.0.len()
    }
}

impl core::fmt::Display for Signature {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>
    ) -> Result<(), core::fmt::Error> {
        write!(f, "{{ ")?;
        for op in self.0.iter() {
            if let Opcode::Code(b) = op {
                write!(f, "{:02x} ", b)?;
            } else {
                write!(f, "?? ")?;
            }
        }
        write!(f, "}}")
    }
}

impl core::fmt::Display for BinarySig {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>
    ) -> Result<(), core::fmt::Error> {
        let mut res = Ok(());

        unsafe {
            // SAFETY: The caller of the diff function ensures this is a valid sig.
            use_region(self.addr.addr(), self.len, || {
                let sig = core::slice::from_raw_parts(self.addr.addr() as *const u8, self.len);
                res = attempt! {{
                    write!(f, "{{ ")?;
                    for b in sig.iter() { write!(f, "{:02x} ", b)?; }
                    write!(f, "}}")
                }};
            });
        }

        return res;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Assembly code generation
////////////////////////////////////////////////////////////////////////////////////////////////////

/// The maximum patch size. Chosen as our largest patch size is 16 (call absolute).
const MAX_ASM_HOOK_SIZE: usize = 12;

/// Represents an assembled x86-64 patch to be written.
struct AssemblyHook {
    addr: usize,
    buf: [u8; MAX_ASM_HOOK_SIZE],
    len: usize
}

/// Encodes the addressing mode of an instruction, or its opcode.
enum Encoding {
    CallRelative,
    JumpRelative,
    CallReg(Register),
    JumpReg(Register),
    MoveImmQReg(Register),
    RelativeD(usize),
    AbsoluteQ(u64)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl AssemblyHook {
    /// Generates a patch to be applied to the given address from the given encoding.
    fn new(
        addr: usize,
        chunks: &[Encoding]
    ) -> Result<Self, ()> {
        let mut this = Self {
            addr,
            buf: [0; MAX_ASM_HOOK_SIZE],
            len: 0
        };

        for chunk in chunks.iter() {
            match chunk {
                Encoding::CallRelative => this.append(&[0xe8]),
                Encoding::JumpRelative => this.append(&[0xe9]),
                Encoding::CallReg(reg) => this.append(&[0xff, 0xd0 + (*reg as u8)]),
                Encoding::JumpReg(reg) => this.append(&[0xff, 0xe0 + (*reg as u8)]),
                Encoding::MoveImmQReg(reg) => this.append(&[0x48, 0xb8 + (*reg as u8)]),
                Encoding::RelativeD(target) => {
                    let rel: i32 = (
                        target.wrapping_sub(addr + this.len + size_of::<i32>())
                    ).try_into().map_err(|_| ())?;

                    unsafe {
                        this.append(slice::from_raw_parts(
                            &rel as *const i32 as *const u8,
                            size_of::<i32>()
                        ));
                    }
                },
                Encoding::AbsoluteQ(q) => unsafe {
                    this.append(
                        slice::from_raw_parts(q as *const u64 as *const u8, size_of::<u64>())
                    );
                }
            }
        }

        Ok(this)
    }

    /// Appends the given bytes to the patch buffer.
    fn append(
        &mut self,
        s: &[u8]
    ) {
        self.buf.split_at_mut(self.len).1.split_at_mut(s.len()).0.copy_from_slice(s);
        self.len += s.len();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// OS goop
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Temporarily marks the given memory region for read/write, then calls the given fn.
unsafe fn use_region(
    addr: usize,
    size: usize,
    func: impl FnOnce()
) {
    let mut old_prot: u32 = 0;
    VirtualProtect(addr as *const c_void, size, PAGE_EXECUTE_READWRITE, &mut old_prot);
    func();
    VirtualProtect(addr as *const c_void, size, old_prot, &mut old_prot);
}
