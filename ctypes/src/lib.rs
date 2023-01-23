//!
//! @file ctype.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief Definitions for c types that are not (and can't be) in core.
//! @bug No known bugs.
//!

#![no_std]

mod prim;
mod cstr;

pub use prim::*;
pub use cstr::*;

// Necessary for macro hygiene
pub use ::core;
