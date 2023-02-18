//!
//! @file util.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Utility macros for defining types from the game.
//! @bug No known bugs.
//!

///
/// Allows the definition of any number of abstract types, with layouts unknown to Rust.
///
/// Types declared here will automatically have repr(C) applied.
///
#[macro_export]
macro_rules! abstract_type {
    ( $( $(#[$meta:meta])* $scope:vis type $name:ident );+; ) => {
        $($(#[$meta])* #[repr(C)] $scope struct $name {
            // Stop construction - without this anyone can construct.
            _private: [u8; 0],

            // Prevent the compiler from marking as Send, Sync, or Unpin.
            _marker: ::std::marker::PhantomData<(*mut u8, ::std::marker::PhantomPinned)>
        })*
    };
}

pub use abstract_type;
