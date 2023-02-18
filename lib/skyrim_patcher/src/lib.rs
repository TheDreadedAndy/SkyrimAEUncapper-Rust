//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Library for applying patches to the skyrim binary.
//! @bug no known bugs.
//!

mod patcher;
mod sig;

pub use patcher::*;
pub use sig::*;

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
