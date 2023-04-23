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
use std::slice;
use std::ffi::c_void;
use std::mem::size_of;

use windows_sys::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE};

use skse64::log::{skse_message, skse_fatal};
use skse64::reloc::RelocAddr;
use skse64::plugin_api::Message;
use versionlib::VersionDb;
use racy_cell::RacyCell;

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

/// The maximum patch size. Chosen as our largest patch size is 16 (call absolute).
const MAX_ASM_HOOK_SIZE: usize = 16;

/// Represents an assembled x86-64 patch to be written.
struct AssemblyHook {
    addr: usize,
    buf: [u8; MAX_ASM_HOOK_SIZE],
    len: usize
}

/// Encodes the addressing mode of an instruction, or its opcode.
enum Encoding {
    JumpNear,
    CallRelative,
    JumpRelative,
    CallIndirect,
    JumpIndirect,
    CallReg(Register),
    JumpReg(Register),
    MoveImmQReg(Register),
    RelativeD(usize),
    AbsoluteSH(i8),
    AbsoluteD(u32),
    AbsoluteQ(u64)
}

///
/// An enumeration of the different types of control flow which can be written.
///
/// Indirect calling methods require an address to write the trampoline to.
///
enum Flow {
    CallRelative,
    JumpRelative,
    CallAbsolute,
    JumpAbsolute,
    CallRegAbsolute(Register),
    JumpRegAbsolute(Register),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Signature definitions
////////////////////////////////////////////////////////////////////////////////////////////////////

///
/// @brief Used to match code to pre-defined signatures.
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

/// @brief Generates a new signature out of hex digits and question marks.
#[macro_export]
macro_rules! signature {
    ( $($sig:tt),+; $size:literal ) => {{
        let psize = [ $($crate::signature!(@munch $sig)),* ].len();
        if $size != psize {
            ::std::panic!("Patch size is incorrect.");
        }
        $crate::Signature::new(&[ $($crate::signature!(@munch $sig)),* ])
    }};

    ( @munch $op:literal ) => { $crate::Opcode::Code($op) };
    ( @munch ? )           => { $crate::Opcode::Any       };
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Helper to print a signature in the games code.
#[derive(Debug)]
struct BinarySig(RelocAddr, usize);

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

///
/// Contains an address retrieved by the patcher.
///
/// This structure is a transparent usize, as some results may be visible to ASM code.
///
#[repr(transparent)]
pub struct GameRef<T>(UnsafeCell<usize>, std::marker::PhantomData<T>);

////////////////////////////////////////////////////////////////////////////////////////////////////

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
// Patcher implementation
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Flattens multiple arrays of patches into a single array.
pub fn flatten_patch_groups<const N: usize>(
    groups: &[&'static [Descriptor]]
) -> [&'static Descriptor; N] {
    let mut res = std::mem::MaybeUninit::<[&Descriptor; N]>::uninit();

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
    /// Gets the "on-site" patch size of the hook.
    fn patch_size(
        &self
    ) -> usize {
        match self {
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
        let res = (|| { match self {
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
                // SAFETY: We know addr is in the skyrim binary, since it came from the db.
                unsafe { sig.check(addr.addr())?; }
                Ok(addr)
            }
        }})();

        match &res {
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

        return res;
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

/// Locates any game functions/objects, and applies any code patches.
pub fn apply<const NUM_PATCHES: usize>(
    patches: [&Descriptor; NUM_PATCHES]
) -> Result<(), ()> {
    skse_message!(
        "--------------------- Skyrim Patcher {} ---------------------",
        env!("CARGO_PKG_VERSION")
    );

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Find patches
    ////////////////////////////////////////////////////////////////////////////////////////////////

    let db = VersionDb::new(skse64::version::current_runtime());
    let mut res_addrs: [usize; NUM_PATCHES] = [0; NUM_PATCHES];
    let mut installed_patches: PatchSet = PatchSet(Vec::new());

    // Attempt to locate all of the patch signatures.
    let mut fails = 0;
    for (i, sig) in patches.iter().enumerate() {
        match sig.find(&db) {
            Ok(addr) => {
                assert!(sig.hook().map(|h| h.patch_size() <= sig.size()).unwrap_or(true));
                res_addrs[i] = addr.addr();

                if let Some(patch_result) = sig.patch_result(addr) {
                    installed_patches.0.push(patch_result);
                }
            },
            Err(DescriptorError::Disabled) | Err(DescriptorError::IncompatibleGameVersion) => (),
            _ => fails += 1
        }
    }

    if fails > 0 {
        skse_message!("[FAILURE] Could not locate every game signature!");
        skse_message!("----------------------------------------------------------------");
        return Err(());
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Install patches
    ////////////////////////////////////////////////////////////////////////////////////////////////

    for (i, sig) in patches.iter().enumerate() {
        if sig.disabled() { continue; }

        let hook_size = sig.hook().map(|h| h.patch_size()).unwrap_or(0);
        let ret_addr = res_addrs[i] + hook_size;
        unsafe {
            match sig {
                Descriptor::Patch { hook, .. } => {
                    // SAFETY: We will ensure our return address is valid by writing NOPS to any
                    //         bytes that are part of the patch and after the return address.
                    if let Some(t) = hook.trampoline() {
                        *(t.as_ref().get()) = ret_addr;
                    }
                    hook.install(res_addrs[i]);
                },
                Descriptor::Function { result, .. } | Descriptor::Object { result, .. } => {
                    // SAFETY: The version DB ensures that we have the write object for
                    //         the requested ID.
                    *(result.as_ref().get()) = res_addrs[i];
                }
            }

            // SAFETY: We have matched signatures to ensure our patch is valid.
            let remain = sig.size() - hook_size;
            if remain > 0 {
                use_region(ret_addr, remain, || {
                    ::std::ptr::write_bytes::<u8>(ret_addr as *mut u8, 0x90, remain);
                });
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
            skse64::event::register_listener(Message::SKSE_POST_POST_LOAD, |_| {
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
    ) -> Result<(), DescriptorError> {
        assert!(a != 0);
        if self.0.len() == 0 { return Ok(()); }

        let mut diff = 0;
        use_region(a, self.0.len(), || {
            for (i, op) in self.0.iter().enumerate() {
                if let Opcode::Code(b) = *op {
                    diff += if b == *(a as *mut u8).add(i) { 0 } else { 1 };
                }
            }
        });

        if diff > 0 {
            Err(DescriptorError::Mismatch(*self, BinarySig(RelocAddr::from_addr(a), self.len())))
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

impl BinarySig {
    fn reloc(
        &self
    ) -> RelocAddr {
        self.0
    }
}

impl std::fmt::Display for Signature {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>
    ) -> Result<(), std::fmt::Error> {
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

impl std::fmt::Display for BinarySig {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>
    ) -> Result<(), std::fmt::Error> {
        let mut res = Ok(());

        unsafe {
            // SAFETY: The caller of the diff function ensures this is a valid sig.
            use_region(self.0.addr(), self.1, || {
                let sig = std::slice::from_raw_parts(self.0.addr() as *const u8, self.1);

                if let Err(e) = write!(f, "{{ ") {
                    res = Err(e);
                    return;
                }

                for b in sig.iter() {
                    if let Err(e) = write!(f, "{:02x} ", b) {
                        res = Err(e);
                        return;
                    }
                }

                res = write!(f, "}}");
            });
        }

        return res;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Code injection implementation
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
                Encoding::JumpNear => this.append(&[0xeb]),
                Encoding::CallRelative => this.append(&[0xe8]),
                Encoding::JumpRelative => this.append(&[0xe9]),
                Encoding::CallIndirect => this.append(&[0xff, 0x15]),
                Encoding::JumpIndirect => this.append(&[0xff, 0x25]),
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
                Encoding::AbsoluteSH(h) => this.append(&[*h as u8]),
                Encoding::AbsoluteD(d) => unsafe {
                    this.append(
                        slice::from_raw_parts(d as *const u32 as *const u8, size_of::<u32>())
                    );
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

    /// Applies the given patch, protecting the region.
    unsafe fn apply(
        self
    ) {
        use_region(self.addr, self.len, || {
            std::ptr::copy(self.buf.as_ptr(), self.addr as *mut u8, self.len);
        });
    }

    /// Verifies that the given region of code contains the patch, protecting the region.
    unsafe fn verify(
        self
    ) -> Result<(), ()> {
        let mut ret = Err(());
        use_region(self.addr, self.len, || {
            let patch = self.buf.split_at(self.len).0;
            let code = std::slice::from_raw_parts(self.addr as *mut u8, self.len);
            ret = if code == patch { Ok(()) } else { Err(()) };
        });
        return ret;
    }
}

impl Flow {
    ///
    /// Creates a new patch from the invoking flow, address, and target.
    ///
    /// If the flow is an indirect, the target is written to the trampoline on success.
    ///
    fn as_patch(
        &self,
        addr: usize,
        target: usize
    ) -> Result<AssemblyHook, ()> {
        let patch = match self {
            Flow::CallRelative => AssemblyHook::new(addr,  &[
                Encoding::CallRelative, Encoding::RelativeD(target)
            ]),

            Flow::JumpRelative => AssemblyHook::new(addr, &[
                Encoding::JumpRelative, Encoding::RelativeD(target)
            ]),

            Flow::CallAbsolute => AssemblyHook::new(addr, &[
                Encoding::CallIndirect, Encoding::AbsoluteD(0x02),
                Encoding::JumpNear, Encoding::AbsoluteSH(0x08),
                Encoding::AbsoluteQ(target as u64)
            ]),
            Flow::JumpAbsolute => AssemblyHook::new(addr, &[
                Encoding::JumpIndirect, Encoding::AbsoluteD(0),
                Encoding::AbsoluteQ(target as u64)
            ]),
            Flow::CallRegAbsolute(reg) => AssemblyHook::new(addr, &[
                Encoding::MoveImmQReg(*reg), Encoding::AbsoluteQ(target as u64),
                Encoding::CallReg(*reg)
            ]),
            Flow::JumpRegAbsolute(reg) => AssemblyHook::new(addr, &[
                Encoding::MoveImmQReg(*reg), Encoding::AbsoluteQ(target as u64),
                Encoding::JumpReg(*reg)
            ]),
        }?;

        Ok(patch)
    }
}

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

///
/// Writes the given instruction type to the given address, changing RIP to the given target.
///
/// This function may be called on code which is not currently marked read/write.
///
/// In order to use this function safely, the given address must be in the skyrim binary.
///
unsafe fn write_flow(
    addr: usize,
    target: usize,
    flow: Flow
) -> Result<(), ()> {
    flow.as_patch(addr, target)?.apply();
    Ok(())
}

///
/// Verifies that the given control flow operation was installed correctly to the given address.
///
/// This function may be called on code which is not currently marked read/write.
///
/// In order to use this function safely, the given address must be in the skyrim binary.
///
unsafe fn verify_flow(
    addr: usize,
    target: usize,
    flow: Flow
) -> Result<(), ()> {
    flow.as_patch(addr, target)?.verify()
}
