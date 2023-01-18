//!
//! @file extern_method.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief Implements a macro for exposing struct methods over FFI.
//! @bug No known bugs.
//!

///
/// Allows for the easy exporting of methods on structs by creating associated FFI
/// functions. The syntax is as follows:
/// extern_method! {
///     <Function declaration> => <extern function name>;
///     __init__(...) => <extern function name>;
///     __destroy__() => <extern function name>;
/// }
/// where self, &self, and &mut self will be mapped to Self, *mut Self, and *mut Self
/// respectively in the function declaration, and @init and @destroy are special requests
/// that map back to new and drop respectively.
///
#[macro_export]
macro_rules! extern_method {
    ( $( $(#[$meta:meta])*
        $scope:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)? => $ext:ident
    );+; ) => {
        $($crate::extern_method! {
            @fn $(#[$meta])* $scope fn $name($($args)*) $(-> $ret)? => $ext
        })*
    };

    ( @fn $(#[$meta:meta])* $scope:vis fn __init__(
        $($argn:ident: $argt:ty),*
    ) => $ext:ident ) => {
        $(#[$meta])* #[no_mangle] $scope unsafe extern "C" fn $ext(
            this: *mut $crate::core::mem::MaybeUninit<Self>
            $(, $argn: $argt)*
        ) {
            this.as_mut().unwrap().write(Self::new($($argn),*));
        }
    };

    ( @fn $(#[$meta:meta])* $scope:vis fn __destroy__() => $ext:ident ) => {
        $(#[$meta])* #[no_mangle] $scope unsafe extern "C" fn $ext(
            this: *mut Self
        ) {
            $crate::core::ptr::drop_in_place(this.as_mut().unwrap());
        }
    };

    ( @fn $(#[$meta:meta])* $scope:vis fn $name:ident(
        self
        $(, $argn:ident: $argt:ty)*
    ) $(-> $ret:ty)? => $ext:ident ) => {
        $(#[$meta])* #[no_mangle] $scope unsafe extern "C" fn $ext(
            this: Self
            $(, $argn: $argt)*
        ) $(-> $ret)? {
            this.$name($($argn),*)
        }
    };

    ( @fn $(#[$meta:meta])* $scope:vis fn $name:ident(
        &self
        $(, $argn:ident: $argt:ty)*
    ) $(-> $ret:ty)? => $ext:ident ) => {
        $(#[$meta])* #[no_mangle] $scope unsafe extern "C" fn $ext(
            this: *mut Self
            $(, $argn: $argt)*
        ) $(-> $ret)? {
            this.as_ref().unwrap().$name($($argn),*)
        }
    };

    ( @fn $(#[$meta:meta])* $scope:vis fn $name:ident(
        &mut self
        $(, $argn:ident: $argt:ty)*
    ) $(-> $ret:ty)? => $ext:ident ) => {
        $(#[$meta])* #[no_mangle] $scope unsafe extern "C" fn $ext(
            this: *mut Self
            $(, $argn: $argt)*
        ) $(-> $ret)? {
            this.as_mut().unwrap().$name($($argn),*)
        }
    };
}

///
/// Variant of extern_method! that first takes in a global variable and then another inner
/// block containing the functions to be remapped. Self need not be specified for these functions,
/// it will implicitely map all functions to &self.
///
#[macro_export]
macro_rules! global_extern_method {
    ( $global:ident {
        $($(#[$meta:meta])* $scope:vis fn $name:ident($($args:tt)*) $(-> $ret:ty)? => $ext:ident);+;
    } ) => {
        $($crate::global_extern_method! {
            @fn $global; $(#[$meta])* $scope fn $name($($args)*) $(-> $ret)? => $ext
        })*
    };

    ( @fn $global:ident; $(#[$meta:meta])* $scope:vis fn $name:ident(
        $($argn:ident: $argt:ty),*
    ) $(-> $ret:ty)? => $ext:ident ) => {
        $(#[$meta])* #[no_mangle] $scope unsafe extern "C" fn $ext(
            $($argn: $argt),*
        ) $(-> $ret)? {
            $global.$name($($argn),*)
        }
    };
}

pub use extern_method;
