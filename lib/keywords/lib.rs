//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Keywords/blocks which should be in Rust, but aren't.
//!
//! This file declares a number of macros which provides features that Rust probably *should*
//! provide by default. This includes:
//!   - A method of scoping the question mark operator.
//!   - A method of initializing static arrays where the size of the array has no real meaning and
//!     thus can't easily be defined.
//!   - A method of declaring abstract types, where the internal data layout is unknown.
//!

#![no_std]

///
/// Scopes the question mark operator within each of its blocks.
///
/// This macro can either be used as:
///     test! {{ /* stuff */ }}
/// or:
///     test! {{ /* stuff */ } catch(e) { }},
/// where the catch block is a call to map_err() on the original try block.
///
#[macro_export]
macro_rules! test {
    ( $try:block ) => {
        (|| $try)()
    };

    ( $try:block catch($arg:ident) $catch:block ) => {
        ::core::result::Result::map_err((|| $try)(), |$arg| $catch)
    };
}

///
/// Allows for a dynamically sized initialization of an array, capturing its size
/// in the identifier specified in the array type.
///
#[macro_export]
macro_rules! disarray {
    // Size capturing.
    ( $(#[$meta:meta])* $scope:vis static $arr:ident: [$type:ty; $size:ident] = [
        $($items:expr),*
    ]; ) => {
        $scope const $size: usize = $crate::disarray!(@maybe_count $($items),*);
        $(#[$meta])* $scope static $arr: [$type; $size] = [ $($items),* ];
    };

    // Non-capturing.
    ( $(#[$meta:meta])* $scope:vis static $arr:ident: [$type:ty] = [
        $($items:expr),*
    ]; ) => {
        $(#[$meta])* $scope static $arr: [$type; $crate::disarray!(@maybe_count $($items),*)] = [
            $($items),*
        ];
    };

    // Empty array len angers the compiler (idk).
    ( @maybe_count ) => { 0 };
    ( @maybe_count $($items:expr),+ ) => { [ $($crate::disarray!(@count $items)),* ].len() };

    // Make sure items are const.
    ( @count $item:expr ) => { 0 };
}

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
